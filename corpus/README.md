# Corpus Tower

Progressively expanding, **non-user-editable** textual/gematria corpus for gematrAI.

## Layers

| Level | Name | Description |
|-------|------|-------------|
| 0 | Source | Sefaria text + metadata (fetched at build time) or local fixtures |
| 1 | Tokens | Normalized Hebrew tokens with source refs |
| 2 | Gematria | Values from gematrAI engines (not from Sefaria) |
| 3 | Reverse indexes | number → matching tokens |
| 4 | Occurrences | token → textual references |
| 5 | Equivalence graph | value clusters (computational only) |

**Numerical equivalence is a property of the chosen mapping.** It does not establish semantic, theological, historical, or causal relationships. Sefaria supplies text; gematrAI supplies the arithmetic.

## Source

Initial provider: **[Sefaria](https://www.sefaria.org)**  
- Export: https://github.com/Sefaria/Sefaria-Export  
- Preferred version: *Tanach with Text Only* (Public Domain)

Offline fixtures live in `corpus/fixtures/tower_segments.json` so tests and local builds always work.

## Scope

Checked-in `corpus/generated/` may contain a larger pre-built Tanakh index. Local rebuilds default to fixtures until a full export is provided to the builder.

## Build

```bash
python3 scripts/tests/test_corpus.py -v
python3 scripts/corpus/build_tower.py
# Or: Actions → Corpus Tower → Run workflow
```

Outputs under `corpus/generated/`:

- `manifest.json` — browser entry point
- `values/` — sharded reverse indexes
- `values_low.json` / `values_mid.json` — compact packs for common values
- `metadata/build.json` — build stats

The browser does **not** call Sefaria at runtime for corpus search.
