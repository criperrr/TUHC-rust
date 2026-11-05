#!/usr/bin/env python3
"""
Memory Footprint Gatekeeper for TUHC-rust.
Spawns the backend server, exercises key endpoints, and verifies that
process Resident Set Size (RSS) stays strictly under the 150MB SLA ceiling.
"""

import os
import subprocess
import sys
import time
import urllib.error
import urllib.request

try:
    import psutil
except ImportError:
    psutil = None

MEMORY_CEILING_MB = 150.0
MIN_SANITY_RSS_MB = 5.0  # Server indexing data cannot sanely take less than 5MB


def find_server_binary():
    base_dir = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    release_bin = os.path.join(base_dir, "target", "release", "server")
    debug_bin = os.path.join(base_dir, "target", "debug", "server")

    if os.path.isfile(release_bin) and os.access(release_bin, os.X_OK):
        return release_bin, base_dir
    if os.path.isfile(debug_bin) and os.access(debug_bin, os.X_OK):
        return debug_bin, base_dir

    # Build server if not present
    print("[*] Server binary not found, compiling cargo build --bin server...")
    subprocess.run(["cargo", "build", "--bin", "server"], cwd=base_dir, check=True)
    return debug_bin, base_dir


def get_process_rss_mb(pid):
    if psutil:
        try:
            p = psutil.Process(pid)
            return p.memory_info().rss / (1024 * 1024)
        except Exception:
            pass

    # Fallback to /proc/[pid]/status on Linux
    status_file = f"/proc/{pid}/status"
    if os.path.exists(status_file):
        try:
            with open(status_file, "r") as f:
                for line in f:
                    if line.startswith("VmRSS:"):
                        parts = line.split()
                        kb = float(parts[1])
                        return kb / 1024.0
        except Exception:
            pass
    return 0.0


def main():
    print("=" * 60)
    print("  TUHC-rust Memory Footprint Gatekeeper (< 150MB SLA)")
    print("=" * 60)

    server_bin, cwd = find_server_binary()
    print(f"[*] Launching server binary: {server_bin}")

    proc = subprocess.Popen(
        [server_bin, "0"],
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    try:
        port = None
        start_time = time.time()

        # Read server port from stdout
        while time.time() - start_time < 10:
            line = proc.stdout.readline()
            if not line:
                if proc.poll() is not None:
                    break
                time.sleep(0.05)
                continue
            print("    " + line.strip())
            if "listening on http://127.0.0.1:" in line:
                port = int(line.split(":")[-1].strip())
                break

        if not port:
            _, stderr = proc.communicate(timeout=2)
            print(f"[-] Failed to detect server port. Server stderr: {stderr}")
            sys.exit(1)

        print(f"[*] Detected loopback port: {port}")
        time.sleep(0.5)

        # Exercise key endpoints
        endpoints = [
            f"http://127.0.0.1:{port}/ipc/get_app_info",
            f"http://127.0.0.1:{port}/ipc/get_story_page?id=001901",
            f"http://127.0.0.1:{port}/ipc/get_story_range?start=001901&count=5",
            f"http://127.0.0.1:{port}/ipc/get_music_data",
            f"http://127.0.0.1:{port}/ipc/get_available_mods",
        ]

        successful_requests = 0
        for ep in endpoints:
            try:
                req = urllib.request.urlopen(ep, timeout=3)
                data = req.read()
                if len(data) > 0:
                    successful_requests += 1
            except Exception as e:
                print(f"[!] Warning: error querying {ep}: {e}")

        # Check process vitality
        if proc.poll() is not None:
            _, stderr = proc.communicate(timeout=2)
            print(f"[-] Server process died during test execution! Stderr: {stderr}")
            sys.exit(1)

        if successful_requests < len(endpoints):
            print(f"[-] Incomplete endpoint exercising: {successful_requests}/{len(endpoints)} succeeded.")
            sys.exit(1)

        rss_mb = get_process_rss_mb(proc.pid)
        print(f"[*] Measured Server RSS: {rss_mb:.2f} MB")
        print(f"[*] Memory SLA Threshold: {MEMORY_CEILING_MB:.2f} MB")

        # Rigorous assertions: reject zero/unmeasurable readings
        if rss_mb <= 0.0 or rss_mb < MIN_SANITY_RSS_MB:
            print(f"[FAIL] Invalid or unmeasurable memory footprint: {rss_mb:.2f} MB (expected >= {MIN_SANITY_RSS_MB} MB).")
            sys.exit(1)

        if rss_mb > MEMORY_CEILING_MB:
            print(f"[FAIL] Server memory ({rss_mb:.2f} MB) strictly exceeded {MEMORY_CEILING_MB} MB limit!")
            sys.exit(1)

        margin = MEMORY_CEILING_MB - rss_mb
        print(f"[PASS] Server memory is healthy ({rss_mb:.2f} MB). Headroom: {margin:.2f} MB.")
        sys.exit(0)

    finally:
        # Guarantee process termination
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=3)
            except subprocess.TimeoutExpired:
                proc.kill()


if __name__ == "__main__":
    main()
