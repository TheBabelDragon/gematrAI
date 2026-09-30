"""Segment tokenization helper (avoids name clash with tokenize modules)."""
from __future__ import annotations
from normalize import tokenize as _tokenize

def tokenize_segment(text: str, language: str = "he") -> list[dict]:
    return _tokenize(text, language)
