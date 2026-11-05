"""
Common test harness, authoritative oracles, HTTP client, and base test case for TUHC Tauri E2E.
"""

import os
import sys
import json
import urllib.request
import urllib.error
import urllib.parse
import http.client
import subprocess
import unittest

# Authoritative Paths
ASSET_PACK_DIR = "/home/criper/Downloads/Asset Pack V2"
CONFIG_PATH = "/home/criper/.config/unofficial-homestuck-collection/config.json"
REPO_DIR = "/tmp/tuhc-repo"
FLASH_PROPS_PATH = os.path.join(REPO_DIR, "src/js/flashProps.js")
PTBR_MOD_DIR = os.path.join(ASSET_PACK_DIR, "mods/Homestuck PT-BR")
PTBR_TRANSLATION_PATH = os.path.join(PTBR_MOD_DIR, "translation2.json")
MSPA_DATA_PATH = os.path.join(ASSET_PACK_DIR, "archive/data/mspa.json")
MUSIC_DATA_PATH = os.path.join(ASSET_PACK_DIR, "archive/data/music.json")

# Global test configuration
TEST_CONFIG = {
    "server_url": os.environ.get("TUHC_TEST_URL", "http://127.0.0.1:45100"),
    "use_live": os.environ.get("TUHC_TEST_LIVE", "0") == "1",
}


def load_oracle_mspa():
    """Load authoritative mspa.json data directly from Asset Pack V2."""
    if not os.path.exists(MSPA_DATA_PATH):
        raise FileNotFoundError(f"Authoritative mspa.json missing at {MSPA_DATA_PATH}")
    with open(MSPA_DATA_PATH, "r", encoding="utf-8") as f:
        return json.load(f)


def load_oracle_music():
    """Load authoritative music.json data directly from Asset Pack V2."""
    if not os.path.exists(MUSIC_DATA_PATH):
        raise FileNotFoundError(f"Authoritative music.json missing at {MUSIC_DATA_PATH}")
    with open(MUSIC_DATA_PATH, "r", encoding="utf-8") as f:
        return json.load(f)


def load_oracle_ptbr_translation():
    """Load authoritative PT-BR translation2.json directly from mod directory."""
    if not os.path.exists(PTBR_TRANSLATION_PATH):
        raise FileNotFoundError(f"Authoritative translation2.json missing at {PTBR_TRANSLATION_PATH}")
    with open(PTBR_TRANSLATION_PATH, "r", encoding="utf-8") as f:
        return json.load(f)


def load_oracle_flash_props():
    """Load authoritative flashProps.js metadata (audioDelay, cropHeight, etc.)."""
    if os.path.exists(FLASH_PROPS_PATH):
        try:
            res = subprocess.run(
                ["node", "-e", f'console.log(JSON.stringify(require("{FLASH_PROPS_PATH}")))'],
                capture_output=True,
                text=True,
                check=True,
                timeout=5,
            )
            return json.loads(res.stdout)
        except Exception:
            pass

    # Fallback to key known entries if node execution fails
    return {
        "audioDelay": {
            "00980_1": 1500,
            "01070": 1500,
            "01801": -100,
            "02786": -200,
            "04106_1": -50,
            "04106_2": 575,
            "04106_3": 1550,
            "04106_4": 600,
            "04106_5": 1200,
        },
        "cropHeight": {
            "04106": {
                "0": 650,
                "1": 450,
            }
        },
    }


def load_oracle_config():
    """Load reference user config."""
    if not os.path.exists(CONFIG_PATH):
        raise FileNotFoundError(f"Authoritative config.json missing at {CONFIG_PATH}")
    with open(CONFIG_PATH, "r", encoding="utf-8") as f:
        return json.load(f)


