#!/usr/bin/env python3
"""
Empirical Challenge & Adversarial Stress Harness for Milestone 1: Backend Core & Asset Server.
Executes rigorous verification of:
1. Large media streaming & Range boundaries (196 MB MP4, SWF, MP3).
2. Malformed Range requests & RFC compliance (HTTP 416).
3. Path sanitization, directory traversal attacks, null-byte injection, and malformed IPC.
4. Memory stability & leak detection under burst request loads (target < 150 MB RSS).
"""

import os
import sys
import time
import json
import socket
import hashlib
import urllib.request
import urllib.error
import urllib.parse
import http.client
import subprocess
import threading
from concurrent.futures import ThreadPoolExecutor, as_completed

ASSET_PACK_DIR = "/home/criper/Downloads/Asset Pack V2"
SERVER_BIN = "/home/criper/teamwork_projects/tuhc_tauri/target/debug/server"
PORT = 45300
BASE_URL = f"http://127.0.0.1:{PORT}"

LARGE_MP4_REL = "storyfiles/hs2/08080/08080.mp4"
LARGE_MP4_PATH = os.path.join(ASSET_PACK_DIR, LARGE_MP4_REL)

SWF_REL = "archive/minutia/006385.swf"
SWF_PATH = os.path.join(ASSET_PACK_DIR, SWF_REL)

MP3_REL = "archive/music/extra/hilary-troiano-commentary.mp3"
MP3_PATH = os.path.join(ASSET_PACK_DIR, MP3_REL)


def get_process_rss_kb(pid: int) -> int:
    """Reads RSS in KB from /proc/<pid>/statm."""
    try:
        with open(f"/proc/{pid}/statm", "r") as f:
            parts = f.read().split()
            pages = int(parts[1])
            page_size_kb = os.sysconf("SC_PAGE_SIZE") // 1024
            return pages * page_size_kb
    except Exception:
        # Fallback to ps
        out = subprocess.check_output(["ps", "-o", "rss=", "-p", str(pid)]).decode().strip()
        return int(out)


class HttpResult:
    def __init__(self, status: int, headers: dict, data: bytes, elapsed_ms: float):
        self.status = status
        self.headers = {k.lower(): v for k, v in headers.items()}
        self.data = data
        self.elapsed_ms = elapsed_ms

    @property
    def text(self) -> str:
        return self.data.decode("utf-8", errors="replace")

    def json(self):
        return json.loads(self.text)

    def header(self, key: str, default=None):
        return self.headers.get(key.lower(), default)


def raw_http_request(method: str, path: str, headers: dict = None, body: bytes = None, timeout: float = 10.0) -> HttpResult:
    conn = http.client.HTTPConnection("127.0.0.1", PORT, timeout=timeout)
    req_headers = headers.copy() if headers else {}
    if body and "Content-Length" not in req_headers:
        req_headers["Content-Length"] = str(len(body))

    start = time.perf_counter()
    try:
        conn.request(method, path, body=body, headers=req_headers)
        resp = conn.getresponse()
        data = resp.read()
        elapsed = (time.perf_counter() - start) * 1000.0
        resp_headers = dict(resp.getheaders())
        return HttpResult(resp.status, resp_headers, data, elapsed)
    finally:
        conn.close()


def read_file_slice(file_path: str, start: int, length: int) -> bytes:
    with open(file_path, "rb") as f:
        f.seek(start)
        return f.read(length)


