#!/usr/bin/env python3
"""
E2E Test Runner for TUHC Tauri 2.0 Re-implementation.
Supports 4-tier requirement-driven testing in both standalone contract mode and live-attach mode.
"""

import os
import sys
import time
import json
import argparse
import unittest
import threading

# Add project root to sys.path
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)

from tests.common import TEST_CONFIG
from tests.mock_server import create_mock_server


class TierResultCollector(unittest.TestResult):
    """Custom TestResult to gather per-test and per-tier metrics."""

    def __init__(self):
        super().__init__()
        self.records = []
        self._start_time = None

    def startTest(self, test):
        super().startTest(test)
        self._start_time = time.time()

    def addSuccess(self, test):
        super().addSuccess(test)
        duration = time.time() - self._start_time if self._start_time else 0
        self.records.append({
            "test": str(test),
            "status": "PASS",
            "duration": round(duration, 4),
            "error": None,
        })

    def addFailure(self, test, err):
        super().addFailure(test, err)
        duration = time.time() - self._start_time if self._start_time else 0
        self.records.append({
            "test": str(test),
            "status": "FAIL",
            "duration": round(duration, 4),
            "error": self._exc_info_to_string(err, test),
        })

    def addError(self, test, err):
        super().addError(test, err)
        duration = time.time() - self._start_time if self._start_time else 0
        self.records.append({
            "test": str(test),
            "status": "ERROR",
            "duration": round(duration, 4),
            "error": self._exc_info_to_string(err, test),
        })

    def addSkip(self, test, reason):
        super().addSkip(test, reason)
        self.records.append({
            "test": str(test),
            "status": "SKIP",
            "duration": 0,
            "error": reason,
        })


def discover_suite(selected_tier: str = "all", pattern: str = None) -> unittest.TestSuite:
    """Discover tests matching tier and pattern criteria."""
    loader = unittest.TestLoader()
    suite = unittest.TestSuite()

    tier_dirs = {
        "1": os.path.join(SCRIPT_DIR, "tier1_features"),
        "2": os.path.join(SCRIPT_DIR, "tier2_boundaries"),
        "3": os.path.join(SCRIPT_DIR, "tier3_combinations"),
        "4": os.path.join(SCRIPT_DIR, "tier4_realworld"),
    }

    dirs_to_scan = []
    if selected_tier == "all":
        dirs_to_scan = list(tier_dirs.values())
    elif selected_tier in tier_dirs:
        dirs_to_scan = [tier_dirs[selected_tier]]
    else:
        raise ValueError(f"Unknown tier: {selected_tier}")

    for target_dir in dirs_to_scan:
        if not os.path.exists(target_dir):
            continue
        discovered = loader.discover(target_dir, pattern="test_*.py", top_level_dir=PROJECT_ROOT)
        for test in _flatten_suite(discovered):
            test_id = test.id()
            if pattern and pattern.lower() not in test_id.lower():
                continue
            suite.addTest(test)

    return suite


def _flatten_suite(suite):
    """Recursively flatten nested TestSuites into individual TestCases."""
    for item in suite:
        if isinstance(item, unittest.TestSuite):
            yield from _flatten_suite(item)
        else:
            yield item


