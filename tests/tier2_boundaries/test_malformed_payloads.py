"""
Tier 2: Boundary & Corner Cases - Malformed Payloads & IPC Errors
Tests handling of empty request bodies, invalid JSON, and missing IPC command fields.
"""

from tests.common import TuhcTestCase


class TestMalformedPayloads(TuhcTestCase):

    def test_ipc_empty_body_returns_400(self):
        resp = self.client.post("/ipc", body=b"")
        self.assertStatus(resp, 400)

    def test_ipc_invalid_json_returns_400(self):
        resp = self.client.post("/ipc", body=b"{invalid_json: true")
        self.assertStatus(resp, 400)

    def test_ipc_missing_command_returns_400(self):
        resp = self.client.post("/ipc", json_payload={"args": {}})
        self.assertStatus(resp, 400)

    def test_save_config_invalid_schema_returns_400(self):
        resp = self.client.ipc("save_config", {"config": "not_a_valid_dict"})
        self.assertStatus(resp, 400)

    def test_ipc_unknown_command_returns_400(self):
        resp = self.client.ipc("this_command_does_not_exist_at_all")
        self.assertStatus(resp, 400)
