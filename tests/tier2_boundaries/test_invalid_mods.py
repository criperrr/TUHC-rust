"""
Tier 2: Boundary & Corner Cases - Invalid Mod Structures
Tests error resilience when loading, toggling, or parsing non-existent or invalid mods.
"""

from tests.common import TuhcTestCase


class TestInvalidMods(TuhcTestCase):

    def test_toggle_non_existent_mod_returns_404(self):
        resp = self.client.ipc("toggle_mod", {"mod_id": "NonExistentMod_9876", "enabled": True})
        self.assertStatus(resp, 404)
        self.assertIn("error", resp.json())

    def test_mod_toggle_with_empty_payload_returns_error(self):
        resp = self.client.ipc("toggle_mod", {})
        self.assertStatus(resp, 404)

    def test_available_mods_filters_non_mod_directories(self):
        resp = self.client.ipc("get_available_mods")
        self.assertStatus(resp, 200)
        mods = resp.json()
        # Ensure mods have mandatory attributes
        for m in mods:
            self.assertIn("id", m)
            self.assertIn("title", m)
            self.assertIn("author", m)

    def test_toggle_mod_with_numeric_id_returns_404(self):
        resp = self.client.ipc("toggle_mod", {"mod_id": 12345, "enabled": True})
        self.assertStatus(resp, 404)

    def test_toggle_mod_with_none_enabled_defaults_to_false(self):
        resp = self.client.ipc("toggle_mod", {"mod_id": "Homestuck PT-BR", "enabled": None})
        self.assertStatus(resp, 200)
        self.assertFalse(resp.json()["enabled"])
