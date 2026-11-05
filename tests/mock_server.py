"""
Reference contract server for TUHC Tauri 2.0.
Implements the exact HTTP asset server, VFS overlay, and IPC contracts specified in PROJECT.md.
"""

import os
import sys
import json
import mimetypes
import http.server
import socketserver
import threading
import urllib.parse
from tests.common import (
    ASSET_PACK_DIR,
    PTBR_MOD_DIR,
    PTBR_TRANSLATION_PATH,
    MSPA_DATA_PATH,
    MUSIC_DATA_PATH,
    CONFIG_PATH,
)

mimetypes.add_type("application/wasm", ".wasm")
mimetypes.add_type("application/x-shockwave-flash", ".swf")
mimetypes.add_type("audio/mpeg", ".mp3")


class TuhcContractHandler(http.server.BaseHTTPRequestHandler):
    """HTTP request handler implementing TUHC Tauri 2.0 backend contracts."""

    # Protocol version
    protocol_version = "HTTP/1.1"

    def log_message(self, format, *args):
        # Silence verbose request logs during tests
        pass

    def send_cors_headers(self):
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, HEAD, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type, Range")

    def do_OPTIONS(self):
        self.send_response(204)
        self.send_cors_headers()
        self.end_headers()

    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path

        if path.startswith("/assets/"):
            rel_asset_path = path[len("/assets/"):]
            self.serve_asset(rel_asset_path)
            return

        if path == "/" or path == "/index.html":
            self.serve_html_shell()
            return

        if path == "/styles/main.css":
            css_data = b"body { font-family: 'Courier New', monospace; background: #000; color: #fff; }"
            self.send_response(200)
            self.send_header("Content-Type", "text/css")
            self.send_header("Content-Length", str(len(css_data)))
            self.send_cors_headers()
            self.end_headers()
            self.wfile.write(css_data)
            return

        if path.startswith("/ipc/"):
            self.handle_rest_ipc(path[len("/ipc/"):], parsed.query)
            return

        self.send_error_response(404, "Not Found")

    def do_HEAD(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path

        if path.startswith("/assets/"):
            rel_asset_path = path[len("/assets/"):]
            self.serve_asset(rel_asset_path, head_only=True)
            return

        self.send_error_response(404, "Not Found")

    def do_POST(self):
        parsed = urllib.parse.urlparse(self.path)
        if parsed.path == "/ipc":
            self.handle_ipc_post()
            return
        self.send_error_response(404, "Not Found")

    def serve_html_shell(self):
        content = (
            b"<!DOCTYPE html><html><head><title>The Unofficial Homestuck Collection</title></head>"
            b"<body><div id='app'>TUHC Webview Shell</div></body></html>"
        )
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(content)))
        self.send_cors_headers()
        self.end_headers()
        self.wfile.write(content)

    def serve_asset(self, rel_path: str, head_only: bool = False):
        # 1. Traversal and security check
        raw_unescaped = urllib.parse.unquote(rel_path)
        if "\0" in raw_unescaped or not rel_path or ".." in raw_unescaped or raw_unescaped.startswith("/"):
            self.send_error_response(404, "Invalid asset path")
            return

        # 2. VFS resolution: check if mod overlay takes precedence
        file_path = None
        state = self.server.app_state

        if "Homestuck PT-BR" in state.get("active_mods", []):
            mod_candidate = os.path.join(PTBR_MOD_DIR, "Assets", raw_unescaped)
            if os.path.isfile(mod_candidate):
                file_path = mod_candidate

        if not file_path:
            base_candidate = os.path.join(ASSET_PACK_DIR, raw_unescaped)
            if os.path.isfile(base_candidate):
                file_path = base_candidate

        if not file_path or not os.path.exists(file_path):
            self.send_error_response(404, "Asset Not Found")
            return

        # 3. Determine MIME type
        ext = os.path.splitext(file_path)[1].lower()
        content_type = mimetypes.guess_type(file_path)[0]
        if ext == ".wasm":
            content_type = "application/wasm"
        elif ext == ".swf":
            content_type = "application/x-shockwave-flash"
        elif ext == ".mp3":
            content_type = "audio/mpeg"
        elif not content_type:
            content_type = "application/octet-stream"

        file_size = os.path.getsize(file_path)

        # 4. Handle Range header
        range_header = self.headers.get("Range")
        if range_header:
            if not range_header.startswith("bytes="):
                self.send_error_response(416, "Invalid Range unit")
                return
            range_val = range_header[len("bytes="):].strip()
            if not range_val or "-" not in range_val or range_val.count("-") > 1:
                self.send_error_response(416, "Malformed Range specification")
                return

            parts = range_val.split("-")
            start_str, end_str = parts[0], parts[1]

            try:
                if start_str and end_str:
                    start = int(start_str)
                    end = int(end_str)
                elif start_str:
                    start = int(start_str)
                    end = file_size - 1
                elif end_str:
                    suffix = int(end_str)
                    start = max(0, file_size - suffix)
                    end = file_size - 1
                else:
                    self.send_error_response(416, "Malformed Range values")
                    return
            except ValueError:
                self.send_error_response(416, "Non-numeric Range values")
                return

            # Range bounds check
            if start > end or start >= file_size or end >= file_size or start < 0:
                self.send_error_response(416, "Range Not Satisfiable")
                return

            length = end - start + 1
            self.send_response(206)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(length))
            self.send_header("Content-Range", f"bytes {start}-{end}/{file_size}")
            self.send_header("Accept-Ranges", "bytes")
            self.send_cors_headers()
            self.end_headers()

            if not head_only:
                with open(file_path, "rb") as f:
                    f.seek(start)
                    self.wfile.write(f.read(length))
            return

        # 5. Full file response (200 OK)
        self.send_response(200)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(file_size))
        self.send_header("Accept-Ranges", "bytes")
        self.send_cors_headers()
        self.end_headers()

        if not head_only:
            with open(file_path, "rb") as f:
                while chunk := f.read(65536):
                    self.wfile.write(chunk)

    def handle_ipc_post(self):
        content_length = self.headers.get("Content-Length")
        if not content_length or int(content_length) == 0:
            self.send_error_response(400, "Empty request body")
            return

        try:
            body = self.rfile.read(int(content_length))
            payload = json.loads(body.decode("utf-8"))
        except Exception:
            self.send_error_response(400, "Invalid JSON payload")
            return

        command = payload.get("cmd") or payload.get("command")
        args = payload.get("args") or payload.get("payload") or {}

        if not command:
            self.send_error_response(400, "Missing IPC command")
            return

        self.dispatch_ipc(command, args)

    def handle_rest_ipc(self, command: str, query: str):
        query_params = urllib.parse.parse_qs(query)
        args = {k: v[0] for k, v in query_params.items()}
        self.dispatch_ipc(command, args)

    def dispatch_ipc(self, command: str, args: dict):
        state = self.server.app_state

        if command == "get_app_info":
            res = {
                "version": "2.0.0",
                "asset_dir": ASSET_PACK_DIR,
                "asset_port": self.server.server_address[1],
            }
            self.send_json_response(200, res)
            return

        elif command == "get_story_page":
            page_id = str(args.get("page_id") or args.get("id") or "").strip()
            if not page_id or not page_id.isdigit():
                self.send_json_response(404, {"error": f"Invalid page ID '{page_id}'"})
                return

            page_id_norm = page_id.zfill(6)
            story = state["mspa"].get("story", {})
            page = story.get(page_id_norm)
            if not page:
                self.send_json_response(404, {"error": f"Page '{page_id}' not found"})
                return

            # Make copy to avoid mutating source archive
            result_page = dict(page)

            # Apply PT-BR translation overlay if enabled
            if "Homestuck PT-BR" in state.get("active_mods", []):
                translation = state.get("ptbr_translation", {}).get(page_id_norm)
                if translation:
                    result_page.update(translation)

            self.send_json_response(200, result_page)
            return

        elif command == "get_story_range":
            try:
                start = int(args.get("start", 1901))
                count = int(args.get("count", 10))
            except (ValueError, TypeError):
                self.send_json_response(400, {"error": "Invalid range parameters"})
                return

            story = state["mspa"].get("story", {})
            summaries = []
            for i in range(start, start + count):
                pid = str(i).zfill(6)
                if pid in story:
                    p = story[pid]
                    summaries.append({
                        "pageId": pid,
                        "title": p.get("title", ""),
                        "theme": p.get("theme", "default"),
                        "media": p.get("media", []),
                    })
            self.send_json_response(200, summaries)
            return

        elif command == "get_music_data":
            self.send_json_response(200, state["music"])
            return

        elif command == "load_config":
            self.send_json_response(200, state["config"])
            return

        elif command == "save_config":
            new_config = args.get("config") or args
            if not isinstance(new_config, dict) or "settings" not in new_config:
                self.send_json_response(400, {"error": "Invalid config schema"})
                return
            state["config"].update(new_config)
            # Sync modListEnabled with active_mods
            enabled_mods = state["config"]["settings"].get("modListEnabled", [])
            state["active_mods"] = list(enabled_mods)
            self.send_json_response(200, {"status": "ok", "saved": True})
            return

        elif command == "get_available_mods":
            mods = [
                {
                    "id": "Homestuck PT-BR",
                    "title": "Homestuck PT-BR",
                    "author": "Homestuck BR Team",
                    "version": 2.0,
                    "description": "Portuguese-Brazilian translation mod for Homestuck",
                    "enabled": "Homestuck PT-BR" in state.get("active_mods", []),
                }
            ]
            self.send_json_response(200, mods)
            return

        elif command == "toggle_mod":
            mod_id = args.get("mod_id") or args.get("id")
            enabled = bool(args.get("enabled", False))

            if mod_id != "Homestuck PT-BR":
                self.send_json_response(404, {"error": f"Unknown mod '{mod_id}'"})
                return

            active = state.setdefault("active_mods", [])
            if enabled and mod_id not in active:
                active.append(mod_id)
            elif not enabled and mod_id in active:
                active.remove(mod_id)

            # Keep config.settings.modListEnabled in sync
            cfg_mods = state["config"].setdefault("settings", {}).setdefault("modListEnabled", [])
            if enabled and mod_id not in cfg_mods:
                cfg_mods.append(mod_id)
            elif not enabled and mod_id in cfg_mods:
                cfg_mods.remove(mod_id)

            self.send_json_response(200, {"status": "ok", "mod_id": mod_id, "enabled": enabled})
            return

        self.send_json_response(400, {"error": f"Unknown IPC command '{command}'"})

    def send_json_response(self, status: int, data: dict):
        body = json.dumps(data).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.send_cors_headers()
        self.end_headers()
        self.wfile.write(body)

    def send_error_response(self, status: int, message: str):
        body = json.dumps({"error": message, "status": status}).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.send_cors_headers()
        self.end_headers()
        self.wfile.write(body)


class ThreadedTCPServer(socketserver.ThreadingMixIn, http.server.HTTPServer):
    daemon_threads = True
    allow_reuse_address = True


def create_mock_server(port: int = 0):
    """Create and initialize a reference contract server loaded with oracle data."""
    # Preload oracle data
    with open(MSPA_DATA_PATH, "r", encoding="utf-8") as f:
        mspa = json.load(f)
    with open(MUSIC_DATA_PATH, "r", encoding="utf-8") as f:
        music = json.load(f)
    with open(PTBR_TRANSLATION_PATH, "r", encoding="utf-8") as f:
        ptbr_translation = json.load(f)
    with open(CONFIG_PATH, "r", encoding="utf-8") as f:
        config = json.load(f)

    server = ThreadedTCPServer(("127.0.0.1", port), TuhcContractHandler)
    server.app_state = {
        "mspa": mspa,
        "music": music,
        "ptbr_translation": ptbr_translation,
        "config": config,
        "active_mods": list(config.get("settings", {}).get("modListEnabled", [])),
    }
    return server
