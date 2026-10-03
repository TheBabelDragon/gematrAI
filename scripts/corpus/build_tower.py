#!/usr/bin/env python3
"""Corpus Tower builder for gematrAI.

Produces token rows + phrase rows from segment fixtures (or future Sefaria export),
then writes sharded reverse indexes under corpus/generated/.

Offline-first: fixtures always work. Full Tanakh rebuild needs a Sefaria export
checked in or fetched separately (see corpus/README.md).
"""
from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))

from normalize import tokenize  # noqa: E402
from gematrify import systems_for_token  # noqa: E402
from build_indexes import build_indexes, write_shards  # noqa: E402


def process_segments(
    segments: list[dict],
    build_id: str,
    phrase_max: int = 2,
) -> tuple[list[dict], list[dict], int]:
    """Tokenize segments and emit per-system value rows + optional bigrams."""
    tokens: list[dict] = []
    phrases: list[dict] = []
    n = 0
    for seg in segments:
        text = seg.get("text_original") or seg.get("text") or ""
        lang = seg.get("language") or "he"
        ref = seg.get("ref") or ""
        book = seg.get("book") or ""
        license_ = seg.get("license") or ""
        source_url = seg.get("sourceUrl") or ""
        toks = tokenize(text, lang)
        if not toks:
            continue
        n += 1
        norms: list[str] = []
        surfaces: list[str] = []
        for t in toks:
            norm = t["normalized"]
            surface = t.get("surface") or norm
            norms.append(norm)
            surfaces.append(surface)
            for sys_id, val in systems_for_token(norm, lang).items():
                tokens.append({
                    "normalized": norm,
                    "surface": surface,
                    "system": sys_id,
                    "value": int(val),
                    "ref": ref,
                    "book": book,
                    "license": license_,
                    "sourceUrl": source_url,
                    "build": build_id,
                })
        # contiguous bigrams (Hebrew-friendly phrase seed)
        if phrase_max >= 2 and len(norms) >= 2:
            for i in range(len(norms) - 1):
                pn = norms[i] + " " + norms[i + 1]
                ps = surfaces[i] + " " + surfaces[i + 1]
                for sys_id, val in systems_for_token(norms[i], lang).items():
                    # phrase value = sum of component values under same system
                    v2 = systems_for_token(norms[i + 1], lang).get(sys_id)
                    if v2 is None:
                        continue
                    phrases.append({
                        "normalized": pn,
                        "surface": ps,
                        "system": sys_id,
                        "value": int(val) + int(v2),
                        "ref": ref,
                        "book": book,
                        "n": 2,
                    })
    return tokens, phrases, n

def write_manifest(
    out_dir: Path,
    build_id: str,
    stats: dict,
    systems: list[str],
    tower: str = "0.2",
) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    manifest = {
        "buildId": build_id,
        "valuesBase": "./corpus/generated/values/",
        "phrasesBase": "./corpus/generated/phrases/",
        "bucketSize": 100,
        "systems": systems,
        "stats": {
            "unique_terms": stats.get("unique_terms", 0),
            "indexed_term_slots": stats.get("indexed_term_slots", 0),
            "terms_per_system": stats.get("terms_per_system", {}),
            "token_occurrences": stats.get("token_occurrences", 0),
            "phrases": stats.get("phrases", 0),
            "indexed_values": stats.get("indexed_values", {}),
            "n_books": stats.get("n_books", 0),
            "segments": stats.get("segments", 0),
            "texts": stats.get("n_books", 0),
        },
        "tower": tower,
        "name": "gematrAI Corpus Tower",
        "exhaustive": False,
        "disclaimer": (
            "Gematria values are computed by gematrAI from selected text segments. "
            "Numerical equivalence does not establish semantic, theological, historical, "
            "or causal relationships."
        ),
        "lowValuesUrl": "./corpus/generated/values_low.json",
        "midValuesUrl": "./corpus/generated/values_mid.json",
    }
    (out_dir / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    meta_dir = out_dir / "metadata"
    meta_dir.mkdir(parents=True, exist_ok=True)
    build_meta = {
        "tower": tower,
        "name": "gematrAI Corpus Tower",
        "source": "fixtures+local",
        "scope": ["fixtures"],
        "build": build_id,
        "builtAt": datetime.now(timezone.utc).isoformat(),
        "systems": systems,
        "stats": {
            "segments": stats.get("segments", 0),
            "unique_tokens": stats.get("unique_terms", 0),
            "token_occurrences": stats.get("token_occurrences", 0),
            "n_books": stats.get("n_books", 0),
            "books": stats.get("books", []),
        },
        "disclaimer": manifest["disclaimer"],
        "sefaria": {
            "docs": "https://developers.sefaria.org/reference/get-v3-texts",
            "export": "https://github.com/Sefaria/Sefaria-Export",
        },
    }
    (meta_dir / "build.json").write_text(
        json.dumps(build_meta, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )


def pack_low_mid(shards: dict, out_dir: Path, low_max: int = 99, mid_max: int = 999) -> None:
    """Optional compact packs for common value ranges (browser fast path)."""
    low: dict = {}
    mid: dict = {}
    for sys, buckets in shards.items():
        low[sys] = {}
        mid[sys] = {}
        for _bkey, payload in buckets.items():
            for val_str, items in payload.items():
                try:
                    v = int(val_str)
                except ValueError:
                    continue
                if v <= low_max:
                    low[sys][val_str] = items
                elif v <= mid_max:
                    mid[sys][val_str] = items
    (out_dir / "values_low.json").write_text(
        json.dumps(low, ensure_ascii=False, separators=(",", ":")), encoding="utf-8"
    )
    (out_dir / "values_mid.json").write_text(
        json.dumps(mid, ensure_ascii=False, separators=(",", ":")), encoding="utf-8"
    )


def main() -> int:
    ap = argparse.ArgumentParser(description="Build gematrAI Corpus Tower indexes")
    ap.add_argument(
        "--fixtures",
        type=Path,
        default=ROOT / "corpus" / "fixtures" / "tower_segments.json",
        help="Segment JSON (default: corpus fixtures)",
    )
    ap.add_argument(
        "--out",
        type=Path,
        default=ROOT / "corpus" / "generated",
        help="Output directory",
    )
    ap.add_argument("--torah-only", action="store_true", help="Reserved for future Sefaria scope")
    ap.add_argument("--phrase-max", type=int, default=2)
    ap.add_argument("--write-packs", action="store_true", default=True)
    args = ap.parse_args()

    if not args.fixtures.exists():
        print(f"Missing fixtures: {args.fixtures}", file=sys.stderr)
        return 1

    segments = json.loads(args.fixtures.read_text(encoding="utf-8"))
    build_id = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    tokens, phrases, n_seg = process_segments(segments, build_id, phrase_max=args.phrase_max)
    data = build_indexes(tokens, phrases)
    data["stats"]["segments"] = n_seg

    systems = sorted(data["stats"].get("terms_per_system", {}).keys()) or [
        "hebrew-standard",
        "hebrew-gadol",
        "hebrew-katan",
    ]

    write_shards(data, args.out)
    write_manifest(args.out, build_id, data["stats"], systems)
    if args.write_packs:
        pack_low_mid(data["shards"], args.out)

    print(json.dumps({"buildId": build_id, "stats": data["stats"]}, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
