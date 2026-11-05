"""
Tier 2: Boundary & Corner Cases - Malformed Range Requests
Tests HTTP 416 Range Not Satisfiable handling for invalid, inverted, and out-of-bounds ranges.
"""

from tests.common import TuhcTestCase


class TestMalformedRanges(TuhcTestCase):

    def test_range_non_numeric_returns_416(self):
        resp = self.client.get(
            "/assets/advimgs/bq/bq001.gif",
            headers={"Range": "bytes=abc-def"}
        )
        self.assertStatus(resp, 416)

    def test_range_inverted_returns_416(self):
        resp = self.client.get(
            "/assets/advimgs/bq/bq001.gif",
            headers={"Range": "bytes=500-100"}
        )
        self.assertStatus(resp, 416)

    def test_range_out_of_bounds_returns_416(self):
        resp = self.client.get(
            "/assets/advimgs/bq/bq001.gif",
            headers={"Range": "bytes=999999999-999999999"}
        )
        self.assertStatus(resp, 416)

    def test_range_multiple_hyphens_returns_416(self):
        resp = self.client.get(
            "/assets/advimgs/bq/bq001.gif",
            headers={"Range": "bytes=1-2-3"}
        )
        self.assertStatus(resp, 416)

    def test_range_invalid_unit_returns_416(self):
        resp = self.client.get(
            "/assets/advimgs/bq/bq001.gif",
            headers={"Range": "items=0-10"}
        )
        self.assertStatus(resp, 416)
