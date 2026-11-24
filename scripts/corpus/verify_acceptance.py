#!/usr/bin/env python3
"""Verify mandatory reverse-lookup acceptance cases against generated artifacts."""
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


def load_pack(name: str):
    return json.loads((GEN / name).read_text(encoding="utf-8"))


def exact_hit(items, term: str) -> bool:
    return any(x.get("t") == term for x in items)


def main() -> int:
    low = load_pack("values_low.json")
    mid = load_pack("values_mid.json")
    failed = 0
    for sys_id, val, term in CASES:
        pack = low if val < 100 else mid
        items = (pack.get(sys_id) or {}).get(str(val)) or []
        ok = exact_hit(items, term)
        if not ok and sys_id == "greek-isopsephy":
            ok = exact_hit(items, "\u03bb\u03cc\u03b3\u03bf\u03c2") or exact_hit(items, "\u03bb\u03bf\u03b3\u03bf\u03c2")
        status = "OK" if ok else "FAIL"
        if not ok:
            failed += 1
        terms = [x.get("t") for x in items]
        print(f"{status}: exact {term!r} @ {sys_id}={val} terms={terms[:12]}")

    found_ref = False
    for pack in (low, mid):
        for val, items in (pack.get("hebrew-standard") or {}).items():
            for it in items:
                if it.get("t") == "\u05d1\u05e8\u05d0\u05e9\u05d9\u05ea" and it.get("r"):
                    found_ref = True
                    print(
                        f"OK: fixture \u05d1\u05e8\u05d0\u05e9\u05d9\u05ea refs={it.get('r')} books={it.get('b')} kind={it.get('kind')}"
                    )
                    break
            if found_ref:
                break
        if found_ref:
            break
    if not found_ref:
        shard_dir = GEN / "values" / "hebrew-standard"
        if shard_dir.is_dir():
            for f in sorted(shard_dir.glob("*.json")):
                data = json.loads(f.read_text(encoding="utf-8"))
                for val, items in data.items():
                    for it in items:
                        if it.get("t") == "\u05d1\u05e8\u05d0\u05e9\u05d9\u05ea" and it.get("r"):
                            found_ref = True
                            print(
                                f"OK: fixture \u05d1\u05e8\u05d0\u05e9\u05d9\u05ea refs={it.get('r')} books={it.get('b')}"
                            )
                            break
                    if found_ref:
                        break
                if found_ref:
                    break
    if not found_ref:
        print("FAIL: fixture term \u05d1\u05e8\u05d0\u05e9\u05d9\u05ea missing source ref")
        failed += 1
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
