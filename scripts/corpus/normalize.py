"""Text normalization for gematrAI corpus tower.

Preserves original surface forms; produces calculation-ready forms.
Hebrew: NFKC, strip niqqud/cantillation, keep final letters as-is.
"""
from __future__ import annotations
import re
import unicodedata

# Hebrew marks to strip for calculation (niqqud + cantillation + some punctuation)
_HEBREW_MARKS = re.compile(
    r"[\u0591-\u05C7\u05F3\u05F4\u200E\u200F\u202A-\u202E\uFEFF]"
)
_WS = re.compile(r"\s+")
_PUNCT = re.compile(r"[^\w\u0590-\u05FF\u0370-\u03FF\u1F00-\u1FFF'-]+", re.UNICODE)

# Final forms map to non-final for some systems; we KEEP finals for calculation
# matching gematrAI JS engines which assign finals their standard values.

def strip_marks(text: str) -> str:
    if not text:
        return ""
    t = unicodedata.normalize("NFKC", text)
    t = _HEBREW_MARKS.sub("", t)
    return t

def for_display(text: str) -> str:
    return strip_marks(text).strip()

def for_calculation_hebrew(text: str) -> str:
    t = strip_marks(text)
    t = t.replace("\u05BE", "")  # maqaf
    t = _WS.sub("", t)
    return t

def for_calculation_greek(text: str) -> str:
    t = unicodedata.normalize("NFKC", text or "")
    t = "".join(c for c in t if not unicodedata.combining(c))
    return t.lower()

def for_calculation_english(text: str) -> str:
    t = unicodedata.normalize("NFKC", text or "")
    return re.sub(r"[^a-zA-Z]", "", t).lower()

def tokenize(text: str, language: str = "he") -> list[dict]:
    """Tokenize a segment into surface/normalized pairs with positions."""
    if not text:
        return []
    raw = strip_marks(text)
    # Split on whitespace and common separators; keep Hebrew/Greek/Latin runs
    parts = re.findall(r"[\u0590-\u05FF\u0370-\u03FF\u1F00-\u1FFFa-zA-Z']+", raw)
    tokens = []
    pos = 0
    for p in parts:
        if not p:
            continue
        lang = language or "he"
        if lang in ("he", "hebrew", "Hebrew"):
            norm = for_calculation_hebrew(p)
        elif lang in ("el", "greek", "Greek"):
            norm = for_calculation_greek(p)
        else:
            norm = for_calculation_english(p)
        if not norm:
            continue
        tokens.append({"surface": p, "normalized": norm, "position": pos})
        pos += 1
    return tokens
