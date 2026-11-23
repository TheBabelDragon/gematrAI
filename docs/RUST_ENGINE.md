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
# JSON array on stdin
echo '[{"id":"t1","text":"שלום"},{"text":"abc","systems":["english-ordinal"]}]' \
  | gematra batch --totals-only

# NDJSON file
gematra batch --input tokens.ndjson --totals-only --pretty

# Limits (defaults shown)
gematra batch --max-input-bytes 33554432 --max-items 100000 --max-text-bytes 8192
```

### Input item schema

| Field | Required | Meaning |
|-------|----------|---------|
| `text` | yes | Surface form to calculate |
| `id` | no | Caller-supplied identifier, echoed in output |
| `systems` | no | System id list. Omitted or `[]` → all applicable systems. Duplicates deduped. |

### Output schema

```json
{
  "engine_version": "0.1.0",
  "cipher_registry_version": "1",
  "unicode_policy_version": "1",
  "result_count": 2,
  "results": [
    {
      "id": "t1",
      "original_input": "שלום",
      "normalized_form": "שלום",
      "totals": {
        "hebrew-standard": 376,
        "hebrew-gadol": 936,
        "hebrew-katan": 16
      },
      "unsupported_count": 0,
      "calculations": null
    }
  ]
}
```

- `normalized_form` is produced by the same `gematra-unicode` analysis path used
  for calculations (NFC by default). It is **not** a second normalization stack.
- `--totals-only` omits the full `calculations` array for smaller corpus payloads.
- Failure is **atomic**: parse errors, invalid system ids, or limit breaches abort
  with exit code 1/2 and no partial results on stdout.

### Safety limits

| Flag | Default | Purpose |
|------|---------|---------|
| `--max-input-bytes` | 32 MiB | Reject oversized stdin/file |
| `--max-items` | 100_000 | Cap batch cardinality |
| `--max-text-bytes` | 8_192 | Cap single token UTF-8 length |

No silent truncation.

### Compatibility

Unchanged subcommands: `calc`, `all`, `systems`, `fixtures`, `version`.