def run_tests():
    print("=" * 80)
    print("TUHC TAURI 2.0 EMPIRICAL CHALLENGE HARNESS — MILESTONE 1")
    print("=" * 80)

    # 1. Verification of test files
    assert os.path.exists(SERVER_BIN), f"Server binary missing: {SERVER_BIN}"
    assert os.path.exists(LARGE_MP4_PATH), f"Large MP4 missing: {LARGE_MP4_PATH}"
    assert os.path.exists(SWF_PATH), f"SWF missing: {SWF_PATH}"
    assert os.path.exists(MP3_PATH), f"MP3 missing: {MP3_PATH}"

    mp4_size = os.path.getsize(LARGE_MP4_PATH)
    swf_size = os.path.getsize(SWF_PATH)
    mp3_size = os.path.getsize(MP3_PATH)

    print(f"[*] Target Files:")
    print(f"    - MP4: {LARGE_MP4_REL} ({mp4_size:,} bytes / ~{mp4_size/(1024*1024):.2f} MB)")
    print(f"    - SWF: {SWF_REL} ({swf_size:,} bytes)")
    print(f"    - MP3: {MP3_REL} ({mp3_size:,} bytes)")

    # 2. Launch Server Process
    print(f"[*] Launching server on port {PORT}...")
    server_proc = subprocess.Popen(
        [SERVER_BIN, str(PORT)],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    server_pid = server_proc.pid

    # Wait for server to accept connections
    connected = False
    for _ in range(50):
        try:
            s = socket.create_connection(("127.0.0.1", PORT), timeout=0.1)
            s.close()
            connected = True
            break
        except Exception:
            time.sleep(0.1)

    if not connected:
        server_proc.kill()
        out, err = server_proc.communicate()
        print("[-] Server failed to start! stdout:", out.decode(), "stderr:", err.decode())
        sys.exit(1)

    initial_rss = get_process_rss_kb(server_pid)
    print(f"[+] Server started (PID: {server_pid}). Initial RSS: {initial_rss:,} KB (~{initial_rss/1024:.2f} MB)")

    test_records = []

    def record(name: str, passed: bool, details: str = ""):
        test_records.append({"name": name, "passed": passed, "details": details})
        status = "[PASS]" if passed else "[FAIL]"
        print(f"  {status} {name}: {details}")

    try:
        # =====================================================================
        # CATEGORY 1: MEDIA STREAMING & RANGE BOUNDARIES (196 MB MP4, SWF, MP3)
        # =====================================================================
        print("\n" + "=" * 60)
        print("CATEGORY 1: MEDIA STREAMING & RANGE BOUNDARY TESTS")
        print("=" * 60)

        # 1.1 Full Stream 200 OK (Head check and initial bytes)
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}")
        p1 = (res.status == 200 and
              res.header("content-type") == "video/mp4" and
              res.header("content-length") == str(mp4_size) and
              res.header("accept-ranges") == "bytes" and
              len(res.data) == mp4_size and
              res.data[:65536] == read_file_slice(LARGE_MP4_PATH, 0, 65536))
        record("MP4 Full Stream (200 OK, 196 MB exact payload match)", p1,
               f"Status {res.status}, Len {len(res.data):,}, Time {res.elapsed_ms:.1f}ms")

        # 1.2 First chunk: bytes=0-1048575 (1 MB)
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": "bytes=0-1048575"})
        expected_chunk = read_file_slice(LARGE_MP4_PATH, 0, 1048576)
        p2 = (res.status == 206 and
              res.header("content-range") == f"bytes 0-1048575/{mp4_size}" and
              res.header("content-length") == "1048576" and
              res.data == expected_chunk)
        record("MP4 Range 0-1048575 (1 MB slice)", p2,
               f"Status {res.status}, Range {res.header('content-range')}")

        # 1.3 Exact single byte at start: bytes=0-0
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": "bytes=0-0"})
        expected_byte_0 = read_file_slice(LARGE_MP4_PATH, 0, 1)
        p3 = (res.status == 206 and
              res.header("content-range") == f"bytes 0-0/{mp4_size}" and
              res.header("content-length") == "1" and
              res.data == expected_byte_0)
        record("MP4 Range Single Byte Start (bytes=0-0)", p3,
               f"Status {res.status}, Byte: {res.data.hex()}")

        # 1.4 Exact single byte at end: bytes=(size-1)-(size-1)
        last_idx = mp4_size - 1
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": f"bytes={last_idx}-{last_idx}"})
        expected_byte_last = read_file_slice(LARGE_MP4_PATH, last_idx, 1)
        p4 = (res.status == 206 and
              res.header("content-range") == f"bytes {last_idx}-{last_idx}/{mp4_size}" and
              res.header("content-length") == "1" and
              res.data == expected_byte_last)
        record(f"MP4 Range Single Byte End (bytes={last_idx}-{last_idx})", p4,
               f"Status {res.status}, Byte: {res.data.hex()}")

        # 1.5 Middle chunk: offset 100 MB, length 8 KB
        mid_start = 100_000_000
        mid_end = 100_008_191
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": f"bytes={mid_start}-{mid_end}"})
        expected_mid = read_file_slice(LARGE_MP4_PATH, mid_start, 8192)
        p5 = (res.status == 206 and
              res.header("content-range") == f"bytes {mid_start}-{mid_end}/{mp4_size}" and
              res.header("content-length") == "8192" and
              res.data == expected_mid)
        record("MP4 Range Middle Chunk (offset 100 MB, 8 KB)", p5,
               f"Status {res.status}, Range {res.header('content-range')}")

        # 1.6 Open-ended range from offset: bytes=200000000- (to EOF)
        open_start = 200_000_000
        expected_len = mp4_size - open_start
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": f"bytes={open_start}-"})
        expected_tail = read_file_slice(LARGE_MP4_PATH, open_start, expected_len)
        p6 = (res.status == 206 and
              res.header("content-range") == f"bytes {open_start}-{last_idx}/{mp4_size}" and
              res.header("content-length") == str(expected_len) and
              res.data == expected_tail)
        record("MP4 Open Range (bytes=200000000- to EOF)", p6,
               f"Status {res.status}, Len: {len(res.data):,}")

        # 1.7 Suffix range: bytes=-4096 (last 4 KB)
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": "bytes=-4096"})
        expected_suffix_start = mp4_size - 4096
        expected_suffix_bytes = read_file_slice(LARGE_MP4_PATH, expected_suffix_start, 4096)
        p7 = (res.status == 206 and
              res.header("content-range") == f"bytes {expected_suffix_start}-{last_idx}/{mp4_size}" and
              res.header("content-length") == "4096" and
              res.data == expected_suffix_bytes)
        record("MP4 Suffix Range (bytes=-4096)", p7,
               f"Status {res.status}, Range {res.header('content-range')}")

        # 1.8 Suffix range single byte: bytes=-1
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": "bytes=-1"})
        p8 = (res.status == 206 and
              res.header("content-range") == f"bytes {last_idx}-{last_idx}/{mp4_size}" and
              res.header("content-length") == "1" and
              res.data == expected_byte_last)
        record("MP4 Suffix Single Byte (bytes=-1)", p8,
               f"Status {res.status}, Byte: {res.data.hex()}")

        # 1.9 Suffix full file: bytes=-<file_size>
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": f"bytes=-{mp4_size}"})
        p9 = (res.status == 206 and
              res.header("content-range") == f"bytes 0-{last_idx}/{mp4_size}" and
              res.header("content-length") == str(mp4_size))
        record("MP4 Suffix Full File (bytes=-201618473)", p9,
               f"Status {res.status}, Range {res.header('content-range')}")

        # 1.10 HEAD request with Range header
        res = raw_http_request("HEAD", f"/assets/{LARGE_MP4_REL}", headers={"Range": "bytes=0-1023"})
        p10 = (res.status == 206 and
               res.header("content-range") == f"bytes 0-1023/{mp4_size}" and
               res.header("content-length") == "1024" and
               len(res.data) == 0)
        record("MP4 HEAD with Range (bytes=0-1023)", p10,
               f"Status {res.status}, Body Len: {len(res.data)}")

        # 1.11 SWF Media Streaming & Range
        res = raw_http_request("GET", f"/assets/{SWF_REL}")
        p11 = (res.status == 200 and
               res.header("content-type") == "application/x-shockwave-flash" and
               len(res.data) == swf_size)
        record("SWF Full Stream (200 OK)", p11,
               f"Status {res.status}, Content-Type: {res.header('content-type')}")

        res = raw_http_request("GET", f"/assets/{SWF_REL}", headers={"Range": "bytes=0-511"})
        p12 = (res.status == 206 and
               res.header("content-range") == f"bytes 0-511/{swf_size}" and
               len(res.data) == 512 and
               res.data == read_file_slice(SWF_PATH, 0, 512))
        record("SWF Range Request (bytes=0-511)", p12,
               f"Status {res.status}, Range: {res.header('content-range')}")

        # 1.12 MP3 Media Streaming & Range
        res = raw_http_request("GET", f"/assets/{MP3_REL}")
        p13 = (res.status == 200 and
               res.header("content-type") == "audio/mpeg" and
               len(res.data) == mp3_size)
        record("MP3 Full Stream (200 OK)", p13,
               f"Status {res.status}, Content-Type: {res.header('content-type')}")

        res = raw_http_request("GET", f"/assets/{MP3_REL}", headers={"Range": "bytes=1000-1999"})
        p14 = (res.status == 206 and
               res.header("content-range") == f"bytes 1000-1999/{mp3_size}" and
               len(res.data) == 1000 and
               res.data == read_file_slice(MP3_PATH, 1000, 1000))
        record("MP3 Range Request (bytes=1000-1999)", p14,
               f"Status {res.status}, Range: {res.header('content-range')}")

        # 1.13 Concurrent Multi-Stream Verification (10 parallel threads to 196 MB MP4)
        print("\n[*] Running 10 Concurrent Range Streams on 196 MB MP4...")
        concurrent_errors = []

        def worker_stream(idx):
            start = idx * 10_000_000
            end = start + 65535  # 64 KB slice
            res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": f"bytes={start}-{end}"})
            if res.status != 206:
                return f"Thread {idx} got status {res.status}"
            expected = read_file_slice(LARGE_MP4_PATH, start, 65536)
            if res.data != expected:
                return f"Thread {idx} data mismatch"
            return None

        with ThreadPoolExecutor(max_workers=10) as executor:
            futures = [executor.submit(worker_stream, i) for i in range(10)]
            for fut in as_completed(futures):
                err = fut.result()
                if err:
                    concurrent_errors.append(err)

        p15 = len(concurrent_errors) == 0
        record("10 Concurrent Parallel Range Streams to MP4", p15,
               f"Errors: {len(concurrent_errors)} / 10")

        # =====================================================================
        # CATEGORY 2: MALFORMED & OUT-OF-BOUNDS RANGES (HTTP 416 VERIFICATION)
        # =====================================================================
        print("\n" + "=" * 60)
        print("CATEGORY 2: MALFORMED & OUT-OF-BOUNDS RANGE TESTS (416)")
        print("=" * 60)

        # 2.1 Start equals file size (bytes=size-)
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": f"bytes={mp4_size}-"})
        p16 = res.status == 416 and res.header("content-range") == f"bytes */{mp4_size}"
        record("Range Start == file_size (bytes=size-)", p16,
               f"Status {res.status}, Content-Range: {res.header('content-range')}")

        # 2.2 Start exceeds file size
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": f"bytes={mp4_size + 1000}-"})
        p17 = res.status == 416 and res.header("content-range") == f"bytes */{mp4_size}"
        record("Range Start > file_size", p17,
               f"Status {res.status}, Content-Range: {res.header('content-range')}")

        # 2.3 Start and end both beyond EOF
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": "bytes=999999999-999999999"})
        p18 = res.status == 416 and res.header("content-range") == f"bytes */{mp4_size}"
        record("Range Start & End beyond EOF (bytes=999999999-999999999)", p18,
               f"Status {res.status}, Content-Range: {res.header('content-range')}")

        # 2.4 Inverted Range: start > end
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": "bytes=5000-1000"})
        p19 = res.status == 416 and res.header("content-range") == f"bytes */{mp4_size}"
        record("Inverted Range (bytes=5000-1000)", p19,
               f"Status {res.status}, Content-Range: {res.header('content-range')}")

        # 2.5 Inverted Large Range: 150MB - 50MB
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": "bytes=150000000-50000000"})
        p20 = res.status == 416
        record("Inverted Large Range (bytes=150000000-50000000)", p20,
               f"Status {res.status}")

        # 2.6 Suffix of zero length (bytes=-0)
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": "bytes=-0"})
        p21 = res.status == 416
        record("Zero Suffix Range (bytes=-0)", p21,
               f"Status {res.status}")

        # 2.7 Non-numeric characters in Range
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": "bytes=abc-100"})
        p22 = res.status == 416
        record("Non-numeric Start (bytes=abc-100)", p22, f"Status {res.status}")

        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": "bytes=100-xyz"})
        p23 = res.status == 416
        record("Non-numeric End (bytes=100-xyz)", p23, f"Status {res.status}")

        # 2.8 Multiple hyphens
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": "bytes=1-2-3"})
        p24 = res.status == 416
        record("Multiple Hyphens (bytes=1-2-3)", p24, f"Status {res.status}")

        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": "bytes=--500"})
        p25 = res.status == 416
        record("Double Leading Hyphens (bytes=--500)", p25, f"Status {res.status}")

        # 2.9 Invalid unit
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": "items=0-100"})
        p26 = res.status == 416
        record("Invalid Unit (items=0-100)", p26, f"Status {res.status}")

        # 2.10 Empty Range values
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": "bytes="})
        p27 = res.status == 416
        record("Empty Range Value (bytes=)", p27, f"Status {res.status}")

        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": "bytes=-"})
        p28 = res.status == 416
        record("Hyphen Only (bytes=-)", p28, f"Status {res.status}")

        # =====================================================================
        # CATEGORY 3: PATH SANITIZATION, TRAVERSAL ATTACKS & MALFORMED IPC
        # =====================================================================
        print("\n" + "=" * 60)
        print("CATEGORY 3: PATH TRAVERSAL & MALFORMED PAYLOAD ATTACKS")
        print("=" * 60)

        # 3.1 Classic dot-dot traversal
        res = raw_http_request("GET", "/assets/../../etc/passwd")
        p29 = res.status == 404
        record("Directory Traversal (/assets/../../etc/passwd)", p29, f"Status {res.status}")

        # 3.2 Deep dot-dot traversal
        res = raw_http_request("GET", "/assets/../../../../../../etc/shadow")
        p30 = res.status == 404
        record("Deep Traversal (/assets/../../../../../../etc/shadow)", p30, f"Status {res.status}")

        # 3.3 Percent-encoded dot-dot (%2e%2e)
        res = raw_http_request("GET", "/assets/%2e%2e/%2e%2e/etc/passwd")
        p31 = res.status == 404
        record("Percent Encoded Traversal (%2e%2e/%2e%2e)", p31, f"Status {res.status}")

        # 3.4 Percent-encoded slash (%2f)
        res = raw_http_request("GET", "/assets/..%2f..%2fetc/passwd")
        p32 = res.status == 404
        record("Encoded Slash Traversal (..%2f..%2f)", p32, f"Status {res.status}")

        # 3.5 Fully encoded (%2e%2e%2f)
        res = raw_http_request("GET", "/assets/%2e%2e%2f%2e%2e%2fetc/passwd")
        p33 = res.status == 404
        record("Fully Encoded Traversal (%2e%2e%2f%2e%2e%2f)", p33, f"Status {res.status}")

        # 3.6 Double URL-encoded traversal (%252e%252e)
        res = raw_http_request("GET", "/assets/%252e%252e%252fetc/passwd")
        p34 = res.status == 404
        record("Double Encoded Traversal (%252e%252e%252f)", p34, f"Status {res.status}")

        # 3.7 Mid-path traversal
        res = raw_http_request("GET", "/assets/storyfiles/hs2/../../../../etc/passwd")
        p35 = res.status == 404
        record("Mid-Path Traversal (/storyfiles/hs2/../../../../)", p35, f"Status {res.status}")

        # 3.8 Target host config.json outside asset dir
        res = raw_http_request("GET", "/assets/../../.config/unofficial-homestuck-collection/config.json")
        p36 = res.status == 404
        record("Target Config File via Traversal", p36, f"Status {res.status}")

        # 3.9 Null byte injection (%00)
        res = raw_http_request("GET", "/assets/storyfiles/hs2/00001.gif%00")
        p37 = res.status == 404
        record("Null Byte Injection (%00)", p37, f"Status {res.status}")

        res = raw_http_request("GET", "/assets/storyfiles/hs2/00001.gif%00.txt")
        p38 = res.status == 404
        record("Null Byte with Extension (%00.txt)", p38, f"Status {res.status}")

        # 3.10 Backslash traversal (%5c)
        res = raw_http_request("GET", "/assets/..%5c..%5cetc/passwd")
        p39 = res.status == 404
        record("Backslash Traversal (..%5c..%5c)", p39, f"Status {res.status}")

        # 3.11 Double slash normalization check
        res = raw_http_request("GET", "/assets//storyfiles//hs2//00001.gif")
        p40 = res.status == 404
        record("Double Slash Path Rejection (/assets//storyfiles//)", p40, f"Status {res.status}")

        # 3.12 Valid path with query parameters and hash fragments (must succeed)
        res = raw_http_request("GET", "/assets/storyfiles/hs2/00001.gif?v=123&ts=9999#preview")
        p41 = res.status == 200 and res.header("content-type") == "image/gif"
        record("Valid Asset with Query & Fragment (?v=123#preview)", p41, f"Status {res.status}")

        # 3.13 Traversal via IPC get_story_page
        ipc_body = json.dumps({"cmd": "get_story_page", "args": {"page_id": "../../../../etc/passwd"}}).encode()
        res = raw_http_request("POST", "/ipc", headers={"Content-Type": "application/json"}, body=ipc_body)
        p42 = res.status == 404
        record("IPC get_story_page Directory Traversal", p42, f"Status {res.status}")

        # 3.14 IPC with null byte in page_id
        ipc_body = json.dumps({"cmd": "get_story_page", "args": {"page_id": "001901\0"}}).encode()
        res = raw_http_request("POST", "/ipc", headers={"Content-Type": "application/json"}, body=ipc_body)
        p43 = res.status == 404
        record("IPC get_story_page Null Byte", p43, f"Status {res.status}")

        # 3.15 IPC with SQL injection attempt in page_id
        ipc_body = json.dumps({"cmd": "get_story_page", "args": {"page_id": "1' OR '1'='1"}}).encode()
        res = raw_http_request("POST", "/ipc", headers={"Content-Type": "application/json"}, body=ipc_body)
        p44 = res.status == 404
        record("IPC SQLi Payload Injection", p44, f"Status {res.status}")

        # 3.16 IPC with XSS string in page_id
        ipc_body = json.dumps({"cmd": "get_story_page", "args": {"page_id": "<script>alert(1)</script>"}}).encode()
        res = raw_http_request("POST", "/ipc", headers={"Content-Type": "application/json"}, body=ipc_body)
        p45 = res.status == 404
        record("IPC XSS Payload Injection", p45, f"Status {res.status}")

        # 3.17 IPC Malformed Payloads: Empty body
        res = raw_http_request("POST", "/ipc", headers={"Content-Type": "application/json"}, body=b"")
        p46 = res.status == 400
        record("IPC Empty Body (400)", p46, f"Status {res.status}")

        # 3.18 IPC Malformed Payloads: Corrupted JSON
        res = raw_http_request("POST", "/ipc", headers={"Content-Type": "application/json"}, body=b"{bad_json: 123")
        p47 = res.status == 400
        record("IPC Corrupted JSON Body (400)", p47, f"Status {res.status}")

        # 3.19 IPC Missing command
        res = raw_http_request("POST", "/ipc", headers={"Content-Type": "application/json"}, body=b'{"args": {}}')
        p48 = res.status == 400
        record("IPC Missing Command Field (400)", p48, f"Status {res.status}")

        # 3.20 IPC Unknown command
        res = raw_http_request("POST", "/ipc", headers={"Content-Type": "application/json"}, body=b'{"cmd": "hack_system"}')
        p49 = res.status == 400
        record("IPC Unknown Command (400)", p49, f"Status {res.status}")

        # 3.21 HTTP Method Not Allowed
        res = raw_http_request("DELETE", f"/assets/{LARGE_MP4_REL}")
        p50 = res.status == 405
        record("DELETE Method Rejection (405 Method Not Allowed)", p50, f"Status {res.status}")

        # =====================================================================
        # CATEGORY 4: BURST REQUEST LOADS & MEMORY STABILITY
        # =====================================================================
        print("\n" + "=" * 60)
        print("CATEGORY 4: BURST REQUEST LOADS & MEMORY STABILITY")
        print("=" * 60)

        pre_burst_rss = get_process_rss_kb(server_pid)
        print(f"[*] Pre-Burst RSS: {pre_burst_rss:,} KB (~{pre_burst_rss/1024:.2f} MB)")

        total_burst_reqs = 1500
        burst_errors = 0
        peak_rss = pre_burst_rss

        start_time = time.perf_counter()

        def send_burst_req(i):
            nonlocal peak_rss, burst_errors
            # Alternate between:
            # Type A: MP4 Range request (64KB at random offset)
            # Type B: IPC get_story_page request
            # Type C: 416 invalid range request
            # Type D: 404 traversal attack
            try:
                mod = i % 4
                if mod == 0:
                    offset = (i * 1234567) % (mp4_size - 65536)
                    res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": f"bytes={offset}-{offset+65535}"})
                    if res.status != 206:
                        burst_errors += 1
                elif mod == 1:
                    page = 1901 + (i % 200)
                    body = json.dumps({"cmd": "get_story_page", "args": {"page_id": f"{page:06}"}}).encode()
                    res = raw_http_request("POST", "/ipc", headers={"Content-Type": "application/json"}, body=body)
                    if res.status != 200:
                        burst_errors += 1
                elif mod == 2:
                    res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": "bytes=999999999-999999999"})
                    if res.status != 416:
                        burst_errors += 1
                else:
                    res = raw_http_request("GET", "/assets/../../etc/passwd")
                    if res.status != 404:
                        burst_errors += 1
            except Exception as e:
                burst_errors += 1

            if i % 100 == 0:
                current_rss = get_process_rss_kb(server_pid)
                if current_rss > peak_rss:
                    peak_rss = current_rss

        print(f"[*] Firing {total_burst_reqs} burst requests across 20 worker threads...")
        with ThreadPoolExecutor(max_workers=20) as executor:
            list(executor.map(send_burst_req, range(total_burst_reqs)))

        burst_duration = time.perf_counter() - start_time
        rps = total_burst_reqs / burst_duration
        print(f"[+] Burst completed in {burst_duration:.2f}s ({rps:.1f} req/s). Errors: {burst_errors}")

        # Settle period
        time.sleep(2.0)
        post_burst_rss = get_process_rss_kb(server_pid)
        peak_rss = max(peak_rss, post_burst_rss)

        print(f"[*] Memory Profile:")
        print(f"    - Baseline Pre-Burst RSS : {pre_burst_rss:,} KB (~{pre_burst_rss/1024:.2f} MB)")
        print(f"    - Peak Load RSS          : {peak_rss:,} KB (~{peak_rss/1024:.2f} MB)")
        print(f"    - Post-Burst Settled RSS : {post_burst_rss:,} KB (~{post_burst_rss/1024:.2f} MB)")
        print(f"    - Net Growth (Post-Pre)  : {post_burst_rss - pre_burst_rss:,} KB (~{(post_burst_rss - pre_burst_rss)/1024:.2f} MB)")

        # Verify burst integrity
        p51 = burst_errors == 0
        record(f"Burst Request Reliability ({total_burst_reqs} reqs, 0 errors)", p51,
               f"{rps:.1f} req/s, errors: {burst_errors}")

        # Verify memory remains strictly below 150 MB ceiling (153,600 KB)
        memory_ceiling_kb = 150 * 1024
        p52 = post_burst_rss < memory_ceiling_kb and peak_rss < memory_ceiling_kb
        record(f"Memory Within Budget (< 150 MB RSS)", p52,
               f"Peak: {peak_rss/1024:.2f} MB, Post: {post_burst_rss/1024:.2f} MB")

        # Verify post-burst responsiveness
        res = raw_http_request("GET", f"/assets/{LARGE_MP4_REL}", headers={"Range": "bytes=0-100"})
        p53 = res.status == 206
        record("Post-Burst Server Liveness & Responsiveness", p53,
               f"Status {res.status}, Latency {res.elapsed_ms:.2f}ms")

    finally:
        print("\n[*] Terminating test server process...")
        server_proc.terminate()
        try:
            server_proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            server_proc.kill()
        print("[+] Test server terminated.")

    # Summary
    total_tests = len(test_records)
    passed_tests = sum(1 for t in test_records if t["passed"])
    failed_tests = total_tests - passed_tests

    print("\n" + "=" * 80)
    print("EMPIRICAL TEST SUITE SUMMARY")
    print("=" * 80)
    print(f"Total Tests Run: {total_tests}")
    print(f"Passed:          {passed_tests}")
    print(f"Failed:          {failed_tests}")
    print(f"Success Rate:    {passed_tests / total_tests * 100:.1f}%")
    print("=" * 80)

    if failed_tests > 0:
        print("\n[-] FAILED TESTS:")
        for t in test_records:
            if not t["passed"]:
                print(f"    - {t['name']}: {t['details']}")
        return False
    else:
        print("\n[+] ALL EMPIRICAL CHALLENGE TESTS PASSED PERFECTLY!")
        return True


if __name__ == "__main__":
    success = run_tests()
    sys.exit(0 if success else 1)
