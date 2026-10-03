"""Gematria engines for corpus tower (mirrors gematrAI JS systems)."""
from __future__ import annotations

# Standard Mispar Hechrechi (finals = non-final values)
_HEB_STD = {
    "\u05D0": 1, "\u05D1": 2, "\u05D2": 3, "\u05D3": 4, "\u05D4": 5,
    "\u05D5": 6, "\u05D6": 7, "\u05D7": 8, "\u05D8": 9, "\u05D9": 10,
    "\u05DB": 20, "\u05DA": 20, "\u05DC": 30, "\u05DE": 40, "\u05DD": 40,
    "\u05E0": 50, "\u05DF": 50, "\u05E1": 60, "\u05E2": 70, "\u05E4": 80,
    "\u05E3": 80, "\u05E6": 90, "\u05E5": 90, "\u05E7": 100, "\u05E8": 200,
    "\u05E9": 300, "\u05EA": 400,
}
# Mispar Gadol — finals as 500-900
_HEB_GADOL = dict(_HEB_STD)
_HEB_GADOL.update({
    "\u05DA": 500, "\u05DD": 600, "\u05DF": 700, "\u05E3": 800, "\u05E5": 900,
})

def digital_root(n: int) -> int:
    n = abs(int(n))
    while n > 9:
        n = sum(int(d) for d in str(n))
    return n

def value_hebrew_standard(text: str) -> int:
    return sum(_HEB_STD.get(c, 0) for c in text)

def value_hebrew_gadol(text: str) -> int:
    return sum(_HEB_GADOL.get(c, 0) for c in text)

def value_hebrew_katan(text: str) -> int:
    return digital_root(value_hebrew_standard(text))

# Classical isopsephy — final sigma ς = 200 (matches gematria/greek.js)
_GREEK = {
    "\u03B1": 1, "\u03B2": 2, "\u03B3": 3, "\u03B4": 4, "\u03B5": 5,
    "\u03B6": 7, "\u03B7": 8, "\u03B8": 9,
    "\u03B9": 10, "\u03BA": 20, "\u03BB": 30, "\u03BC": 40, "\u03BD": 50,
    "\u03BE": 60, "\u03BF": 70, "\u03C0": 80, "\u03C1": 100,
    "\u03C3": 200, "\u03C2": 200, "\u03C4": 300,
    "\u03C5": 400, "\u03C6": 500, "\u03C7": 600, "\u03C8": 700, "\u03C9": 800,
}

def value_greek_isopsephy(text: str) -> int:
    return sum(_GREEK.get(c, 0) for c in text.lower())

def value_english_ordinal(text: str) -> int:
    return sum((ord(c) - 96) for c in text.lower() if "a" <= c <= "z")

def value_english_reduction(text: str) -> int:
    """Per-letter reduction A=1…I=9, J=1… — matches gematria/english.js."""
    total = 0
    for c in text.lower():
        if "a" <= c <= "z":
            total += ((ord(c) - 96 - 1) % 9) + 1
    return total

def value_english_pythagorean(text: str) -> int:
    return value_english_reduction(text)

def systems_for_token(normalized: str, language: str) -> dict[str, int]:
    lang = (language or "he").lower()
    out = {}
    if lang in ("he", "hebrew") or any("\u0590" <= c <= "\u05FF" for c in normalized):
        out["hebrew-standard"] = value_hebrew_standard(normalized)
        out["hebrew-gadol"] = value_hebrew_gadol(normalized)
        out["hebrew-katan"] = value_hebrew_katan(normalized)
    elif lang in ("el", "greek") or any("\u0370" <= c <= "\u03FF" for c in normalized):
        out["greek-isopsephy"] = value_greek_isopsephy(normalized)
    else:
        out["english-ordinal"] = value_english_ordinal(normalized)
        out["english-reduction"] = value_english_reduction(normalized)
        out["english-pythagorean"] = value_english_pythagorean(normalized)
    return out
