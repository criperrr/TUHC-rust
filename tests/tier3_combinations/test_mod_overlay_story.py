"""
Tier 3: Cross-Feature Combinations - Mod PT-BR Overlay + Story Page Fetch
Tests simultaneous JSON translation merging and VFS media overlay.
"""

from tests.common import TuhcTestCase, load_oracle_ptbr_translation


class TestModOverlayStory(TuhcTestCase):

    @classmethod
    def setUpClass(cls):
        super().setUpClass()
        cls.oracle_ptbr = load_oracle_ptbr_translation()

    def test_mod_enabled_overlays_translation_and_vfs_media(self):
        # 1. Enable PT-BR Mod
        self.client.ipc("toggle_mod", {"mod_id": "Homestuck PT-BR", "enabled": True})

        # 2. Verify translated story page 001901
        resp = self.client.ipc("get_story_page", {"page_id": "001901"})
        self.assertStatus(resp, 200)
        page = resp.json()
        expected_content = self.oracle_ptbr["001901"]["content"]
        self.assertEqual(page["content"], expected_content)
        self.assertIn("Um jovem rapaz", page["content"])

        # 3. Verify translated title for page 001902
        resp2 = self.client.ipc("get_story_page", {"page_id": "001902"})
        self.assertStatus(resp2, 200)
        self.assertEqual(resp2.json()["title"], "Insira o nome.")

        # 4. Verify VFS media overlay for 00002.gif (Portuguese localized image is 191,716 bytes)
        asset_resp = self.client.get("/assets/storyfiles/hs2/00002.gif")
        self.assertStatus(asset_resp, 200)
        self.assertEqual(len(asset_resp.data), 191716)

        # 5. Disable PT-BR Mod
        self.client.ipc("toggle_mod", {"mod_id": "Homestuck PT-BR", "enabled": False})

        # 6. Verify base story page returns canonical English
        resp_base = self.client.ipc("get_story_page", {"page_id": "001901"})
        self.assertStatus(resp_base, 200)
        self.assertIn("A young man stands in his bedroom", resp_base.json()["content"])

        # 7. Verify base asset returns canonical size (25,043 bytes)
        asset_base_resp = self.client.get("/assets/storyfiles/hs2/00002.gif")
        self.assertStatus(asset_base_resp, 200)
        self.assertEqual(len(asset_base_resp.data), 25043)
