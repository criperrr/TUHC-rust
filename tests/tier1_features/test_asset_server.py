"""
Tier 1: Feature Coverage - Asset Server (F03)
Tests HTTP loopback asset server, Range streaming, MIME types, and CORS.
"""

import os
from tests.common import TuhcTestCase, ASSET_PACK_DIR


class TestAssetServer(TuhcTestCase):

    def test_asset_server_serves_gif(self):
        rel_path = "advimgs/bq/bq001.gif"
        file_path = os.path.join(ASSET_PACK_DIR, rel_path)
        expected_size = os.path.getsize(file_path)

        resp = self.client.get(f"/assets/{rel_path}")
        self.assertStatus(resp, 200)
        self.assertHeader(resp, "Content-Type", "image/gif")
        self.assertHeader(resp, "Content-Length", str(expected_size))
        self.assertEqual(len(resp.data), expected_size)
        with open(file_path, "rb") as f:
            self.assertEqual(resp.data, f.read())

    def test_asset_server_serves_png(self):
        rel_path = "archive/beyond/ao3_logo.png"
        file_path = os.path.join(ASSET_PACK_DIR, rel_path)
        expected_size = os.path.getsize(file_path)

        resp = self.client.get(f"/assets/{rel_path}")
        self.assertStatus(resp, 200)
        self.assertHeader(resp, "Content-Type", "image/png")
        self.assertHeader(resp, "Content-Length", str(expected_size))
        self.assertEqual(len(resp.data), expected_size)

    def test_asset_server_serves_mp3(self):
        rel_path = "archive/music/extra/hilary-troiano-commentary.mp3"
        file_path = os.path.join(ASSET_PACK_DIR, rel_path)
        expected_size = os.path.getsize(file_path)

        resp = self.client.get(f"/assets/{rel_path}")
        self.assertStatus(resp, 200)
        self.assertHeader(resp, "Content-Type", "audio/mpeg")
        self.assertHeader(resp, "Content-Length", str(expected_size))
        self.assertEqual(len(resp.data), expected_size)

    def test_asset_server_serves_swf(self):
        rel_path = "archive/minutia/006385.swf"
        file_path = os.path.join(ASSET_PACK_DIR, rel_path)
        expected_size = os.path.getsize(file_path)

        resp = self.client.get(f"/assets/{rel_path}")
        self.assertStatus(resp, 200)
        self.assertHeader(resp, "Content-Type", "application/x-shockwave-flash")
        self.assertHeader(resp, "Content-Length", str(expected_size))
        self.assertEqual(len(resp.data), expected_size)

    def test_asset_server_serves_wasm(self):
        rel_path = "archive/imods/_twoToThree/js/ruffle/e6c08a4d57d1c54d19e9.wasm"
        file_path = os.path.join(ASSET_PACK_DIR, rel_path)
        expected_size = os.path.getsize(file_path)

        resp = self.client.get(f"/assets/{rel_path}")
        self.assertStatus(resp, 200)
        self.assertHeader(resp, "Content-Type", "application/wasm")
        self.assertHeader(resp, "Content-Length", str(expected_size))
        self.assertEqual(len(resp.data), expected_size)

    def test_asset_server_range_request_prefix(self):
        rel_path = "advimgs/bq/bq001.gif"
        file_path = os.path.join(ASSET_PACK_DIR, rel_path)
        total_size = os.path.getsize(file_path)

        resp = self.client.get(f"/assets/{rel_path}", headers={"Range": "bytes=0-1023"})
        self.assertStatus(resp, 206)
        self.assertHeader(resp, "Content-Range", f"bytes 0-1023/{total_size}")
        self.assertHeader(resp, "Content-Length", "1024")
        self.assertEqual(len(resp.data), 1024)
        with open(file_path, "rb") as f:
            self.assertEqual(resp.data, f.read(1024))

    def test_asset_server_range_request_suffix(self):
        rel_path = "advimgs/bq/bq001.gif"
        file_path = os.path.join(ASSET_PACK_DIR, rel_path)
        total_size = os.path.getsize(file_path)

        suffix_len = 500
        start = total_size - suffix_len
        end = total_size - 1

        resp = self.client.get(f"/assets/{rel_path}", headers={"Range": f"bytes=-{suffix_len}"})
        self.assertStatus(resp, 206)
        self.assertHeader(resp, "Content-Range", f"bytes {start}-{end}/{total_size}")
        self.assertHeader(resp, "Content-Length", str(suffix_len))
        self.assertEqual(len(resp.data), suffix_len)

    def test_asset_server_head_request(self):
        rel_path = "advimgs/bq/bq001.gif"
        file_path = os.path.join(ASSET_PACK_DIR, rel_path)
        expected_size = os.path.getsize(file_path)

        resp = self.client.head(f"/assets/{rel_path}")
        self.assertStatus(resp, 200)
        self.assertHeader(resp, "Content-Length", str(expected_size))
        self.assertEqual(len(resp.data), 0)

    def test_asset_server_cors_headers(self):
        rel_path = "advimgs/bq/bq001.gif"
        resp = self.client.get(f"/assets/{rel_path}")
        self.assertHeader(resp, "Access-Control-Allow-Origin", "*")
