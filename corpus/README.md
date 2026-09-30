# Corpus Tower

Progressively expanding, **non-user-editable** textual/gematria corpus for gematrAI.

## Layers

| Level | Name | Description |
|-------|------|-------------|
| 0 | Source | Sefaria text + metadata (fetched at build time) |
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

## Scope (tower 0.2)

Full **Tanakh** Hebrew (Torah + Prophets + Writings) — 39 books.

## Build

```bash
python3 scripts/tests/test_corpus.py -v
python3 scripts/corpus/build_tower.py
# Or: Actions → Corpus Tower → Run workflow
```

The browser does **not** call Sefaria at runtime for corpus search.
