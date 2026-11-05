"""
Tier 4: Real-World Scenarios - Music Player & Sound Test Browsing
Simulates browsing the music archive, querying track details, and streaming audio with HTTP Range requests.
"""

from tests.common import TuhcTestCase


class TestMusicPlayerSessionScenario(TuhcTestCase):

    def test_music_player_session(self):
        # 1. Fetch full music archive
        resp = self.client.ipc("get_music_data")
        self.assertStatus(resp, 200)
        music_data = resp.json()

        # 2. Verify album list contains key albums
        albums = music_data.get("albums", {})
        self.assertIn("homestuck-vol-1", albums)
        vol1 = albums["homestuck-vol-1"]
        self.assertEqual(vol1["name"], "Homestuck Vol. 1")

        # 3. Inspect track details
        tracks = music_data.get("tracks", {})
        track = tracks.get("108-stars-of-density")
        self.assertIsNotNone(track)
        self.assertEqual(track["name"], "108 Stars of Density")
        self.assertEqual(track["duration"], 96)

        # 4. Stream audio asset with Range request
        audio_path = "archive/music/extra/hilary-troiano-commentary.mp3"
        range_resp = self.client.get(
            f"/assets/{audio_path}",
            headers={"Range": "bytes=0-4095"}
        )
        self.assertStatus(range_resp, 206)
        self.assertHeader(range_resp, "Content-Type", "audio/mpeg")
        self.assertEqual(len(range_resp.data), 4096)
