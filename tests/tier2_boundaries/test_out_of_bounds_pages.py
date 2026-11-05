"""
Tier 2: Boundary & Corner Cases - Out-of-Bounds Story Pages
Tests boundary behavior for non-existent, negative, zero, and malformed story page requests.
"""

from tests.common import TuhcTestCase


class TestOutOfBoundsPages(TuhcTestCase):

    def test_page_id_zero_returns_404(self):
        resp = self.client.ipc("get_story_page", {"page_id": "000000"})
        self.assertStatus(resp, 404)

    def test_page_id_negative_returns_404(self):
        resp = self.client.ipc("get_story_page", {"page_id": "-1"})
        self.assertStatus(resp, 404)

    def test_page_id_beyond_end_returns_404(self):
        resp = self.client.ipc("get_story_page", {"page_id": "999999"})
        self.assertStatus(resp, 404)

    def test_alphanumeric_invalid_page_id_returns_404(self):
        resp = self.client.ipc("get_story_page", {"page_id": "chapter_two_page_abc"})
        self.assertStatus(resp, 404)

    def test_empty_string_page_id_returns_404(self):
        resp = self.client.ipc("get_story_page", {"page_id": ""})
        self.assertStatus(resp, 404)
