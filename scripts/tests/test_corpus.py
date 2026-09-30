#!/usr/bin/env python3
"""Offline corpus tower tests (fixtures only — no live Sefaria)."""
from __future__ import annotations
import json, sys, unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts" / "corpus"))
from normalize import tokenize, for_calculation_hebrew, strip_marks
from gematrify import value_hebrew_standard, value_hebrew_gadol, value_hebrew_katan, systems_for_token, digital_root
from build_indexes import build_indexes, bucket_for
from build_tower import process_segments

class TestNormalize(unittest.TestCase):
    def test_niqqud(self):
        self.assertEqual(for_calculation_hebrew("בְּרֵאשִׁ֖ית"), "בראשית")
    def test_finals(self):
        for ch in "ךםןףץ":
            self.assertTrue(ch in for_calculation_hebrew(ch) or for_calculation_hebrew(ch)==ch)
        self.assertEqual(for_calculation_hebrew("ארץ"), "ארץ")
    def test_tokenize_hebrew(self):
        toks = tokenize("בראשית ברא אלהים", "he")
        self.assertEqual([t["normalized"] for t in toks], ["בראשית","ברא","אלהים"])

class TestGematria(unittest.TestCase):
    def test_standard(self):
        self.assertEqual(value_hebrew_standard("אהבה"), 13)
        self.assertEqual(value_hebrew_standard("חי"), 18)
        self.assertEqual(value_hebrew_standard("ך"), 20)
    def test_gadol(self):
        self.assertEqual(value_hebrew_gadol("ך"), 500)
        self.assertEqual(value_hebrew_gadol("ם"), 600)
    def test_katan(self):
        self.assertEqual(value_hebrew_katan("י"), 1)
        self.assertEqual(digital_root(18), 9)
    def test_english(self):
        s = systems_for_token("love", "en")
        self.assertEqual(s["english-ordinal"], 54)
        self.assertEqual(s["english-reduction"], 18)

class TestPipeline(unittest.TestCase):
    def setUp(self):
        self.segs = json.loads((ROOT/"corpus/fixtures/tower_segments.json").read_text(encoding="utf-8"))
    def test_process(self):
        tokens, phrases, n = process_segments(self.segs, "test", phrase_max=2)
        self.assertGreater(n, 0)
        self.assertGreater(len(tokens), 0)
        data = build_indexes(tokens, phrases)
        self.assertIn("hebrew-standard", data["stats"]["indexed_values"])
        rows = [t for t in tokens if t["normalized"]=="בראשית" and t["system"]=="hebrew-standard"]
        self.assertGreaterEqual(len(rows), 2)
    def test_bucket(self):
        self.assertEqual(bucket_for(42), "0000-0099")
        self.assertEqual(bucket_for(150), "0100-0199")

if __name__ == "__main__":
    unittest.main()
