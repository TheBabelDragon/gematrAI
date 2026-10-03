# gematrAI

**Client-side interactive gematria explorer**

> Because assigning numbers to letters wasn’t computational enough.

gematrAI is a pure static web application for exploring classical and modern gematria systems. It runs entirely in the browser — no backend, no API keys, no runtime external AI service.

**Live site:** <https://thebabeldragon.github.io/gematrAI/>

**Source:** <https://github.com/TheBabelDragon/gematrAI>

---

## What is gematria?

Gematria is the practice of assigning numerical values to letters and summing those values for words or phrases. Parallel traditions exist in Hebrew (*gematria*), Greek (*isopsephy*), and various Latin/English systems.

**Numerical equivalence is a property of a chosen mapping.** Sharing a total does not, by itself, demonstrate a semantic, historical, mystical, or causal relationship between terms. gematrAI surfaces the arithmetic clearly and keeps interpretation language carefully scoped.

---

## Supported calculation systems

| ID | Name | Mapping notes |
|----|------|---------------|
| `hebrew-standard` | Hebrew Mispar Hechrechi (Standard) | א=1…ת=400; final letters ךםןףץ keep regular values (20/40/50/80/90) |
| `hebrew-gadol` | Hebrew Mispar Gadol | Finals elevated: ך=500, ם=600, ן=700, ף=800, ץ=900 |
| `hebrew-katan` | Hebrew Mispar Katan | Each letter reduced to single digit (1–9) before summing |
| `english-ordinal` | English Ordinal | A=1 … Z=26 (case-insensitive) |
| `english-reduction` | English Reduction | A=1…I=9, J=1…R=9, S=1…Z=8 |
| `english-pythagorean` | English Pythagorean Reduction | Same mapping as Reduction (classic Pythagorean) |
| `greek-isopsephy` | Greek Isopsephy (Standard) | α=1 … ω=800; final sigma ς = 200 |

Exact letter tables:

- [`gematria/hebrew.js`](./gematria/hebrew.js)
- [`gematria/greek.js`](./gematria/greek.js)
- [`gematria/english.js`](./gematria/english.js)
- Registry: [`gematria/systems.js`](./gematria/systems.js)

Variants are named explicitly; competing definitions are never silently mixed.

---

## Features

1. **Live calculator** — Latin, Hebrew Unicode, Greek Unicode, or mixed text. Results update as you type.
2. **Letter-by-letter breakdown** — expandable per system, with copy-to-clipboard for totals.
3. **Corpus Tower** — reverse lookup and equivalence over a large pre-built Hebrew index derived from Sefaria Tanakh text (values computed by gematrAI engines, not Sefaria annotations).
4. **Seed corpus** — illustrative Hebrew / English / Greek terms in [`data/corpus.js`](./data/corpus.js) for instant offline matches (including English Ordinal).
5. **Equivalence explorer** — other corpus terms sharing the same total under the active system; click to load and continue exploring.
6. **AI Interpretation panel** — deterministic, client-side observations only. No supernatural claims, no external model.
7. **Shareable state** — query, selected system, and exploration path encoded in the URL hash (`#q=…&sys=…`).
8. **Hebrew UX** — RTL-aware display, correct Unicode preservation, finals handled per system.
9. **Responsive** — desktop, tablet, phone.

---

## Corpus Tower

Primary reverse-lookup / equivalence data lives under [`corpus/generated/`](./corpus/generated/).

- Source text: [Sefaria](https://www.sefaria.org) export (Tanakh scope) — [Sefaria-Export](https://github.com/Sefaria/Sefaria-Export)
- Values: computed at build time by the same gematria engines used in the browser
- Runtime: the browser only fetches static JSON (`values_low.json`, `values_mid.json`, optional shards). It never calls Sefaria live for search

Full layer notes and build steps: **[corpus/README.md](./corpus/README.md)**

---

## Architecture

```
index.html              # App shell
styles.css / styles-eq.css
app.js                  # UI, URL state, tower fetch, rendering
gematria/               # Calculation engines + systems registry
data/corpus.js          # Seed illustrative terms
corpus/generated/       # Tower artifacts (manifest, value packs)
corpus/fixtures/        # Offline test segments
scripts/corpus/         # Build & normalize helpers (CI / local)
tests/                  # Browser engine tests
.github/workflows/
  static.yml            # Deploy static site to GitHub Pages
  corpus.yml            # Rebuild tower (manual / scheduled)
```

---

## Running locally

No frontend build step. Serve the directory over HTTP (ES modules require a server):

```bash
python3 -m http.server 8080
# open http://localhost:8080
```

### Engine tests

Open [`tests/runner.html`](./tests/runner.html) under the same local server.

### Corpus pipeline tests

```bash
python3 scripts/tests/test_corpus.py -v
python3 scripts/corpus/build_tower.py   # rebuild from fixtures
```

---

## GitHub Pages

Deployed by [`.github/workflows/static.yml`](./.github/workflows/static.yml) on every push to `main`.

**Canonical public URL:** <https://thebabeldragon.github.io/gematrAI/>

There is no custom domain / `CNAME` in this repo. That `github.io` URL is the only supported live endpoint.

---

## Limitations

- Tower coverage is the checked-in Tanakh index under `corpus/generated/`, plus the seed lexicon — not every word in every language
- Nikud and Greek diacritics are ignored for value contribution but preserved in display where present
- No server-side persistence or user accounts
- “AI Interpretation” is rule-based deterministic text, not a language model

---

## License

[MIT](./LICENSE)

---

*Numerical equivalence ≠ demonstrated semantic or historical connection.*
