#!/usr/bin/env python3
"""
Advanced Stress & Adversarial Hardening Suite for Milestone 1.
Tests:
1. Client disconnection / abort mid-stream on 196 MB MP4 (Broken pipe resilience).
2. File descriptor leak detection (/proc/<pid>/fd count before vs after).
3. Oversized payload rejection (>10 MB body on /ipc).
4. Deeply nested paths (1000 directory segments).
5. High concurrency burst (3000 requests across 30 threads).
"""

import os
import sys
import time
import json
import socket
import subprocess
from concurrent.futures import ThreadPoolExecutor

ASSET_PACK_DIR = "/home/criper/Downloads/Asset Pack V2"
SERVER_BIN = "/home/criper/teamwork_projects/tuhc_tauri/target/debug/server"
PORT = 45310
LARGE_MP4_REL = "storyfiles/hs2/08080/08080.mp4"


def count_open_fds(pid: int) -> int:
    try:
        return len(os.listdir(f"/proc/{pid}/fd"))
    except Exception:
        return -1


def run_advanced_stress():
    print("=" * 80)
    print("ADVANCED ADVERSARIAL STRESS & RESILIENCE SUITE")
    print("=" * 80)

    server_proc = subprocess.Popen(
        [SERVER_BIN, str(PORT)],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    server_pid = server_proc.pid

    for _ in range(50):
        try:
            s = socket.create_connection(("127.0.0.1", PORT), timeout=0.1)
            s.close()
            break
        except Exception:
            time.sleep(0.1)

    initial_fds = count_open_fds(server_pid)
    print(f"[+] Server started (PID {server_pid}). Initial open FDs: {initial_fds}")

    results = []

    try:
        # 1. Client Abort / Mid-Stream Socket Close
        print("\n[*] Test 1: Client Socket Abort Mid-Stream (Simulate browser tab closed while buffering video)...")
        for i in range(20):
            s = socket.create_connection(("127.0.0.1", PORT))
            req = f"GET /assets/{LARGE_MP4_REL} HTTP/1.1\r\nHost: 127.0.0.1:{PORT}\r\nConnection: close\r\n\r\n"
            s.sendall(req.encode())
            # Read only first 4KB then abruptly close socket
            _ = s.recv(4096)
            s.close()

        time.sleep(0.5)
        # Verify server is still completely responsive
        s = socket.create_connection(("127.0.0.1", PORT))
        s.sendall(f"GET /assets/storyfiles/hs2/00001.gif HTTP/1.1\r\nHost: 127.0.0.1:{PORT}\r\nConnection: close\r\n\r\n".encode())
        res = s.recv(4096)
        s.close()
        t1 = b"200 OK" in res
        print(f"  [{'PASS' if t1 else 'FAIL'}] Server survived 20 abrupt client socket aborts mid-stream")
        results.append(("Client Abort Mid-Stream", t1))

        # 2. File Descriptor Leak Detection
        print("\n[*] Test 2: File Descriptor Leak Detection...")
        # Stream 100 range requests
        for i in range(100):
            s = socket.create_connection(("127.0.0.1", PORT))
            s.sendall(f"GET /assets/{LARGE_MP4_REL} HTTP/1.1\r\nHost: 127.0.0.1:{PORT}\r\nRange: bytes=0-1023\r\nConnection: close\r\n\r\n".encode())
            _ = s.recv(4096)
            s.close()

        time.sleep(1.0)
        final_fds = count_open_fds(server_pid)
        print(f"    Initial FDs: {initial_fds}, Post-Stream FDs: {final_fds}")
        # Allow minor delta (e.g. epoll/worker thread reuse within 5 FDs)
        t2 = abs(final_fds - initial_fds) <= 5
        print(f"  [{'PASS' if t2 else 'FAIL'}] File Descriptor count stable (no leak)")
        results.append(("File Descriptor Stability", t2))

        # 3. Oversized IPC Payload (> 10MB)
        print("\n[*] Test 3: Oversized IPC Payload (12 MB body)...")
        oversized_data = b"X" * (12 * 1024 * 1024)
        s = socket.create_connection(("127.0.0.1", PORT))
        header = f"POST /ipc HTTP/1.1\r\nHost: 127.0.0.1:{PORT}\r\nContent-Type: application/json\r\nContent-Length: {len(oversized_data)}\r\nConnection: close\r\n\r\n"
        s.sendall(header.encode() + oversized_data)
        res = s.recv(4096)
        s.close()
        t3 = b"400 Bad Request" in res
        print(f"  [{'PASS' if t3 else 'FAIL'}] Oversized payload rejected with 400 Bad Request")
        results.append(("Oversized Payload Rejection", t3))

        # 4. Deeply Nested Paths (1,000 segments)
        print("\n[*] Test 4: Deeply Nested Path Attack (1,000 segments)...")
        deep_path = "/assets/" + "/".join(["nested"] * 1000) + "/file.txt"
        s = socket.create_connection(("127.0.0.1", PORT))
        s.sendall(f"GET {deep_path} HTTP/1.1\r\nHost: 127.0.0.1:{PORT}\r\nConnection: close\r\n\r\n".encode())
        res = s.recv(4096)
        s.close()
        t4 = b"404 Not Found" in res
        print(f"  [{'PASS' if t4 else 'FAIL'}] Deeply nested path safely rejected with 404")
        results.append(("Deeply Nested Path Defense", t4))

        # 5. Heavy Concurrency Burst (3,000 requests across 30 threads)
        print("\n[*] Test 5: Heavy Concurrency Burst (3,000 requests across 30 threads)...")
        burst_errs = 0

        def hit_server(i):
            nonlocal burst_errs
            try:
                s = socket.create_connection(("127.0.0.1", PORT), timeout=5.0)
                if i % 2 == 0:
                    s.sendall(f"GET /assets/storyfiles/hs2/00001.gif HTTP/1.1\r\nHost: 127.0.0.1:{PORT}\r\nConnection: close\r\n\r\n".encode())
                else:
                    s.sendall(f"GET /assets/{LARGE_MP4_REL} HTTP/1.1\r\nHost: 127.0.0.1:{PORT}\r\nRange: bytes=100-200\r\nConnection: close\r\n\r\n".encode())
                res = s.recv(1024)
                s.close()
                if b"200 OK" not in res and b"206 Partial Content" not in res:
                    burst_errs += 1
            except Exception:
                burst_errs += 1

        start_time = time.perf_counter()
        with ThreadPoolExecutor(max_workers=30) as executor:
            list(executor.map(hit_server, range(3000)))
        dur = time.perf_counter() - start_time
        t5 = burst_errs == 0
        print(f"  [{'PASS' if t5 else 'FAIL'}] 3,000 requests completed in {dur:.2f}s ({3000/dur:.1f} req/s). Errors: {burst_errs}")
        results.append(("Heavy Burst Concurrency", t5))

    finally:
        server_proc.terminate()
        try:
            server_proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            server_proc.kill()

    print("\n" + "=" * 80)
    all_passed = all(r[1] for r in results)
    print(f"Advanced Stress Suite Verdict: {'ALL PASSED' if all_passed else 'FAILURES DETECTED'}")
    print("=" * 80)
    return all_passed


if __name__ == "__main__":
    success = run_advanced_stress()
    sys.exit(0 if success else 1)
