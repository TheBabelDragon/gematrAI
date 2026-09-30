# gematrAI

**Client-side interactive gematria explorer**

> Because assigning numbers to letters wasn’t computational enough.

gematrAI is a pure static web application for exploring classical and modern gematria systems. It runs entirely in the browser — no backend, no API keys, no build step, no external AI service.

**Live:** [https://thebabeldragon.github.io/gematrAI/](https://thebabeldragon.github.io/gematrAI/)

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
3. **Reverse lookup** — search the included local corpus by numerical value and system.
4. **Equivalence explorer** — other corpus entries sharing the same total under the active system; click to load.
5. **AI Interpretation panel** — deterministic, client-side observations only. No supernatural claims, no external model.
6. **Shareable state** — query and selected system encoded in the URL hash (`#q=…&sys=…`).
7. **Hebrew UX** — RTL-aware display, correct Unicode preservation, finals handled per system.
8. **Responsive** — desktop, tablet, iPhone.

---

## Local corpus

The reverse-lookup and equivalence features use a **small illustrative corpus** (~120 entries across Hebrew, English, and Greek). It is **not comprehensive**. The UI labels it as the included/local corpus.

Values are computed at runtime from the same calculation engines — no pre-baked totals that can drift from the live mappings.

---

## Architecture

```
index.html
styles.css
app.js                  # UI controller, URL state, rendering
gematria/
  hebrew.js             # Mispar Hechrechi / Gadol / Katan
  greek.js              # Isopsephy
  english.js            # Ordinal / Reduction / Pythagorean
  systems.js            # Registry + calculateAll / calculateOne
data/
  corpus.js             # Local illustrative terms
tests/
  test-gematria.js      # Engine test suite
  runner.html           # Browser test runner
README.md
```

Calculation logic is fully separated from the UI. Additional systems can be registered in `systems.js` without touching the view layer.

---

## Running locally

No build step. Serve the directory over HTTP (ES modules require a server; `file://` may be blocked by browsers):

```bash
# Python
python -m http.server 8080

# Node
npx serve .

# Then open http://localhost:8080
```

### Tests

Open `tests/runner.html` in a browser (same origin / local server) or inspect the console after loading the module. All calculation engines include representative vectors for finals, reduction, and mixed scripts.

---

## GitHub Pages deployment

This repository is configured for **GitHub Pages from the `main` branch root**.

1. Repo → **Settings** → **Pages**
2. Source: **Deploy from a branch**
3. Branch: `main` / `/ (root)`
4. Save

After the first push, the site is available at:

**https://thebabeldragon.github.io/gematrAI/**

No Actions workflow required for a pure static site.

---

## Design notes

Aesthetic: archaeological-computational instrument — dark surface, high-contrast type, subtle grid, monospaced metadata, large numerical readouts. Hebrew / Greek / Latin remain highly readable. Minimal animation.

---

## Limitations

- Local corpus is illustrative only; reverse lookup will not find arbitrary dictionary words.
- Nikud (Hebrew vowel points) and Greek diacritics are ignored for value contribution but preserved in display.
- No server-side persistence or user accounts.
- “AI Interpretation” is rule-based deterministic text, not a language model.

---

## License

MIT — see repository root (or SPDX identifier in package metadata if added later).

---

*Numerical equivalence ≠ demonstrated semantic or historical connection.*
