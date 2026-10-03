# gematrAI

**Client-side interactive gematria explorer**

> Because assigning numbers to letters wasn’t computational enough.

gematrAI is a pure static web application for exploring classical and modern gematria systems. It runs entirely in the browser — no backend, no API keys, no runtime external AI service.

**Live:** [https://thebabeldragon.github.io/gematrAI/](https://thebabeldragon.github.io/gematrAI/)

Custom-domain experiments (e.g. gematrai.com) were reverted; the canonical public URL is the GitHub Pages default above. No `CNAME` file is present in the repo.

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

Exact letter tables live in:

- `gematria/hebrew.js`
- `gematria/greek.js`
- `gematria/english.js`

Variants are named explicitly; competing definitions are never silently mixed.

---

## Features

1. **Live calculator** — Latin, Hebrew Unicode, Greek Unicode, or mixed text. Results update as you type.
2. **Letter-by-letter breakdown** — expandable per system, with copy-to-clipboard for totals.
3. **Corpus Tower** — reverse lookup and equivalence over a large pre-built Hebrew index derived from Sefaria Tanakh text (values computed by gematrAI engines, not Sefaria annotations).
4. **Seed corpus** — small illustrative Hebrew / English / Greek terms in `data/corpus.js` for instant offline matches.
5. **Equivalence explorer** — other corpus terms sharing the same total under the active system; click to load and continue exploring.
6. **AI Interpretation panel** — deterministic, client-side observations only. No supernatural claims, no external model.
7. **Shareable state** — query, selected system, and exploration path encoded in the URL hash (`#q=…&sys=…`).
8. **Hebrew UX** — RTL-aware display, correct Unicode preservation, finals handled per system.
9. **Responsive** — desktop, tablet, iPhone.

---

## Corpus Tower

The primary reverse-lookup / equivalence data is the **Corpus Tower** under `corpus/generated/`.

- Source text: Sefaria export (Tanakh scope).
- Values: computed at build time by the same gematria engines used in the browser.
- Runtime: the browser only fetches static JSON shards (`values_low.json`, `values_mid.json`, and optional per-bucket files). It never calls Sefaria live for search.

See `corpus/README.md` for layers, disclaimer, and build notes.

A small **seed corpus** (`data/corpus.js`) remains for quick English/Greek/Hebrew examples and offline fallback.

---

## Architecture

```
index.html              # App shell
styles.css / styles-eq.css
app.js                  # UI, URL state, tower fetch, rendering
gematria/
  hebrew.js / greek.js / english.js
  systems.js            # Registry + calculateAll / calculateOne
data/
  corpus.js             # Seed illustrative terms
corpus/
  generated/            # Tower artifacts (manifest, value packs)
  fixtures/             # Offline test segments
scripts/corpus/         # Build & normalize helpers (CI / local)
tests/
  test-gematria.js + runner.html
.github/workflows/
  static.yml            # Deploy static site to GitHub Pages
  corpus.yml            # Rebuild tower (manual / scheduled)
```

Calculation logic is fully separated from the UI. Additional systems can be registered in `systems.js` without touching the view layer.

---

## Running locally

No frontend build step. Serve the directory over HTTP (ES modules require a server; `file://` may be blocked):

```bash
python -m http.server 8080
# or: npx serve .
# then open http://localhost:8080
```

### Engine tests

Open `tests/runner.html` under the same local server.

### Corpus pipeline tests

```bash
python3 scripts/tests/test_corpus.py -v
```

(Requires the corpus helper modules under `scripts/corpus/`.)

---

## GitHub Pages deployment

Deployment is handled by **Actions** (`.github/workflows/static.yml`):

- Trigger: push to `main` or manual `workflow_dispatch`
- Artifact: the repository root (static assets + generated corpus)
- Target: GitHub Pages environment

Public URL:

**https://thebabeldragon.github.io/gematrAI/**

A previous custom-domain `CNAME` (gematrai.com) was removed to restore the default Pages URL and avoid leftover DNS / HTML path issues. Re-adding a custom domain requires:

1. DNS records at the registrar pointing to GitHub Pages
2. A root `CNAME` file containing the domain name
3. Enabling the custom domain under Repo → Settings → Pages

Until then, use the `github.io` URL only.

---

## Design notes

Aesthetic: archaeological-computational instrument — dark surface, high-contrast type, subtle grid, monospaced metadata, large numerical readouts. Hebrew / Greek / Latin remain highly readable. Minimal animation.

---

## Limitations

- Corpus Tower coverage is the indexed Sefaria Tanakh export for the build that is checked in; it is not a live dictionary of every possible word.
- Nikud (Hebrew vowel points) and Greek diacritics are ignored for value contribution but preserved in display where present.
- No server-side persistence or user accounts.
- “AI Interpretation” is rule-based deterministic text, not a language model.
- Full tower rebuild scripts may require additional local setup; pre-built artifacts under `corpus/generated/` are what the live site uses.

---

## License

MIT — see repository root (or SPDX identifier in package metadata if added later).

---

*Numerical equivalence ≠ demonstrated semantic or historical connection.*
