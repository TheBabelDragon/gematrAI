#!/usr/bin/env python3
"""Exact acceptance: terms must appear as exact matches in generated packs."""
from __future__ import annotations
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GEN = ROOT / "corpus" / "generated"

CASES = [
    ("hebrew-standard", 18, "\u05d7\u05d9"),
    ("hebrew-standard", 376, "\u05e9\u05dc\u05d5\u05dd"),
    ("hebrew-standard", 13, "\u05d0\u05d4\u05d1\u05d4"),
    ("english-ordinal", 54, "love"),
    ("greek-isopsephy", 373, "\u03bb\u03bf\u03b3\u03bf\u03c2"),
]


def pack_for(val: int):
    if val < 100:
        return json.loads((GEN / "values_low.json").read_text(encoding="utf-8"))
    if val < 1000:
        return json.loads((GEN / "values_mid.json").read_text(encoding="utf-8"))
    raise SystemExit(f"value {val} needs shard lookup")


def main() -> int:
    if not (GEN / "manifest.json").is_file():
        print("FAIL: missing manifest")
        return 1
    manifest = json.loads((GEN / "manifest.json").read_text(encoding="utf-8"))
    st = manifest.get("stats") or {}
    failed = 0
    for sys_id, val, term in CASES:
        pack = pack_for(val)
        items = (pack.get(sys_id) or {}).get(str(val)) or []
        hits = [x for x in items if x.get("t") == term]
        if not hits:
            print(
                f"FAIL: exact {term!r} @ {sys_id}={val} missing; have={[x.get('t') for x in items]}"
            )
            failed += 1
        else:
            print(f"OK: exact {term!r} @ {sys_id}={val} terms={[x.get('t') for x in hits]}")
    low = json.loads((GEN / "values_low.json").read_text(encoding="utf-8"))
    mid = json.loads((GEN / "values_mid.json").read_text(encoding="utf-8"))
    found = False
    for pack in (low, mid):
        for sys_id, valmap in pack.items():
            if not sys_id.startswith("hebrew"):
                continue
            for val, items in valmap.items():
                for it in items:
                    if it.get("t") == "\u05d1\u05e8\u05d0\u05e9\u05d9\u05ea":
                        refs = it.get("r") or []
                        books = it.get("b") or []
                        kind = it.get("kind")
                        if refs or kind == "fixture":
                            print(
                                f"OK: fixture \u05d1\u05e8\u05d0\u05e9\u05d9\u05ea refs={refs} books={books} kind={kind}"
                            )
                            found = True
    if not found:
        print("FAIL: bereshit fixture provenance not found")
        failed += 1
    print(
        json.dumps(
            {
                "all_ok": failed == 0,
                "unique_terms": st.get("unique_terms"),
                "segments": st.get("segments"),
            }
        )
    )
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