def evaluate_ptbr_log_button_text(content: str, log_hidden: bool = True) -> str:
    """
    Pure reference evaluation of the PT-BR vueHooks logButtonText logic:
    let text = this.content.match(/\\|(.*?)\\|/)[1]
    const state = (this.logHidden) ? 'Abrir ' : 'Fechar '
    if (/P4SSWORD H1NT/i.test(text)) return text
    else if (/trkstrlog/i.test(text)) text = 'tricksterlog'
    else if (/sriousbiz/i.test(text)) text = 'serious business'
    return state + text.toLowerCase()
    """
    import re
    match = re.search(r"\|(.*?)\|", content)
    if not match:
        return "Abrir pesterlog" if log_hidden else "Fechar pesterlog"
    text = match.group(1)
    state = "Abrir " if log_hidden else "Fechar "
    if re.search(r"P4SSWORD H1NT", text, re.IGNORECASE):
        return text
    elif re.search(r"trkstrlog", text, re.IGNORECASE):
        text = "tricksterlog"
    elif re.search(r"sriousbiz", text, re.IGNORECASE):
        text = "serious business"
    return state + text.lower()


class TuhcResponse:
    """Wrapper around HTTP response for test assertions."""

    def __init__(self, status: int, headers: dict, data: bytes):
        self.status = status
        self.headers = {k.lower(): v for k, v in headers.items()}
        self.data = data

    @property
    def text(self) -> str:
        return self.data.decode("utf-8", errors="replace")

    def json(self):
        return json.loads(self.text)

    def get_header(self, name: str, default=None):
        return self.headers.get(name.lower(), default)


class TuhcClient:
    """Opaque-box HTTP & IPC client for communicating with the Tauri backend."""

    def __init__(self, base_url: str):
        self.base_url = base_url.rstrip("/")

    def request(self, method: str, path: str, headers: dict = None, body: bytes = None) -> TuhcResponse:
        url = f"{self.base_url}/{path.lstrip('/')}"
        parsed = urllib.parse.urlparse(url)
        conn = http.client.HTTPConnection(parsed.netloc, timeout=10)

        req_headers = headers.copy() if headers else {}
        if body and "Content-Length" not in req_headers:
            req_headers["Content-Length"] = str(len(body))

        try:
            req_path = parsed.path
            if parsed.query:
                req_path += f"?{parsed.query}"
            conn.request(method, req_path, body=body, headers=req_headers)
            resp = conn.getresponse()
            data = resp.read()
            resp_headers = dict(resp.getheaders())
            return TuhcResponse(resp.status, resp_headers, data)
        finally:
            conn.close()

    def get(self, path: str, headers: dict = None) -> TuhcResponse:
        return self.request("GET", path, headers=headers)

    def head(self, path: str, headers: dict = None) -> TuhcResponse:
        return self.request("HEAD", path, headers=headers)

    def post(self, path: str, body: bytes = None, json_payload: dict = None, headers: dict = None) -> TuhcResponse:
        req_headers = headers.copy() if headers else {}
        req_body = body
        if json_payload is not None:
            req_body = json.dumps(json_payload).encode("utf-8")
            if "Content-Type" not in req_headers:
                req_headers["Content-Type"] = "application/json"
        return self.request("POST", path, headers=req_headers, body=req_body)

    def ipc(self, command: str, payload: dict = None) -> TuhcResponse:
        """Issue an IPC command to /ipc endpoint."""
        data = {
            "cmd": command,
            "args": payload or {},
            "command": command,
            "payload": payload or {},
        }
        return self.post("/ipc", json_payload=data)


class TuhcTestCase(unittest.TestCase):
    """Base class for all TUHC E2E test cases."""

    @classmethod
    def setUpClass(cls):
        cls.server_url = TEST_CONFIG["server_url"]
        cls.client = TuhcClient(cls.server_url)

    def assertStatus(self, response: TuhcResponse, expected: int, msg: str = None):
        self.assertEqual(
            response.status,
            expected,
            msg or f"Expected status {expected}, got {response.status}. Body: {response.text[:200]}",
        )

    def assertHeader(self, response: TuhcResponse, header: str, expected: str, msg: str = None):
        actual = response.get_header(header)
        self.assertIsNotNone(actual, f"Header '{header}' was not found in response headers: {response.headers}")
        self.assertEqual(
            actual.lower(),
            expected.lower(),
            msg or f"Header '{header}': expected '{expected}', got '{actual}'",
        )

    def assertHeaderContains(self, response: TuhcResponse, header: str, substring: str, msg: str = None):
        actual = response.get_header(header)
        self.assertIsNotNone(actual, f"Header '{header}' was not found in response headers")
        self.assertIn(
            substring.lower(),
            actual.lower(),
            msg or f"Header '{header}': expected '{substring}' in '{actual}'",
        )
