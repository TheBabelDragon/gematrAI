#!/usr/bin/env python3
"""Verify mandatory reverse-lookup acceptance cases against generated artifacts."""
from __future__ import annotations
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GEN = ROOT / "corpus" / "generated"

CASES = [
    ("hebrew-standard", 18, "חי"),
    ("hebrew-standard", 376, "שלום"),
    ("hebrew-standard", 13, "אהבה"),
    ("english-ordinal", 54, "love"),
    ("greek-isopsephy", 373, "λογος"),
]

def load_pack(name: str):
    return json.loads((GEN / name).read_text(encoding="utf-8"))

def main() -> int:
    low = load_pack("values_low.json")
    mid = load_pack("values_mid.json")
    failed = 0
    for sys_id, val, term in CASES:
        pack = low if val < 100 else mid
        items = (pack.get(sys_id) or {}).get(str(val)) or []
        terms = [x.get("t") for x in items]
        ok = term in terms or any(term in (t or "") for t in terms)
        if sys_id.startswith("greek"):
            ok = ok or any("λογος" in (t or "") or "λόγος" in (t or "") for t in terms)
        status = "OK" if ok else "FAIL"
        if not ok:
            failed += 1
        print(f"{status}: {term} @ {sys_id}={val} terms={terms[:8]}")
    found_ref = False
    for pack in (low, mid):
        for val, items in (pack.get("hebrew-standard") or {}).items():
            for it in items:
                if it.get("t") == "בראשית" and it.get("r"):
                    found_ref = True
                    print(f"OK: fixture בראשית refs={it.get('r')}")
    if not found_ref:
        shard = GEN / "values" / "hebrew-standard"
        if shard.is_dir():
            for f in shard.glob("*.json"):
                data = json.loads(f.read_text(encoding="utf-8"))
                for val, items in data.items():
                    for it in items:
                        if it.get("t") == "בראשית" and it.get("r"):
                            found_ref = True
                            print(f"OK: fixture בראשית refs={it.get('r')} books={it.get('b')}")
    if not found_ref:
        print("FAIL: fixture term missing source ref")
        failed += 1
    return 1 if failed else 0

if __name__ == "__main__":
    raise SystemExit(main())
