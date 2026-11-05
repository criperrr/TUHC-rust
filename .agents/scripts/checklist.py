#!/usr/bin/env python3
"""
Fast AG Kit Validation Checklist for TUHC-rust.
Executes core verification gates in sequence with execution timing and reporting.

Usage:
    python3 .agents/scripts/checklist.py
    python3 .agents/scripts/checklist.py --stop-on-fail
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import time
from pathlib import Path


def run_command(name: str, cmd: list[str], cwd: Path) -> tuple[bool, float, str]:
    start = time.time()
    try:
        proc = subprocess.run(
            cmd,
            cwd=cwd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=180,
        )
        duration = time.time() - start
        return (proc.returncode == 0, duration, proc.stdout)
    except subprocess.TimeoutExpired:
        return (False, time.time() - start, "Timed out after 180s")
    except Exception as e:
        return (False, time.time() - start, str(e))


def main() -> int:
    parser = argparse.ArgumentParser(description="Run the TUHC-rust validation checklist")
    parser.add_argument("project", nargs="?", default=".", help="Project path to validate")
    parser.add_argument("--stop-on-fail", action="store_true", help="Stop after the first failed check")
    parser.add_argument("--verbose", action="store_true", help="Print command outputs on success")
    args = parser.parse_args()

    project = Path(args.project).resolve()
    if not project.is_dir():
        parser.error(f"Project directory does not exist: {project}")

    checks = [
        ("Compilation Check (cargo check)", ["cargo", "check"]),
        ("Code Formatting (cargo fmt)", ["cargo", "fmt", "--check"]),
        ("Static Analysis & Lints (cargo clippy)", ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"]),
        ("Unit & Integration Tests (cargo test)", ["cargo", "test"]),
        ("Python Dual-Track E2E Contracts", ["python3", "tests/e2e_runner.py"]),
        ("Memory Footprint Gatekeeper (<150MB SLA)", ["python3", ".agents/scripts/memory_check.py"]),
    ]

    print("=" * 66)
    print("  TUHC-rust — AG Kit Fast Validation Checklist")
    print(f"  Target: {project}")
    print("=" * 66)

    passed_count = 0
    total_time = 0.0
    failures = []

    for idx, (title, cmd) in enumerate(checks, 1):
        print(f"[{idx}/{len(checks)}] Running: {title} ...", end=" ", flush=True)
        success, duration, output = run_command(title, cmd, project)
        total_time += duration

        if success:
            print(f"✅ PASSED ({duration:.2f}s)")
            passed_count += 1
            if args.verbose:
                print(output)
        else:
            print(f"❌ FAILED ({duration:.2f}s)")
            failures.append((title, output))
            if args.stop_on_fail:
                break

    print("\n" + "=" * 66)
    print(f"  CHECKLIST SUMMARY: {passed_count}/{len(checks)} Passed in {total_time:.2f}s")
    print("=" * 66)

    if failures:
        print("\n--- Failure Details ---")
        for title, output in failures:
            print(f"\n[-] {title}:")
            # print last 15 lines of output
            lines = output.strip().splitlines()
            for line in lines[-15:]:
                print(f"    {line}")
        return 1

    print("\n✨ All quality, correctness, and memory gates are 100% compliant!")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
