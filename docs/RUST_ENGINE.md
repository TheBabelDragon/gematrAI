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
    gematra-cli/             # native `gematra` binary
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
