# Migration: JS engines → Rust/WASM authority

## Base

- Branch: `feat/rust-unicode-engine`
- Base main SHA: `fcba0cb916ba070a83053a26eadb9c0d9d174d45`

## What changed

| Layer | Before | After |
|-------|--------|-------|
| Calculation | `gematria/*.js` | `gematra-core` (Rust) via WASM |
| Unicode | ad-hoc JS `normalize('NFD')` | `gematra-unicode` policy v1 |
| Cipher tables | JS objects | `gematra-ciphers` registry v1 |
| Dashboard | direct imports of JS engines | `js/engine-adapter.js` → WASM |
| Corpus | Python + JS parity assumed | CLI `gematra` as authority |

## What is preserved

- All seven system IDs and exact letter maps
- Final Hebrew letter behavior (standard vs gadol)
- Greek final sigma = 200
- Niqqud ignored for value, kept for display
- Greek diacritics stripped for lookup only
- Live calculator, breakdowns, Corpus Tower, reverse lookup, equivalence explorer, URL state, responsive layout
- Static GitHub Pages deployment (no backend)

## Parity status

Mandatory fixture suite (`gematra fixtures`) passes, including:

```
herro prease / english-ordinal → 128
```

Hebrew shalom standard → 376, finals, Greek alpha/final sigma, empty/whitespace cases covered in Rust unit tests.

## Remaining work before merge

1. Produce committed WASM artifacts (or CI-generated) under `wasm/`.
2. Wire `app.js` to call `initEngine()` + `calculateAllForDashboard` and show init failure UI.
3. Keep legacy `gematria/*.js` until browser integration tests confirm parity, then remove.
4. Point corpus build scripts at `gematra` CLI where values are computed.
5. Extend CI workflow for Rust + wasm-bindgen + browser tests.

## Non-goals of this branch

- Dashboard visual redesign
- New cipher systems
- Server-side API
