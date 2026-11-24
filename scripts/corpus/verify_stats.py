#!/usr/bin/env python3
"""Exact independent recount of corpus artifacts vs manifest — no double-counting."""
from __future__ import annotations
import json
import sys
from pathlib import Path

GEN = Path(__file__).resolve().parents[2] / "corpus" / "generated"


def main() -> int:
    manifest = json.loads((GEN / "manifest.json").read_text(encoding="utf-8"))
    st = manifest.get("stats") or {}

    unique: dict[tuple[str, str], dict] = {}
    pack_keys: set[tuple[str, str, str]] = set()

    def ingest(sys_id: str, val: str, items: list, from_shard: bool) -> None:
        for it in items:
            t = it.get("t") or ""
            key = (sys_id, t)
            triple = (sys_id, str(val), t)
            if from_shard and triple in pack_keys:
                continue
            if not from_shard:
                pack_keys.add(triple)
            kinds = set(it.get("kinds") or ([it["kind"]] if it.get("kind") else []))
            prev = unique.get(key)
            if prev is None:
                unique[key] = {"c": int(it.get("c") or 0), "kinds": kinds}
            else:
                prev["c"] = max(prev["c"], int(it.get("c") or 0))
                prev["kinds"] |= kinds

    for name in ("values_low.json", "values_mid.json"):
        pack = json.loads((GEN / name).read_text(encoding="utf-8"))
        for sys_id, valmap in pack.items():
            for val, items in valmap.items():
                ingest(sys_id, val, items, from_shard=False)

    values_dir = GEN / "values"
    shard_files = list(values_dir.rglob("*.json")) if values_dir.is_dir() else []
    for sf in shard_files:
        payload = json.loads(sf.read_text(encoding="utf-8"))
        parts = sf.parts
        try:
            sys_id = parts[parts.index("values") + 1]
        except ValueError:
            continue
        for val, items in payload.items():
            ingest(sys_id, val, items, from_shard=True)

    seed_slots = {k for k, v in unique.items() if "seed" in v["kinds"]}
    text_slots = {k for k, v in unique.items() if v["kinds"] & {"fixture", "text"}}
    multi = sum(1 for v in unique.values() if len(v["kinds"]) > 1)
    occ = sum(v["c"] for v in unique.values())

    errors = []
    if st.get("unique_terms") != len(unique):
        errors.append(f"unique_terms manifest={st.get('unique_terms')} actual={len(unique)}")
    if st.get("seed_terms") is not None and st.get("seed_terms") != len(seed_slots):
        errors.append(f"seed_terms manifest={st.get('seed_terms')} actual={len(seed_slots)}")
    if st.get("source_terms") is not None and st.get("source_terms") != len(text_slots):
        errors.append(f"source_terms manifest={st.get('source_terms')} actual={len(text_slots)}")
    if st.get("token_occurrences") is not None and st.get("token_occurrences") != occ:
        errors.append(f"token_occurrences manifest={st.get('token_occurrences')} actual={occ}")
    if st.get("multi_kind_terms") is not None and st.get("multi_kind_terms") != multi:
        errors.append(f"multi_kind_terms manifest={st.get('multi_kind_terms')} actual={multi}")

    for name in ("values_low.json", "values_mid.json", "manifest.json"):
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

    print(f"unique={len(unique)} manifest={st.get('unique_terms')}")
    print(f"seed={len(seed_slots)} manifest={st.get('seed_terms')}")
    print(f"text={len(text_slots)} manifest={st.get('source_terms')}")
    print(f"multi={multi} manifest={st.get('multi_kind_terms')}")
    print(f"occ={occ} manifest={st.get('token_occurrences')}")
    print(f"shards={len(shard_files)} segments={st.get('segments')}")
    if errors:
        for e in errors:
            print("FAIL:", e)
        return 1
    print("OK: exact stats verification passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
