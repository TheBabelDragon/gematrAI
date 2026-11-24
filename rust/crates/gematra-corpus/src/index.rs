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
                kind: row.kind.clone(),
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
        if row.kind == "text" || row.kind == "fixture" {
            entry.kind = row.kind.clone();
        }
    }

    let mut seen_seed: BTreeSet<(String, String)> = BTreeSet::new();
    let mut seen_text: BTreeSet<(String, String)> = BTreeSet::new();
    for row in token_rows {
        let key = (row.system.clone(), row.normalized.clone());
        if row.kind == "seed" {
            seen_seed.insert(key);
        } else {
            seen_text.insert(key);
        }
    }
    let seed_terms = seen_seed.len() as u32;
    let text_terms = seen_text.len() as u32;

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
                    kind: agg.kind.clone(),
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
                    kind: agg.kind.clone(),
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
    pub kind: String,
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
