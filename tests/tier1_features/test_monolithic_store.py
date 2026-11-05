"""
Tier 1: Feature Coverage - In-Memory Monolithic Store (F04)
Tests mspa.json and music.json loading, querying, and range indexing.
"""

from tests.common import TuhcTestCase, load_oracle_mspa, load_oracle_music


class TestMonolithicStore(TuhcTestCase):

    @classmethod
    def setUpClass(cls):
        super().setUpClass()
        cls.oracle_mspa = load_oracle_mspa()
        cls.oracle_music = load_oracle_music()

    def test_store_loads_first_homestuck_page(self):
        expected = self.oracle_mspa["story"]["001901"]
        resp = self.client.ipc("get_story_page", {"page_id": "001901"})
        self.assertStatus(resp, 200)
        data = resp.json()
        self.assertEqual(data.get("pageId"), "001901")
        self.assertEqual(data.get("title"), expected["title"])
        self.assertEqual(data.get("media"), expected["media"])
        self.assertEqual(data.get("next"), expected["next"])

    def test_store_loads_second_homestuck_page(self):
        expected = self.oracle_mspa["story"]["001902"]
        resp = self.client.ipc("get_story_page", {"page_id": "001902"})
        self.assertStatus(resp, 200)
        data = resp.json()
        self.assertEqual(data.get("pageId"), "001902")
        self.assertEqual(data.get("title"), expected["title"])
        self.assertEqual(data.get("media"), expected["media"])

    def test_store_loads_pesterlog_page(self):
        expected = self.oracle_mspa["story"]["002060"]
        resp = self.client.ipc("get_story_page", {"page_id": "002060"})
        self.assertStatus(resp, 200)
        data = resp.json()
        self.assertEqual(data.get("pageId"), "002060")
        self.assertIn("PESTERLOG", data.get("content", ""))

    def test_store_returns_story_range(self):
        resp = self.client.ipc("get_story_range", {"start": 1901, "count": 5})
        self.assertStatus(resp, 200)
        items = resp.json()
        self.assertEqual(len(items), 5)
        self.assertEqual(items[0]["pageId"], "001901")
        self.assertEqual(items[4]["pageId"], "001905")
        for i, item in enumerate(items):
            pid = str(1901 + i).zfill(6)
            self.assertEqual(item["title"], self.oracle_mspa["story"][pid]["title"])

    def test_store_loads_music_archive(self):
        resp = self.client.ipc("get_music_data")
        self.assertStatus(resp, 200)
        music_data = resp.json()
        self.assertIn("albums", music_data)
        self.assertIn("tracks", music_data)
        self.assertEqual(len(music_data["albums"]), len(self.oracle_music["albums"]))
        self.assertEqual(len(music_data["tracks"]), len(self.oracle_music["tracks"]))

    def test_store_queries_specific_track(self):
        expected_track = self.oracle_music["tracks"]["108-stars-of-density"]
        resp = self.client.ipc("get_music_data")
        self.assertStatus(resp, 200)
        music_data = resp.json()
        track = music_data["tracks"].get("108-stars-of-density")
        self.assertIsNotNone(track)
        self.assertEqual(track["name"], expected_track["name"])
        self.assertEqual(track["duration"], expected_track["duration"])
