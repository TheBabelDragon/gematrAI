# Rust calculation engine

**Status:** authoritative computational core for gematrAI.

Engine version is independent of the npm/JS layer. Current: `0.1.0`.

## Workspace

```
rust/
  Cargo.toml                 # workspace
  crates/
    gematra-unicode/         # UTF-8, NFC/NFD, scripts, marks
    gematra-ciphers/         # seven systems, exact JS parity maps
    gematra-core/            # calculation, result contract
    gematra-wasm/            # wasm-bindgen surface
    gematra-cli/             # native `gematra` binary + `gematra_cli` lib
    gematra-fixtures/        # shared deterministic cases
```

## Result contract

Every calculation returns:

| Field | Meaning |
|-------|---------|
| `original_input` | Exact user string (never mutated) |
| `system_id` / `system_name` | Selected cipher |
| `total` | Sum of included letter values |
| `digital_root` | Optional (English systems) |
| `characters[]` | Per-grapheme audit trail |
| `words[]` | Whitespace-split word totals |
| `unicode` | Normalization diagnostics |
| `unsupported[]` | Explicit exclusion reasons |
| `engine_version` | Engine stamp |
| `cipher_registry_version` | Cipher table stamp |
| `unicode_policy_version` | Unicode policy stamp |

## Mandatory regression

```
Input:  herro prease
System: english-ordinal
HERRO  = 64
PREASE = 64
TOTAL  = 128
```

## Design rules

1. Rust is the sole authority for numerical values.
2. JavaScript may format and display; it must not redefine values.
3. No silent JS fallback after migration.
4. Original input is preserved independently of normalized lookup form.
5. Competing cipher definitions are never merged.

## Batch CLI (`gematra batch`)

Preferred interface for corpus builders. One process invocation computes
many tokens so Python does not re-implement cipher tables.

### Invocation

```bash
echo '[{"id":"t1","text":"שלום"},{"text":"abc","systems":["english-ordinal"]}]' \
  | gematra batch --totals-only

gematra batch --input tokens.ndjson --totals-only --pretty

gematra batch --max-input-bytes 33554432 --max-items 100000 --max-text-bytes 8192
```

### Input item schema

| Field | Required | Meaning |
|-------|----------|---------|
| `text` | yes | Surface form to calculate |
| `id` | no | Caller-supplied identifier, echoed in output |
| `systems` | no | System id list. Omitted or `[]` → all applicable. Duplicates deduped. |

### Output schema

Includes `engine_version`, `cipher_registry_version`, `unicode_policy_version`,
`result_count`, and per-item `id`, `original_input`, `normalized_form` (from
`gematra-unicode::analyze`), `totals`, `unsupported_count`, and optional
`calculations`.

- Failure is **atomic**: no partial JSON on stdout.
- `unsupported_count` is the sum of `unsupported.len()` across systems for that item.

### Strict normalization

`--norm` accepts only `nfc`, `nfd`, or `none` (case-insensitive).
Invalid values exit nonzero with a diagnostic; there is **no** silent NFC fallback.
Default when omitted: `nfc`. Applies to `calc`, `all`, and `batch`.

### Bounded input

`gematra batch` reads at most `max_input_bytes + 1` bytes from stdin or `--input`.
Exceeding the limit fails immediately without buffering the full stream and without
emitting partial JSON on stdout. No truncation.

### Safety limits

| Flag | Default | Purpose |
|------|---------|---------|
| `--max-input-bytes` | 32 MiB | Reject oversized stdin/file |
| `--max-items` | 100_000 | Cap batch cardinality |
| `--max-text-bytes` | 8_192 | Cap single token UTF-8 length |

### Compatibility

Unchanged subcommands: `calc`, `all`, `systems`, `fixtures`, `version`.
