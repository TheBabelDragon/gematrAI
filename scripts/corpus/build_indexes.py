"""Build sharded reverse indexes, phrase index, and stats."""
from __future__ import annotations
import json
from collections import defaultdict
from pathlib import Path
from typing import Any

def bucket_for(value: int, size: int = 100) -> str:
    b = (value // size) * size
    return f"{b:04d}-{(b + size - 1):04d}"

def build_indexes(token_rows: list[dict], phrase_rows: list[dict], bucket_size: int = 100) -> dict:
    """
    token_rows: {normalized, surface, system, value, ref, book, license, sourceUrl}
    Returns structure for writing shards.
    """
    by_sys: dict[str, dict[int, dict[str, dict]]] = defaultdict(lambda: defaultdict(dict))
    books = set()
    for row in token_rows:
        sys = row["system"]
        val = row["value"]
        norm = row["normalized"]
        books.add(row.get("book") or "")
        bucket = by_sys[sys][val]
        if norm not in bucket:
            bucket[norm] = {
                "t": norm,
                "s": row.get("surface") or norm,
                "c": 0,
                "r": [],
                "b": set(),
                "lic": row.get("license") or "",
            }
        e = bucket[norm]
        e["c"] += 1
        if row.get("book"):
            e["b"].add(row["book"])
        if row.get("ref") and len(e["r"]) < 8 and row["ref"] not in e["r"]:
            e["r"].append(row["ref"])

    phrases_by_sys: dict[str, dict[int, list]] = defaultdict(lambda: defaultdict(list))
    for row in phrase_rows:
        sys = row["system"]
        val = row["value"]
        phrases_by_sys[sys][val].append({
            "t": row["normalized"],
            "s": row.get("surface") or row["normalized"],
            "ref": row.get("ref") or "",
            "book": row.get("book") or "",
            "n": row.get("n", 2),
        })

    shards: dict[str, dict[str, dict[str, list]]] = defaultdict(lambda: defaultdict(dict))
    unique_terms: set[tuple[str, str]] = set()
    total_occ = 0
    indexed_values: dict[str, int] = {}

    for sys, valmap in by_sys.items():
        indexed_values[sys] = len(valmap)
        for val, terms in valmap.items():
            bkey = bucket_for(val, bucket_size)
            items = []
            for norm, e in terms.items():
                unique_terms.add((sys, norm))
                total_occ += e["c"]
                items.append({
                    "t": e["t"],
                    "s": e["s"],
                    "c": e["c"],
                    "r": e["r"][:5],
                    "b": sorted(e["b"])[:6],
                    "lic": e["lic"],
                })
            # Exhaustive: every unique normalized term for this (system, value)
            items.sort(key=lambda x: (-x["c"], x["t"]))
            shards[sys][bkey][str(val)] = items

    phrase_shards: dict[str, dict[str, dict[str, list]]] = defaultdict(lambda: defaultdict(dict))
    phrase_count = 0
    for sys, valmap in phrases_by_sys.items():
        for val, items in valmap.items():
            phrase_count += len(items)
            bkey = bucket_for(val, bucket_size)
            grouped: dict[str, dict] = {}
            for it in items:
                k = it["t"]
                if k not in grouped:
                    grouped[k] = {"t": it["t"], "s": it["s"], "c": 0, "r": [], "b": set(), "n": it["n"]}
                grouped[k]["c"] += 1
                if it["ref"] and len(grouped[k]["r"]) < 5:
                    grouped[k]["r"].append(it["ref"])
                if it["book"]:
                    grouped[k]["b"].add(it["book"])
            out = [{
                "t": g["t"], "s": g["s"], "c": g["c"], "r": g["r"],
                "b": sorted(g["b"])[:6], "n": g["n"],
            } for g in grouped.values()]
            out.sort(key=lambda x: (-x["c"], x["t"]))
            phrase_shards[sys][bkey][str(val)] = out[:50]

    unique_norms = {norm for (_sys, norm) in unique_terms}
    terms_per_system: dict[str, int] = defaultdict(int)
    for sys, norm in unique_terms:
        terms_per_system[sys] += 1
    indexed_term_slots = sum(terms_per_system.values())

    return {
        "shards": shards,
        "phrase_shards": phrase_shards,
        "stats": {
            "unique_terms": len(unique_norms),
            "indexed_term_slots": indexed_term_slots,
            "terms_per_system": dict(terms_per_system),
            "token_occurrences": total_occ,
            "phrases": phrase_count,
            "indexed_values": indexed_values,
            "n_books": len([b for b in books if b]),
            "books": sorted(b for b in books if b),
        },
    }

def write_shards(data: dict, out_dir: Path, bucket_size: int = 100) -> None:
    values_dir = out_dir / "values"
    phrases_dir = out_dir / "phrases"
    values_dir.mkdir(parents=True, exist_ok=True)
    phrases_dir.mkdir(parents=True, exist_ok=True)

    for sys, buckets in data["shards"].items():
        sys_dir = values_dir / sys
        sys_dir.mkdir(parents=True, exist_ok=True)
        for bkey, payload in buckets.items():
            path = sys_dir / f"{bkey}.json"
            path.write_text(json.dumps(payload, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")

    for sys, buckets in data["phrase_shards"].items():
        sys_dir = phrases_dir / sys
        sys_dir.mkdir(parents=True, exist_ok=True)
        for bkey, payload in buckets.items():
            path = sys_dir / f"{bkey}.json"
            path.write_text(json.dumps(payload, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
