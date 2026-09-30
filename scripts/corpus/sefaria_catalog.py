"""Fetch and filter Sefaria books.json catalog."""
from __future__ import annotations
import json
import urllib.request

CATALOG_URL = "https://raw.githubusercontent.com/Sefaria/Sefaria-Export/master/books.json"

def fetch_catalog(url: str = CATALOG_URL, timeout: int = 60) -> list:
    req = urllib.request.Request(url, headers={"User-Agent": "gematrAI-corpus/0.2"})
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read().decode("utf-8"))

def filter_by_categories(catalog: list, category_path: list[str], language: str = "Hebrew") -> list:
    out = []
    for book in catalog:
        cats = book.get("categories") or []
        if len(cats) < len(category_path):
            continue
        if cats[: len(category_path)] != category_path:
            continue
        # Prefer Hebrew versions when language is Hebrew
        out.append(book)
    return out
