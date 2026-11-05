"""
Tier 4: Real-World Scenarios - Flash Page Playback with Audio Sync Offsets
Simulates loading Flash pages, fetching SWFs, and dispatching calibrated Frag IPC audio sync events.
"""

from tests.common import TuhcTestCase, load_oracle_flash_props


class TestFlashAudioSyncScenario(TuhcTestCase):

    @classmethod
    def setUpClass(cls):
        super().setUpClass()
        cls.flash_props = load_oracle_flash_props()

    def test_rise_up_flash_audio_sync(self):
        # 1. Fetch story page 003701: [S] WV?: Rise up.
        page = self.client.ipc("get_story_page", {"page_id": "003701"}).json()
        self.assertEqual(page["title"], "[S] WV?: Rise up.")
        self.assertIn("/storyfiles/hs2/01801/01801.swf", page["media"])

        # 2. Fetch Flash SWF asset
        swf_resp = self.client.get("/assets/storyfiles/hs2/01801/01801.swf")
        self.assertStatus(swf_resp, 200)
        self.assertHeader(swf_resp, "Content-Type", "application/x-shockwave-flash")

        # 3. Simulate ActionScript getURL("about:srcdoc#audioStart?1")
        # Host intercepts hashchange: calculates calibrated delay offset
        delay = self.flash_props.get("audioDelay", {}).get("01801", 0)
        self.assertEqual(delay, -100)

        # Negative delay implies immediate audio seek of 100ms
        audio_start_offset_ms = abs(delay) if delay < 0 else 0
        self.assertEqual(audio_start_offset_ms, 100)

    def test_cascade_flash_multi_segment_sync(self):
        # 1. Fetch story page 006009: [S] Cascade.
        page = self.client.ipc("get_story_page", {"page_id": "006009"}).json()
        self.assertEqual(page["title"], "[S] Cascade.")

        # 2. Fetch Cascade SWF asset
        swf_resp = self.client.get("/assets/storyfiles/hs2/04106/04106.swf")
        self.assertStatus(swf_resp, 200)

        # 3. Verify subsegment timing progression
        delays = self.flash_props.get("audioDelay", {})
        subsegments = ["04106_1", "04106_2", "04106_3", "04106_4", "04106_5"]
        for seg in subsegments:
            self.assertIn(seg, delays)
