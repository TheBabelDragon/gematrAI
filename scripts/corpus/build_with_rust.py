#!/usr/bin/env python3
"""Orchestrate corpus tower build via the Rust gematra CLI.

Python responsibilities only:
  - locate fixture segments and seed lexicon
  - invoke `gematra corpus build`
  - validate generated artifacts exist
  - no cipher tables, no numerical calculation, no competing normalization
"""
from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_FIXTURES = ROOT / "corpus" / "fixtures" / "tower_segments.json"
DEFAULT_SEED = ROOT / "data" / "seed" / "lexicon.json"
DEFAULT_OUT = ROOT / "corpus" / "generated"


def find_gematra() -> Path:
    candidates = [
        ROOT / "rust" / "target" / "release" / "gematra",
        ROOT / "rust" / "target" / "debug" / "gematra",
        Path(shutil.which("gematra") or ""),
    ]
    for c in candidates:
        if c and c.is_file() and c.stat().st_mode & 0o111:
            return c
    raise SystemExit(
        "gematra binary not found. Build with: "
        "cd rust && cargo build -p gematra-cli --release"
    )


def load_fixtures(path: Path) -> list[dict]:
    raw = json.loads(path.read_text(encoding="utf-8"))
    out = []
    for f in raw:
        text = f.get("text_original") or f.get("text") or ""
        if not text.strip():
            continue
        out.append(
            {
                "ref": f.get("ref"),
                "book": f.get("book"),
                "language": f.get("language"),
                "text": text,
                "license": f.get("license"),
                "source_url": f.get("sourceUrl") or f.get("source_url"),
                "kind": f.get("kind") or "fixture",
            }
        )
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description="Build corpus via Rust engine")
    ap.add_argument("--fixtures", type=Path, default=DEFAULT_FIXTURES)
    ap.add_argument("--seed", type=Path, default=DEFAULT_SEED)
    ap.add_argument("--out", type=Path, default=DEFAULT_OUT)
    ap.add_argument("--gematra", type=Path, default=None)
    ap.add_argument("--phrase-max", type=int, default=2)
    ap.add_argument("--build-id", type=str, default=None)
    args = ap.parse_args()

    gematra = args.gematra or find_gematra()
    if not args.fixtures.is_file():
        raise SystemExit(f"fixtures missing: {args.fixtures}")
    if not args.seed.is_file():
        raise SystemExit(f"seed lexicon missing: {args.seed}")

    segments = load_fixtures(args.fixtures)
    tmp = tempfile.NamedTemporaryFile(
        mode="w",
        suffix=".json",
        prefix="gematra_segments_",
        delete=False,
        encoding="utf-8",
    )
    try:
        json.dump(segments, tmp, ensure_ascii=False, indent=2)
        tmp.close()
        seg_path = Path(tmp.name)
    except Exception:
        tmp.close()
        Path(tmp.name).unlink(missing_ok=True)
        raise

    build_id = args.build_id or datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    args.out.mkdir(parents=True, exist_ok=True)

    cmd = [
        str(gematra),
        "corpus",
        "build",
        "--input",
        str(seg_path),
        "--seed",
        str(args.seed),
        "--out",
        str(args.out),
        "--phrase-max",
        str(args.phrase_max),
        "--build-id",
        build_id,
    ]
    print("running:", " ".join(cmd), file=sys.stderr)
    proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.returncode != 0:
        sys.stderr.write(proc.stderr or proc.stdout or "build failed\n")
        try:
            seg_path.unlink(missing_ok=True)
        except Exception:
            pass
        return proc.returncode
    try:
        manifest = json.loads(proc.stdout)
    except json.JSONDecodeError:
        sys.stderr.write(proc.stdout)
        sys.stderr.write(proc.stderr)
        try:
            seg_path.unlink(missing_ok=True)
        except Exception:
            pass
        return 1

    required = [
        args.out / "manifest.json",
        args.out / "values_low.json",
        args.out / "values_mid.json",
        args.out / "metadata" / "build.json",
    ]
    missing = [str(p) for p in required if not p.is_file()]
    if missing:
        print("missing artifacts:", missing, file=sys.stderr)
        try:
            seg_path.unlink(missing_ok=True)
        except Exception:
            pass
        return 1

    stats = manifest.get("stats") or {}
    print(
        f"OK tower={manifest.get('tower')} segments={stats.get('segments')} "
        f"unique_terms={stats.get('unique_terms')} "
        f"seed_terms={stats.get('seed_terms')} "
        f"engine={manifest.get('engine_version')}",
        file=sys.stderr,
    )
    print(json.dumps(manifest, ensure_ascii=False, indent=2))
    try:
        seg_path.unlink(missing_ok=True)
    except Exception:
        pass
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
