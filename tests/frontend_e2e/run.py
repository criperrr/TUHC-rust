#!/usr/bin/env python3
"""
Master Launcher for TUHC-rust Frontend Visual & Subsystem E2E Test Suite.

Automates:
1. Spawning the embedded Axum loopback server on an ephemeral loopback port.
2. Executing the Playwright frontend test runner across all core subsystems.
3. Generating screenshots in tests/frontend_e2e/screenshots/.
4. Logging all browser console, error, and network events into tests/frontend_e2e/logs/.
5. Printing an ASCII summary table of test results and diagnostic metrics.
"""

import os
import sys
import time
import json
import socket
import argparse
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
FRONTEND_E2E_DIR = REPO_ROOT / "tests" / "frontend_e2e"
RUNNER_JS = FRONTEND_E2E_DIR / "e2e_runner.js"
REPORT_JSON = FRONTEND_E2E_DIR / "reports" / "summary.json"
LOG_FILE = FRONTEND_E2E_DIR / "logs" / "frontend_e2e.log"
SERVER_BIN = REPO_ROOT / "target" / "debug" / "server"
SERVER_LOG = FRONTEND_E2E_DIR / "logs" / "server.log"


def get_free_port():
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def ensure_server_binary():
    if not SERVER_BIN.exists():
        print("[*] Server binary not found. Compiling 'server' binary with cargo...")
        cmd = ["cargo", "build", "--bin", "server"]
        res = subprocess.run(cmd, cwd=REPO_ROOT)
        if res.returncode != 0:
            print("[!] Failed to build server binary via cargo.", file=sys.stderr)
            sys.exit(1)


def main():
    parser = argparse.ArgumentParser(description="TUHC Frontend Visual & Subsystem E2E Runner")
    parser.add_argument("--port", type=int, default=0, help="Axum server port (0 for auto)")
    parser.add_argument("--headed", action="store_true", help="Run browser in headed mode (visible window)")
    parser.add_argument("--browser", type=str, default="chromium", choices=["chromium", "firefox"], help="Browser engine")
    parser.add_argument("--server-url", type=str, default="", help="Connect to an already running server URL")
    args = parser.parse_args()

    server_process = None
    server_log_fp = None
    base_url = args.server_url

    # Ensure output directories exist upfront
    (FRONTEND_E2E_DIR / "logs").mkdir(parents=True, exist_ok=True)
    (FRONTEND_E2E_DIR / "screenshots").mkdir(parents=True, exist_ok=True)
    (FRONTEND_E2E_DIR / "reports").mkdir(parents=True, exist_ok=True)

    try:
        if not base_url:
            ensure_server_binary()
            port = args.port if args.port > 0 else get_free_port()
            print(f"[*] Spawning TUHC Axum backend server on 127.0.0.1:{port}...")
            server_log_fp = open(SERVER_LOG, "w", encoding="utf-8")
            server_process = subprocess.Popen(
                [str(SERVER_BIN), str(port)],
                cwd=REPO_ROOT,
                stdout=server_log_fp,
                stderr=subprocess.STDOUT,
                text=True
            )
            time.sleep(1.2)
            base_url = f"http://127.0.0.1:{port}"

            # Verify server is listening
            try:
                import urllib.request
                urllib.request.urlopen(f"{base_url}/index.html", timeout=3)
                print(f"[✓] Backend server healthy and responding at {base_url}")
            except Exception as e:
                print(f"[!] Server health check failed on {base_url}: {e}", file=sys.stderr)
                sys.exit(1)

        # Build runner command
        headless_val = "false" if args.headed else "true"
        node_cmd = [
            "node",
            str(RUNNER_JS),
            f"--url={base_url}",
            f"--headless={headless_val}",
            f"--browser={args.browser}"
        ]

        print(f"[*] Executing Playwright E2E suite via {args.browser.capitalize()}...")
        env = os.environ.copy()
        test_run = subprocess.run(node_cmd, cwd=FRONTEND_E2E_DIR, env=env)

        # Parse and display summary
        if REPORT_JSON.exists():
            with open(REPORT_JSON, "r", encoding="utf-8") as f:
                report = json.load(f)

            stats = report.get("stats", {})
            tests = report.get("tests", [])
            shots = report.get("screenshots", [])

            print("\n" + "=" * 70)
            print("  TUHC-rust Frontend Visual & Subsystem E2E Test Suite Summary")
            print("=" * 70)
            print(f"  Target URL:          {report.get('baseUrl')}")
            print(f"  Browser Engine:      {report.get('browser')}")
            print(f"  Total Duration:      {report.get('durationMs', 0) / 1000:.2f}s")
            print(f"  Tests Passed:        {stats.get('passed')}/{stats.get('total')}")
            print(f"  Screenshots Saved:   {stats.get('screenshots')}")
            print(f"  Console Errors:      {stats.get('consoleErrors', 0)}")
            print("-" * 70)

            print(f"  {'Status':<8} | {'Duration':<10} | {'Test Name'}")
            print("-" * 70)
            for t in tests:
                status_icon = "✓ PASS" if t.get("status") == "PASS" else "✗ FAIL"
                dur = f"{t.get('durationMs', 0)}ms"
                print(f"  {status_icon:<8} | {dur:<10} | {t.get('name')}")

            print("-" * 70)
            print("  Captured Screenshots (in tests/frontend_e2e/screenshots/):")
            for s in shots:
                kb = s.get("sizeBytes", 0) / 1024
                print(f"    - {s.get('filename')} ({kb:.1f} KB) -> {s.get('description')}")

            print("-" * 70)
            print(f"  Detailed Log File:   {LOG_FILE}")
            print(f"  Machine JSON Report: {REPORT_JSON}")
            print("=" * 70 + "\n")

        sys.exit(test_run.returncode)

    finally:
        if server_process:
            print("[*] Stopping Axum backend server...")
            server_process.terminate()
            try:
                server_process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                server_process.kill()
        if server_log_fp:
            try:
                server_log_fp.close()
            except Exception:
                pass


if __name__ == "__main__":
    main()
