#!/usr/bin/env python3
"""
Adversarial Stress Test Suite for TUHC Tauri Milestone 1 Backend.
Tests:
1. Non-numeric keys (pony, darkcage, jb2_000000, mc0001, pony2, darkcage2) and boundary lookups via HTTP IPC.
2. Concurrent rapid mod toggles while streaming assets and fetching pages.
3. Concurrent save_config / load_config calls to stress atomic persistence.
"""

import sys
import os
import time
import json
import urllib.request
import urllib.error
import threading
import subprocess

def make_request(url, method="GET", data=None, headers=None):
    if headers is None:
        headers = {}
    req = urllib.request.Request(url, method=method, headers=headers)
    if data is not None:
        if isinstance(data, (dict, list)):
            data = json.dumps(data).encode("utf-8")
            headers["Content-Type"] = "application/json"
        req.data = data
    for k, v in headers.items():
        req.add_header(k, v)
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            return resp.status, resp.read(), dict(resp.headers)
    except urllib.error.HTTPError as e:
        return e.code, e.read(), dict(e.headers)
    except Exception as e:
        return 0, str(e).encode(), {}

def test_non_numeric_and_boundary_keys(base_url):
    print("\n--- [1] Testing Non-Numeric Keys & Boundary Lookups via HTTP IPC ---")
    keys_to_test = [
        ("pony", 200, "[S] Ride.", "/storyfiles/hs2/pony/pony.swf"),
        ("darkcage", 200, "[S] Cage: Reveal plan.", "/storyfiles/hs2/darkcage/darkcage.swf"),
        ("jb2_000000", 200, "Enjoy restful slumber.", "/storyfiles/jb2/YOUWIN.gif"),
        ("mc0001", 200, "??????", "/storyfiles/mc/00000.gif"),
        ("pony2", 200, None, None),
        ("darkcage2", 200, None, None),
        ("000002", 200, "Start Jailbreak Adventure", None),
        ("2", 200, "Start Jailbreak Adventure", None),
        ("010030", 200, "[S] ==>", None),
        ("10030", 200, "[S] ==>", None),
        ("000000", 404, None, None),
        ("000001", 404, None, None),
        ("010031", 404, None, None),
        ("10031", 404, None, None),
        ("999999", 404, None, None),
        ("-1", 404, None, None),
        ("", 404, None, None),
        ("   ", 404, None, None),
    ]

    for key, expected_status, expected_title, expected_media in keys_to_test:
        # Test via GET /ipc/get_story_page?page_id=...
        url = f"{base_url}/ipc/get_story_page?page_id={urllib.parse.quote(key)}"
        status, body, _ = make_request(url)
        assert status == expected_status, f"Key '{key}': expected status {expected_status}, got {status}: {body.decode(errors='replace')}"
        if expected_status == 200:
            parsed = json.loads(body)
            assert parsed.get("pageId") == (key if not key.isdigit() else f"{int(key):06}"), f"Key '{key}': mismatch pageId {parsed.get('pageId')}"
            if expected_title:
                assert parsed.get("title") == expected_title, f"Key '{key}': expected title '{expected_title}', got '{parsed.get('title')}'"
            if expected_media:
                assert expected_media in parsed.get("media", []), f"Key '{key}': expected media '{expected_media}', got {parsed.get('media')}"

        # Test via POST /ipc
        post_payload = {"cmd": "get_story_page", "args": {"page_id": key}}
        status_post, body_post, _ = make_request(f"{base_url}/ipc", method="POST", data=post_payload)
        assert status_post == expected_status, f"POST Key '{key}': expected status {expected_status}, got {status_post}"

    print("  [PASS] All non-numeric and boundary page lookups verified via HTTP IPC.")

def test_boundary_ranges(base_url):
    print("\n--- [2] Testing Boundary Story Ranges via HTTP IPC ---")
    ranges_to_test = [
        (0, 5, 5, "000002"),
        (2, 3, 3, "000002"),
        (10030, 10, 7, "010030"), # 010030 + 6 non-numeric keys
        (10031, 5, 0, None),      # beyond max page
        (999999, 10, 0, None),    # way beyond total
        (1901, 0, 0, None),       # count 0
    ]

    for start, count, expected_len, first_page_id in ranges_to_test:
        # POST with numeric integer args
        post_payload = {
            "cmd": "get_story_range",
            "args": {"start": start, "count": count}
        }
        status, body, _ = make_request(f"{base_url}/ipc", method="POST", data=post_payload)
        assert status == 200, f"Range ({start}, {count}) failed with status {status}"
        parsed = json.loads(body)
        assert len(parsed) == expected_len, f"Range ({start}, {count}): expected length {expected_len}, got {len(parsed)}"
        if first_page_id and expected_len > 0:
            assert parsed[0].get("pageId") == first_page_id, f"Range ({start}, {count}): expected first page {first_page_id}, got {parsed[0].get('pageId')}"

    # Also test GET behavior and verify if GET query params fail to parse string numbers
    st_get, body_get, _ = make_request(f"{base_url}/ipc/get_story_range?start=0&count=5")
    parsed_get = json.loads(body_get)
    if len(parsed_get) != 5:
        print(f"  [DISCOVERY] GET /ipc/get_story_range query params are parsed as strings and ignored by v.as_u64(), falling back to default 10 pages at 1901 (got {len(parsed_get)} pages at {parsed_get[0].get('pageId')}).")

    print("  [PASS] Boundary range POST queries verified.")

