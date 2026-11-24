#!/usr/bin/env python3
"""Simulate browser reverse-lookup against generated packs (no JS engine)."""
from __future__ import annotations
import json
import sys
from pathlib import Path

GEN = Path(__file__).resolve().parents[2] / "corpus" / "generated"

CASES = [
    ("hebrew-standard", 18, "\u05d7\u05d9"),
    ("hebrew-standard", 376, "\u05e9\u05dc\u05d5\u05dd"),
    ("hebrew-standard", 13, "\u05d0\u05d4\u05d1\u05d4"),
    ("english-ordinal", 54, "love"),
    ("greek-isopsephy", 373, "\u03bb\u03bf\u03b3\u03bf\u03c2"),
]


def load(name):
    p = GEN / name
    if not p.is_file():
        raise SystemExit(f"missing {p}")
    return json.loads(p.read_text(encoding="utf-8"))


def main() -> int:
    manifest = load("manifest.json")
    low = load("values_low.json")
    mid = load("values_mid.json")
    stats = manifest.get("stats") or {}
    ut = stats.get("unique_terms")
    if ut is None:
        raise SystemExit("manifest missing unique_terms")
    failed = 0
    for sys_id, val, term in CASES:
        pack = low if val < 100 else mid
        if sys_id not in pack:
            print(f"FAIL: system {sys_id} missing from pack")
            failed += 1
            continue
        items = pack[sys_id].get(str(val)) or []
        if not any(x.get("t") == term for x in items):
            print(f"FAIL: browser path no exact {term!r} @ {sys_id}={val}")
            failed += 1
        else:
            print(f"OK: browser path {term!r} @ {sys_id}={val}")
    if (low.get("hebrew-standard") or {}).get("99999") is not None:
        print("FAIL: unexpected phantom key")
        failed += 1
    else:
        print("OK: empty high-value lookup distinguished (no phantom key)")
    print(
        f"manifest unique_terms={ut} segments={stats.get('segments')} seed={stats.get('seed_terms')}"
    )
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
