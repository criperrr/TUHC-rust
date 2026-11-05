"""
Tier 2: Boundary & Corner Cases - Missing Assets & Path Traversal
Tests 404 error responses, directory traversal prevention, and path sanitization.
"""

from tests.common import TuhcTestCase


class TestMissingAssets(TuhcTestCase):

    def test_non_existent_asset_returns_404(self):
        resp = self.client.get("/assets/this_asset_does_not_exist_at_all.png")
        self.assertStatus(resp, 404)

    def test_path_traversal_dotdot_returns_404(self):
        resp = self.client.get("/assets/../../../../etc/passwd")
        self.assertStatus(resp, 404)

    def test_path_traversal_encoded_returns_404(self):
        resp = self.client.get("/assets/%2e%2e%2f%2e%2e%2fetc/passwd")
        self.assertStatus(resp, 404)

    def test_null_byte_in_path_returns_404(self):
        resp = self.client.get("/assets/advimgs/bq/bq001.gif%00.txt")
        self.assertStatus(resp, 404)

    def test_empty_asset_path_returns_404(self):
        resp = self.client.get("/assets/")
        self.assertStatus(resp, 404)