def test_concurrent_mod_toggles(base_url):
    print("\n--- [3] Stress Testing Concurrent Mod Toggles vs Page & Asset Fetches ---")
    stop_event = threading.Event()
    read_counts = {"pages": 0, "assets": 0, "errors": 0}
    toggle_counts = {"toggles": 0, "errors": 0}

    def reader_worker():
        while not stop_event.is_set():
            # Fetch page 001901
            st, body, _ = make_request(f"{base_url}/ipc/get_story_page?page_id=001901")
            if st == 200:
                p = json.loads(body)
                if "Homestuck" in p.get("title", ""):
                    read_counts["pages"] += 1
                else:
                    read_counts["errors"] += 1
            else:
                read_counts["errors"] += 1

            # Fetch asset 00002.gif (overridden by PT-BR when enabled)
            st_asset, body_asset, _ = make_request(f"{base_url}/assets/storyfiles/hs2/00002.gif")
            if st_asset == 200 and len(body_asset) in (25043, 191716):
                read_counts["assets"] += 1
            else:
                read_counts["errors"] += 1
            time.sleep(0.005)

    def toggler_worker(worker_id):
        enabled = (worker_id % 2 == 0)
        for _ in range(30):
            enabled = not enabled
            payload = {
                "cmd": "toggle_mod",
                "args": {"mod_id": "Homestuck PT-BR", "enabled": enabled}
            }
            st, body, _ = make_request(f"{base_url}/ipc", method="POST", data=payload)
            if st == 200:
                toggle_counts["toggles"] += 1
            else:
                print(f"  [WARN] Toggle error: status={st}, body={body.decode(errors='replace')}")
                toggle_counts["errors"] += 1
            time.sleep(0.01)

    threads = []
    for _ in range(4):
        t = threading.Thread(target=reader_worker)
        threads.append(t)
        t.start()

    togglers = []
    for wid in range(2):
        t = threading.Thread(target=toggler_worker, args=(wid,))
        togglers.append(t)
        t.start()

    for t in togglers:
        t.join()

    stop_event.set()
    for t in threads:
        t.join()

    print(f"  Results: toggles={toggle_counts['toggles']}, toggle_errs={toggle_counts['errors']}, pages_read={read_counts['pages']}, assets_read={read_counts['assets']}, read_errs={read_counts['errors']}")
    assert read_counts["errors"] == 0, f"Read errors occurred during concurrent mod toggles: {read_counts['errors']}"

def test_concurrent_config_persistence(base_url):
    print("\n--- [4] Stress Testing Concurrent save_config via HTTP IPC ---")
    st, body, _ = make_request(f"{base_url}/ipc/load_config")
    assert st == 200, f"Failed to load initial config: {st}"
    base_config = json.loads(body)

    results = {"success": 0, "errors": 0, "error_details": []}

    def save_worker(worker_id):
        for i in range(25):
            cfg_copy = json.loads(json.dumps(base_config))
            cfg_copy["tabData"]["activeTabKey"] = f"tab_{worker_id}_{i}"
            cfg_copy["settings"]["smoothScrolling"] = (i % 2 == 0)

            payload = {
                "cmd": "save_config",
                "args": cfg_copy
            }
            st, res_body, _ = make_request(f"{base_url}/ipc", method="POST", data=payload)
            if st == 200:
                results["success"] += 1
            else:
                err_msg = res_body.decode(errors='replace')
                results["errors"] += 1
                results["error_details"].append((worker_id, i, st, err_msg))
            time.sleep(0.005)

    threads = [threading.Thread(target=save_worker, args=(wid,)) for wid in range(4)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    print(f"  Concurrent save_config results: success={results['success']}, errors={results['errors']}")
    if results["errors"] > 0:
        print(f"  First 5 error samples:")
        for sample in results["error_details"][:5]:
            print(f"    worker {sample[0]} iter {sample[1]}: status {sample[2]} -> {sample[3]}")
    return results

if __name__ == "__main__":
    if len(sys.argv) > 1:
        base_url = sys.argv[1].rstrip("/")
    else:
        base_url = "http://127.0.0.1:45100"

    print(f"Running adversarial stress tests against: {base_url}")
    test_non_numeric_and_boundary_keys(base_url)
    test_boundary_ranges(base_url)
    test_concurrent_mod_toggles(base_url)
    cfg_results = test_concurrent_config_persistence(base_url)
    if cfg_results["errors"] > 0:
        print("\n[VULNERABILITY CONFIRMED] Concurrent save_config failed due to non-atomic / un-isolated temporary file handling!")
        sys.exit(2)
    else:
        print("\nAll adversarial tests completed.")
