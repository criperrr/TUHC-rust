#!/usr/bin/env python3
"""
Master Verification Script for TUHC-rust.
Executes the full pipeline:
1. cargo check
2. cargo fmt --check
3. cargo clippy -- -D warnings
4. cargo test
5. Python Dual-Track E2E Test Suite (e2e_runner.py)
6. Memory Footprint Gatekeeper (< 150MB SLA)
"""

import subprocess
import sys
import os

def run_step(name, cmd, cwd):
    print("\n" + "=" * 60)
    print(f"  Step: {name}")
    print(f"  Command: {' '.join(cmd)}")
    print("=" * 60)
    res = subprocess.run(cmd, cwd=cwd)
    if res.returncode != 0:
        print(f"\n[-] FAILED: {name} (exit code {res.returncode})")
        sys.exit(res.returncode)
    print(f"[+] PASSED: {name}")

def main():
    base_dir = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

    steps = [
        ("Cargo Check", ["cargo", "check"]),
        ("Cargo Format Check", ["cargo", "fmt", "--check"]),
        ("Cargo Clippy (Lints)", ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"]),
        ("Cargo Unit Tests", ["cargo", "test"]),
        ("Python E2E Test Suite", ["python3", "tests/e2e_runner.py"]),
        ("Memory Footprint Gatekeeper", ["python3", ".agents/scripts/memory_check.py"]),
    ]

    for name, cmd in steps:
        run_step(name, cmd, base_dir)

    print("\n" + "=" * 60)
    print("  ALL VERIFICATION CHECKS PASSED (100% HEALTHY)")
    print("=" * 60)

if __name__ == "__main__":
    main()
