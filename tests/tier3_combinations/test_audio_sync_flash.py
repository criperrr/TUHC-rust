"""
Tier 3: Cross-Feature Combinations - Audio Sync Delay Calculations & Flash Assets
Tests cross-referencing between Flash .swf asset delivery and audio delay calibration tables.
"""

from tests.common import TuhcTestCase, load_oracle_flash_props


class TestAudioSyncFlash(TuhcTestCase):

    @classmethod
    def setUpClass(cls):
        super().setUpClass()
        cls.flash_props = load_oracle_flash_props()

    def test_flash_asset_and_negative_seek_delay(self):
        # Page 003701: [S] WV?: Rise up. -> /storyfiles/hs2/01801/01801.swf
        swf_path = "storyfiles/hs2/01801/01801.swf"
        resp = self.client.get(f"/assets/{swf_path}")
        self.assertStatus(resp, 200)
        self.assertHeader(resp, "Content-Type", "application/x-shockwave-flash")

        # Audio delay lookup for 01801: negative 100ms
        audio_delays = self.flash_props.get("audioDelay", {})
        delay = audio_delays.get("01801")
        self.assertEqual(delay, -100)

    def test_cascade_multi_segment_flash_and_delays(self):
        # Page 006009: [S] Cascade. -> /storyfiles/hs2/04106/04106.swf
        swf_path = "storyfiles/hs2/04106/04106.swf"
        resp = self.client.get(f"/assets/{swf_path}")
        self.assertStatus(resp, 200)
        self.assertHeader(resp, "Content-Type", "application/x-shockwave-flash")

        # Cascade multi-segment delay table verification
        audio_delays = self.flash_props.get("audioDelay", {})
        self.assertEqual(audio_delays.get("04106_1"), -50)
        self.assertEqual(audio_delays.get("04106_2"), 575)
        self.assertEqual(audio_delays.get("04106_3"), 1550)
        self.assertEqual(audio_delays.get("04106_4"), 600)
        self.assertEqual(audio_delays.get("04106_5"), 1200)
