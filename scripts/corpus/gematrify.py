"""Hebrew gematria engines (Python mirror of gematria/hebrew.js)."""

from __future__ import annotations

HEBREW_STANDARD = {
    "א": 1, "ב": 2, "ג": 3, "ד": 4, "ה": 5, "ו": 6, "ז": 7, "ח": 8, "ט": 9,
    "י": 10, "כ": 20, "ך": 20, "ל": 30, "מ": 40, "ם": 40, "נ": 50, "ן": 50,
    "ס": 60, "ע": 70, "פ": 80, "ף": 80, "צ": 90, "ץ": 90, "ק": 100, "ר": 200,
    "ש": 300, "ת": 400,
}

HEBREW_GADOL = {
    **HEBREW_STANDARD,
    "ך": 500, "ם": 600, "ן": 700, "ף": 800, "ץ": 900,
}


def digital_root(n: int) -> int:
    if n == 0:
        return 0
    r = n % 9
    return 9 if r == 0 else r


def value_standard(token: str) -> int:
    return sum(HEBREW_STANDARD.get(ch, 0) for ch in token)


def value_gadol(token: str) -> int:
    return sum(HEBREW_GADOL.get(ch, 0) for ch in token)


def value_katan(token: str) -> int:
    total = 0
    for ch in token:
        v = HEBREW_STANDARD.get(ch)
        if v is not None:
            total += digital_root(v)
    return total


def systems_for_token(normalized: str) -> dict[str, int]:
    return {
        "hebrew_standard": value_standard(normalized),
        "hebrew_gadol": value_gadol(normalized),
        "hebrew_katan": value_katan(normalized),
    }
