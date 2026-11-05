"""
Tier 1: Feature Coverage - Config & Save Persistence (F05)
Tests config.json loading, updating, schema validation, and persistence.
"""

from tests.common import TuhcTestCase, load_oracle_config


class TestConfigPersistence(TuhcTestCase):

    @classmethod
    def setUpClass(cls):
        super().setUpClass()
        cls.oracle_config = load_oracle_config()

    def test_load_config_matches_schema(self):
        resp = self.client.ipc("load_config")
        self.assertStatus(resp, 200)
        cfg = resp.json()
        self.assertIn("assetDir", cfg)
        self.assertIn("tabData", cfg)
        self.assertIn("saveData", cfg)
        self.assertIn("settings", cfg)
        self.assertEqual(cfg["assetDir"], self.oracle_config["assetDir"])

    def test_config_mutates_active_tab(self):
        resp = self.client.ipc("load_config")
        self.assertStatus(resp, 200)
        cfg = resp.json()

        cfg["tabData"]["activeTabKey"] = "test_tab_001"
        save_resp = self.client.ipc("save_config", {"config": cfg})
        self.assertStatus(save_resp, 200)

        reload_resp = self.client.ipc("load_config")
        self.assertStatus(reload_resp, 200)
        reloaded = reload_resp.json()
        self.assertEqual(reloaded["tabData"]["activeTabKey"], "test_tab_001")

    def test_config_preserves_new_reader_settings(self):
        resp = self.client.ipc("load_config")
        self.assertStatus(resp, 200)
        cfg = resp.json()

        cfg["settings"]["newReader"]["current"] = "002500"
        self.client.ipc("save_config", {"config": cfg})

        reloaded = self.client.ipc("load_config").json()
        self.assertEqual(reloaded["settings"]["newReader"]["current"], "002500")

    def test_config_persists_theme_override(self):
        resp = self.client.ipc("load_config")
        cfg = resp.json()

        cfg["settings"]["themeOverride"] = "dark"
        self.client.ipc("save_config", {"config": cfg})

        reloaded = self.client.ipc("load_config").json()
        self.assertEqual(reloaded["settings"]["themeOverride"], "dark")

    def test_config_mod_list_enabled_persistence(self):
        resp = self.client.ipc("load_config")
        cfg = resp.json()

        cfg["settings"]["modListEnabled"] = ["Homestuck PT-BR"]
        self.client.ipc("save_config", {"config": cfg})

        reloaded = self.client.ipc("load_config").json()
        self.assertEqual(reloaded["settings"]["modListEnabled"], ["Homestuck PT-BR"])

        # Reset back to empty list to preserve test independence
        cfg["settings"]["modListEnabled"] = []
        self.client.ipc("save_config", {"config": cfg})
