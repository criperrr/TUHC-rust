"""
Tier 4: Real-World Scenarios - Tab Session State Preservation & Restore
Simulates multi-tab browsing session creation, persistence, and state restore across app sessions.
"""

from tests.common import TuhcTestCase


class TestTabSessionRestoreScenario(TuhcTestCase):

    def test_tab_session_persistence_and_restore(self):
        # 1. Load initial config
        cfg = self.client.ipc("load_config").json()

        # 2. Simulate opening two tabs with navigation history
        mock_tab_data = {
            "activeTabKey": "tab_beta",
            "tabs": {
                "tab_alpha": {
                    "key": "tab_alpha",
                    "url": "/homestuck/1",
                    "title": "Homestuck",
                    "history": ["/"],
                    "future": [],
                },
                "tab_beta": {
                    "key": "tab_beta",
                    "url": "/homestuck/2060",
                    "title": "John: Go out to balcony.",
                    "history": ["/", "/homestuck/1"],
                    "future": [],
                },
            },
            "tabList": ["tab_alpha", "tab_beta"],
        }

        cfg["tabData"] = mock_tab_data
        save_resp = self.client.ipc("save_config", {"config": cfg})
        self.assertStatus(save_resp, 200)

        # 3. Simulate reload / app restart
        reloaded = self.client.ipc("load_config").json()
        reloaded_tabs = reloaded["tabData"]

        self.assertEqual(reloaded_tabs["activeTabKey"], "tab_beta")
        self.assertIn("tab_alpha", reloaded_tabs["tabs"])
        self.assertIn("tab_beta", reloaded_tabs["tabs"])
        self.assertEqual(reloaded_tabs["tabs"]["tab_beta"]["url"], "/homestuck/2060")
        self.assertEqual(reloaded_tabs["tabs"]["tab_beta"]["history"], ["/", "/homestuck/1"])
