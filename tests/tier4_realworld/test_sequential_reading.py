"""
Tier 4: Real-World Scenarios - Sequential Story Reader Walkthrough
Simulates an end-to-end user reading session traversing consecutive Homestuck pages with history tracking.
"""

from tests.common import TuhcTestCase


class TestSequentialReadingScenario(TuhcTestCase):

    def test_sequential_reading_walkthrough(self):
        history = []
        current_page_id = "001901"

        expected_sequence = [
            ("001901", "Homestuck", "/storyfiles/hs2/00001.gif"),
            ("001902", "Enter name.", "/storyfiles/hs2/00002.gif"),
            ("001903", "Try again.", "/storyfiles/hs2/00003.gif"),
            ("001904", "Examine room.", "/storyfiles/hs2/00004.gif"),
        ]

        for pid, expected_title, expected_media in expected_sequence:
            resp = self.client.ipc("get_story_page", {"page_id": pid})
            self.assertStatus(resp, 200)
            page_data = resp.json()

            # Verify page metadata
            self.assertEqual(page_data["pageId"], pid)
            self.assertEqual(page_data["title"], expected_title)
            self.assertIn(expected_media, page_data["media"])

            # Verify asset server can serve the page media
            media_rel = expected_media.lstrip("/")
            asset_resp = self.client.get(f"/assets/{media_rel}")
            self.assertStatus(asset_resp, 200)

            # Record history
            history.append(pid)

        self.assertEqual(history, ["001901", "001902", "001903", "001904"])

        # Backward navigation check
        last_page = self.client.ipc("get_story_page", {"page_id": "001904"}).json()
        self.assertEqual(last_page.get("previous"), "001903")
