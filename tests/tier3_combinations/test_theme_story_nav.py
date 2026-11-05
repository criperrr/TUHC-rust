"""
Tier 3: Cross-Feature Combinations - Theme Override & Story Navigation
Tests persistence of theme override combined with story page retrieval.
"""

from tests.common import TuhcTestCase


class TestThemeStoryNav(TuhcTestCase):

    def test_theme_override_persistence_with_story_fetch(self):
        # 1. Update theme in config to 'dark'
        cfg = self.client.ipc("load_config").json()
        cfg["settings"]["themeOverride"] = "dark"
        self.client.ipc("save_config", {"config": cfg})

        # 2. Fetch story page
        page = self.client.ipc("get_story_page", {"page_id": "001901"}).json()
        self.assertEqual(page["pageId"], "001901")

        # 3. Reload config to confirm theme still 'dark'
        reloaded_cfg = self.client.ipc("load_config").json()
        self.assertEqual(reloaded_cfg["settings"]["themeOverride"], "dark")

        # Clean up
        reloaded_cfg["settings"]["themeOverride"] = "default"
        self.client.ipc("save_config", {"config": reloaded_cfg})
