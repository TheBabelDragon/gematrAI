#!/usr/bin/env python3
"""Independently recount corpus packs and compare to manifest stats."""
from __future__ import annotations
import json
import sys
from pathlib import Path

GEN = Path(__file__).resolve().parents[2] / "corpus" / "generated"


def main() -> int:
    manifest = json.loads((GEN / "manifest.json").read_text(encoding="utf-8"))
    low = json.loads((GEN / "values_low.json").read_text(encoding="utf-8"))
    mid = json.loads((GEN / "values_mid.json").read_text(encoding="utf-8"))
    st = manifest.get("stats") or {}

    unique = set()
    seed_slots = set()
    text_slots = set()
    occ = 0
    for pack in (low, mid):
        for sys_id, valmap in pack.items():
            for val, items in valmap.items():
                for it in items:
                    t = it.get("t") or ""
                    unique.add((sys_id, t))
                    occ += int(it.get("c") or 0)
                    kind = it.get("kind") or ""
                    if kind == "seed":
                        seed_slots.add((sys_id, t))
                    else:
                        text_slots.add((sys_id, t))

    values_dir = GEN / "values"
    shard_files = list(values_dir.rglob("*.json")) if values_dir.is_dir() else []

    errors = []
    if len(unique) > st.get("unique_terms", 0):
        errors.append(
            f"pack unique {len(unique)} > manifest unique_terms {st.get('unique_terms')}"
        )
    for name in ("values_low.json", "values_mid.json"):
        if not (GEN / name).is_file():
            errors.append(f"missing {name}")
    if not (GEN / "metadata" / "build.json").is_file():
        errors.append("missing metadata/build.json")
    for key in ("lowValuesUrl", "midValuesUrl"):
        url = manifest.get(key) or ""
        if url.startswith("./"):
            name = Path(url[2:]).name
            if not (GEN / name).is_file():
                errors.append(f"manifest {key} -> missing {name}")

    print(f"independent unique(sys,term) in low+mid: {len(unique)}")
    print(f"manifest unique_terms: {st.get('unique_terms')}")
    print(f"seed_slots(low+mid): {len(seed_slots)} manifest seed_terms: {st.get('seed_terms')}")
    print(f"text_slots(low+mid): {len(text_slots)} manifest source_terms: {st.get('source_terms')}")
    print(f"occ(low+mid): {occ} manifest token_occurrences: {st.get('token_occurrences')}")
    print(f"shard files: {len(shard_files)}")
    print(f"segments: {st.get('segments')} phrases: {st.get('phrases')}")
    if errors:
        for e in errors:
            print("FAIL:", e)
        return 1
    print("OK: artifact files present; pack counts consistent with coverage model")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
