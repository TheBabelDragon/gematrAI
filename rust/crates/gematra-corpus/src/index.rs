//! Aggregation and reverse-index structures.
use std::collections::{BTreeMap, BTreeSet};

use crate::models::{CorpusBuildConfig, IndexItem, PhraseRecord, TokenRecord};

pub(crate) fn bucket_for(value: u32, size: u32) -> String {
    let size = size.max(1);
    let b = (value / size) * size;
    format!("{b:04}-{end:04}", end = b + size - 1)
}

/// Aggregate token rows into reverse index structure.
pub fn build_index(
    token_rows: &[TokenRecord],
    phrase_rows: &[PhraseRecord],
    cfg: &CorpusBuildConfig,
) -> BuiltIndex {
    let mut by_sys: BTreeMap<String, BTreeMap<u32, BTreeMap<String, Agg>>> = BTreeMap::new();
    let mut books = BTreeSet::new();
    for row in token_rows {
        books.insert(row.book.clone());
        let entry = by_sys
            .entry(row.system.clone())
            .or_default()
            .entry(row.value)
            .or_default()
            .entry(row.normalized.clone())
            .or_insert_with(|| Agg {
                surface: row.surface.clone(),
                count: 0,
                refs: Vec::new(),
                books: BTreeSet::new(),
                license: row.license.clone(),
                kinds: BTreeSet::new(),
            });
        entry.count += 1;
        if !row.book.is_empty() {
            entry.books.insert(row.book.clone());
        }
        if !row.r#ref.is_empty()
            && entry.refs.len() < cfg.max_refs_per_term
            && !entry.refs.contains(&row.r#ref)
        {
            entry.refs.push(row.r#ref.clone());
        }
        if !row.kind.is_empty() {
            entry.kinds.insert(row.kind.clone());
        }
        if row.kind != "seed" && !row.surface.is_empty() {
            entry.surface = row.surface.clone();
        }
        if entry.license.is_empty() && !row.license.is_empty() {
            entry.license = row.license.clone();
        }
    }

    // Provenance stats from aggregated kinds (not mutually exclusive)
    let mut seed_terms_set: BTreeSet<(String, String)> = BTreeSet::new();
    let mut text_terms_set: BTreeSet<(String, String)> = BTreeSet::new();
    let mut multi_kind: u32 = 0;
    for (sys, valmap) in &by_sys {
        for terms in valmap.values() {
            for (norm, agg) in terms {
                let key = (sys.clone(), norm.clone());
                if agg.kinds.contains("seed") {
                    seed_terms_set.insert(key.clone());
                }
                if agg.kinds.iter().any(|k| k == "fixture" || k == "text") {
                    text_terms_set.insert(key);
                }
                if agg.kinds.len() > 1 {
                    multi_kind += 1;
                }
            }
        }
    }
    let seed_terms = seed_terms_set.len() as u32;
    let text_terms = text_terms_set.len() as u32;
    let _multi_kind_terms = multi_kind;

    let mut phrases_by_sys: BTreeMap<String, BTreeMap<u32, Vec<serde_json::Value>>> =
        BTreeMap::new();
    for row in phrase_rows {
        phrases_by_sys
            .entry(row.system.clone())
            .or_default()
            .entry(row.value)
            .or_default()
            .push(serde_json::json!({
                "t": row.normalized,
                "s": row.surface,
                "ref": row.r#ref,
                "book": row.book,
                "n": row.n,
            }));
    }

    let mut value_shards: BTreeMap<String, BTreeMap<String, Vec<IndexItem>>> = BTreeMap::new();
    let mut phrase_shards: BTreeMap<String, BTreeMap<String, Vec<serde_json::Value>>> =
        BTreeMap::new();
    let mut unique_terms: BTreeSet<(String, String)> = BTreeSet::new();
    let mut total_occ = 0u64;
    let mut indexed_values: BTreeMap<String, u32> = BTreeMap::new();
    let mut terms_per_system: BTreeMap<String, u32> = BTreeMap::new();

    for (sys, valmap) in &by_sys {
        indexed_values.insert(sys.clone(), valmap.len() as u32);
        let mut term_count = 0u32;
        for (&val, terms) in valmap {
            let bkey = bucket_for(val, cfg.bucket_size);
            let mut items = Vec::new();
            for (norm, agg) in terms {
                unique_terms.insert((sys.clone(), norm.clone()));
                total_occ += agg.count as u64;
                term_count += 1;
                items.push(IndexItem {
                    t: norm.clone(),
                    s: agg.surface.clone(),
                    c: agg.count,
                    r: agg.refs.clone(),
                    b: agg.books.iter().cloned().collect(),
                    lic: agg.license.clone(),
                    kind: agg.primary_kind(),
                    kinds: agg.kinds.iter().cloned().collect(),
                });
            }
            items.sort_by(|a, b| a.t.cmp(&b.t));
            value_shards
                .entry(sys.clone())
                .or_default()
                .entry(bkey)
                .or_default()
                .extend(items);
        }
        terms_per_system.insert(sys.clone(), term_count);
    }

    for (sys, valmap) in &phrases_by_sys {
        for (&val, items) in valmap {
            let bkey = bucket_for(val, cfg.bucket_size);
            phrase_shards
                .entry(sys.clone())
                .or_default()
                .entry(bkey)
                .or_default()
                .extend(items.clone());
        }
    }

    let mut low: BTreeMap<String, BTreeMap<String, Vec<IndexItem>>> = BTreeMap::new();
    let mut mid: BTreeMap<String, BTreeMap<String, Vec<IndexItem>>> = BTreeMap::new();
    for (sys, valmap) in &by_sys {
        for (&val, terms) in valmap {
            let items: Vec<IndexItem> = terms
                .iter()
                .map(|(norm, agg)| IndexItem {
                    t: norm.clone(),
                    s: agg.surface.clone(),
                    c: agg.count,
                    r: agg.refs.clone(),
                    b: agg.books.iter().cloned().collect(),
                    lic: agg.license.clone(),
                    kind: agg.primary_kind(),
                    kinds: agg.kinds.iter().cloned().collect(),
                })
                .collect();
            if val < 100 {
                low.entry(sys.clone())
                    .or_default()
                    .entry(val.to_string())
                    .or_default()
                    .extend(items);
            } else if val < 1000 {
                mid.entry(sys.clone())
                    .or_default()
                    .entry(val.to_string())
                    .or_default()
                    .extend(items);
            }
        }
    }

    BuiltIndex {
        value_shards,
        phrase_shards,
        low,
        mid,
        unique_term_count: unique_terms.len() as u32,
        token_occurrences: total_occ,
        phrase_count: phrase_rows.len() as u32,
        indexed_values,
        terms_per_system,
        books: books.into_iter().filter(|b| !b.is_empty()).collect(),
        seed_term_slots: seed_terms,
        text_term_slots: text_terms,
        systems: by_sys.keys().cloned().collect(),
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Agg {
    pub surface: String,
    pub count: u32,
    pub refs: Vec<String>,
    pub books: BTreeSet<String>,
    pub license: String,
    /// All contributing kinds (seed, fixture, text) — not mutually exclusive
    pub kinds: BTreeSet<String>,
}

impl Agg {
    pub fn primary_kind(&self) -> String {
        if self.kinds.contains("fixture") {
            "fixture".into()
        } else if self.kinds.contains("text") {
            "text".into()
        } else if self.kinds.contains("seed") {
            "seed".into()
        } else {
            self.kinds.iter().next().cloned().unwrap_or_default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct BuiltIndex {
    pub value_shards: BTreeMap<String, BTreeMap<String, Vec<IndexItem>>>,
    pub phrase_shards: BTreeMap<String, BTreeMap<String, Vec<serde_json::Value>>>,
    pub low: BTreeMap<String, BTreeMap<String, Vec<IndexItem>>>,
    pub mid: BTreeMap<String, BTreeMap<String, Vec<IndexItem>>>,
    pub unique_term_count: u32,
    pub token_occurrences: u64,
    pub phrase_count: u32,
    pub indexed_values: BTreeMap<String, u32>,
    pub terms_per_system: BTreeMap<String, u32>,
    pub books: Vec<String>,
    pub seed_term_slots: u32,
    pub text_term_slots: u32,
    pub systems: Vec<String>,
}
