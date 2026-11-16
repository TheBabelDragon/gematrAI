# Unicode policy

**Policy version:** `1`

## Principles

1. Original input is sacred.
2. Normalization is explicit (`none` | `nfc` | `nfd`). Default NFC.
3. No NFKC.
4. No silent transliteration between scripts.
5. Code points ≠ gematria values.
6. UTF-8 bytes are never treated as characters.
7. Unsupported characters are reported with reasons.

Hebrew niqqud ignored for value, kept for display. Greek combining marks stripped for lookup only.