def print_summary(records, total_elapsed):
    """Print terminal summary table."""
    total = len(records)
    passed = sum(1 for r in records if r["status"] == "PASS")
    failed = sum(1 for r in records if r["status"] == "FAIL")
    errors = sum(1 for r in records if r["status"] == "ERROR")
    skipped = sum(1 for r in records if r["status"] == "SKIP")

    print("\n" + "=" * 70)
    print("TUHC TAURI 2.0 E2E TEST EXECUTION SUMMARY")
    print("=" * 70)
    print(f"Total Tests Run: {total}")
    print(f"Passed:          {passed} \033[92m[OK]\033[0m")
    if failed > 0:
        print(f"Failed:          {failed} \033[91m[FAIL]\033[0m")
    if errors > 0:
        print(f"Errors:          {errors} \033[91m[ERROR]\033[0m")
    if skipped > 0:
        print(f"Skipped:         {skipped}")
    print(f"Elapsed Time:    {round(total_elapsed, 2)}s")
    print("-" * 70)

    # Tier breakdown
    tier_stats = {"Tier 1": [0, 0], "Tier 2": [0, 0], "Tier 3": [0, 0], "Tier 4": [0, 0]}
    for r in records:
        tname = r["test"]
        tier_key = None
        if "tier1_features" in tname:
            tier_key = "Tier 1"
        elif "tier2_boundaries" in tname:
            tier_key = "Tier 2"
        elif "tier3_combinations" in tname:
            tier_key = "Tier 3"
        elif "tier4_realworld" in tname:
            tier_key = "Tier 4"

        if tier_key:
            tier_stats[tier_key][0] += 1
            if r["status"] == "PASS":
                tier_stats[tier_key][1] += 1

    print("Tier Breakdown:")
    for tier, (count, pass_count) in tier_stats.items():
        if count > 0:
            pct = (pass_count / count) * 100
            print(f"  - {tier:8s}: {pass_count}/{count} passed ({pct:.1f}%)")
    print("=" * 70)

    if failed or errors:
        print("\n\033[91mFAILURES & ERRORS DETAILS:\033[0m")
        for r in records:
            if r["status"] in ("FAIL", "ERROR"):
                print(f"\n--- {r['test']} ---")
                print(r["error"])


def main():
    parser = argparse.ArgumentParser(description="TUHC Tauri E2E Test Runner")
    parser.add_argument("--tier", choices=["1", "2", "3", "4", "all"], default="all", help="Target test tier")
    parser.add_argument("-k", "--filter", default=None, help="Filter tests by substring")
    parser.add_argument("--live", action="store_true", help="Connect to running backend without mock server")
    parser.add_argument("--url", default=None, help="Backend URL (e.g. http://127.0.0.1:45100)")
    parser.add_argument("--json-report", default=None, help="Path to write JSON test results")
    parser.add_argument("-v", "--verbose", action="store_true", help="Verbose test execution")

    args = parser.parse_args()

    server = None
    server_thread = None

    if args.live:
        url = args.url or "http://127.0.0.1:45100"
        TEST_CONFIG["server_url"] = url
        TEST_CONFIG["use_live"] = True
        print(f"[*] Attaching tests to live backend at {url}")
    else:
        # Start reference contract server on ephemeral port
        server = create_mock_server(port=0)
        assigned_port = server.server_address[1]
        TEST_CONFIG["server_url"] = f"http://127.0.0.1:{assigned_port}"
        TEST_CONFIG["use_live"] = False

        server_thread = threading.Thread(target=server.serve_forever, daemon=True)
        server_thread.start()
        print(f"[*] Started reference contract server on port {assigned_port}")

    suite = discover_suite(selected_tier=args.tier, pattern=args.filter)
    collector = TierResultCollector()

    start_time = time.time()
    try:
        verbosity = 2 if args.verbose else 1
        runner = unittest.TextTestRunner(resultclass=lambda *a: collector, verbosity=verbosity)
        runner.run(suite)
    finally:
        total_time = time.time() - start_time
        if server:
            server.shutdown()
            server.server_close()

    print_summary(collector.records, total_time)

    if args.json_report:
        report_data = {
            "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "total_elapsed": round(total_time, 4),
            "total_tests": len(collector.records),
            "passed": sum(1 for r in collector.records if r["status"] == "PASS"),
            "failed": sum(1 for r in collector.records if r["status"] == "FAIL"),
            "errors": sum(1 for r in collector.records if r["status"] == "ERROR"),
            "results": collector.records,
        }
        with open(args.json_report, "w", encoding="utf-8") as f:
            json.dump(report_data, f, indent=2)
        print(f"\n[+] Wrote JSON report to {args.json_report}")

    exit_code = 0 if (len(collector.failures) == 0 and len(collector.errors) == 0) else 1
    sys.exit(exit_code)


if __name__ == "__main__":
    main()
