//! Full corpus build: process segments, write shards and manifest.
use gematra_core::ENGINE_VERSION;
use gematra_unicode::UNICODE_POLICY_VERSION;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::calc::process_segment;
use crate::index::{bucket_for, Agg};
use crate::models::{
    CorpusBuildConfig, IndexItem, SegmentInput, CORPUS_SCHEMA_VERSION, TOWER_VERSION,
};

/// Full corpus build from segments → artifacts on disk.
pub fn build_corpus(
    segments: &[SegmentInput],
    out_dir: &Path,
    cfg: &CorpusBuildConfig,
    build_id: &str,
) -> Result<serde_json::Value, String> {
    let mut all_tokens = Vec::new();
    let mut all_phrases = Vec::new();
    for seg in segments {
        let (t, p) = process_segment(seg, cfg);
        all_tokens.extend(t);
        all_phrases.extend(p);
    }
    let segment_count = segments.len() as u32;
    let mut source_inventory: Vec<String> = Vec::new();
    let mut kinds = BTreeSet::new();
    for seg in segments {
        kinds.insert(seg.kind_str().to_string());
    }
    for k in kinds {
        source_inventory.push(k);
    }

    let mut by_sys_val: BTreeMap<String, BTreeMap<u32, BTreeMap<String, Agg>>> = BTreeMap::new();
    let mut books = BTreeSet::new();
    for row in &all_tokens {
        books.insert(row.book.clone());
        let entry = by_sys_val
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

    let values_dir = out_dir.join("values");
    let phrases_dir = out_dir.join("phrases");
    let meta_dir = out_dir.join("metadata");
    if values_dir.exists() {
        fs::remove_dir_all(&values_dir).map_err(|e| e.to_string())?;
    }
    if phrases_dir.exists() {
        fs::remove_dir_all(&phrases_dir).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&values_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&phrases_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&meta_dir).map_err(|e| e.to_string())?;

    let mut systems: Vec<String> = by_sys_val.keys().cloned().collect();
    systems.sort();
    let mut indexed_values: BTreeMap<String, u32> = BTreeMap::new();
    let mut terms_per_system: BTreeMap<String, u32> = BTreeMap::new();
    let mut unique_terms: BTreeSet<(String, String)> = BTreeSet::new();
    let mut total_occ = 0u64;
    let mut seed_slots: BTreeSet<(String, String)> = BTreeSet::new();
    let mut text_slots: BTreeSet<(String, String)> = BTreeSet::new();
    let mut low_pack: BTreeMap<String, BTreeMap<String, Vec<IndexItem>>> = BTreeMap::new();
    let mut mid_pack: BTreeMap<String, BTreeMap<String, Vec<IndexItem>>> = BTreeMap::new();

    for (sys, valmap) in &by_sys_val {
        indexed_values.insert(sys.clone(), valmap.len() as u32);
        let mut tcount = 0u32;
        let mut bucket_acc: BTreeMap<String, BTreeMap<String, Vec<IndexItem>>> = BTreeMap::new();
        for (&val, terms) in valmap {
            let bkey = bucket_for(val, cfg.bucket_size);
            let mut items = Vec::new();
            for (norm, agg) in terms {
                unique_terms.insert((sys.clone(), norm.clone()));
                total_occ += agg.count as u64;
                tcount += 1;
                if agg.kinds.contains("seed") {
                    seed_slots.insert((sys.clone(), norm.clone()));
                }
                if agg.kinds.iter().any(|k| k == "fixture" || k == "text") {
                    text_slots.insert((sys.clone(), norm.clone()));
                }
                let item = IndexItem {
                    t: norm.clone(),
                    s: agg.surface.clone(),
                    c: agg.count,
                    r: agg.refs.clone(),
                    b: agg.books.iter().cloned().collect(),
                    lic: agg.license.clone(),
                    kind: agg.primary_kind(),
                    kinds: agg.kinds.iter().cloned().collect(),
                };
                items.push(item.clone());
                if val < 100 {
                    low_pack
                        .entry(sys.clone())
                        .or_default()
                        .entry(val.to_string())
                        .or_default()
                        .push(item.clone());
                } else if val < 1000 {
                    mid_pack
                        .entry(sys.clone())
                        .or_default()
                        .entry(val.to_string())
                        .or_default()
                        .push(item.clone());
                }
            }
            items.sort_by(|a, b| a.t.cmp(&b.t));
            bucket_acc
                .entry(bkey)
                .or_default()
                .insert(val.to_string(), items);
        }
        terms_per_system.insert(sys.clone(), tcount);
        let sys_dir = values_dir.join(sys);
        fs::create_dir_all(&sys_dir).map_err(|e| e.to_string())?;
        for (bkey, content) in bucket_acc {
            let path = sys_dir.join(format!("{bkey}.json"));
            let json = serde_json::to_string(&content).map_err(|e| e.to_string())?;
            fs::write(&path, json).map_err(|e| e.to_string())?;
        }
    }

    let mut phrases_by_sys: BTreeMap<String, BTreeMap<u32, Vec<serde_json::Value>>> =
        BTreeMap::new();
    for row in &all_phrases {
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
    for (sys, valmap) in &phrases_by_sys {
        let mut bucket_acc: BTreeMap<String, BTreeMap<String, Vec<serde_json::Value>>> =
            BTreeMap::new();
        for (&val, items) in valmap {
            let bkey = bucket_for(val, cfg.bucket_size);
            bucket_acc
                .entry(bkey)
                .or_default()
                .insert(val.to_string(), items.clone());
        }
        let sys_dir = phrases_dir.join(sys);
        fs::create_dir_all(&sys_dir).map_err(|e| e.to_string())?;
        for (bkey, content) in bucket_acc {
            let path = sys_dir.join(format!("{bkey}.json"));
            fs::write(
                &path,
                serde_json::to_string(&content).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
        }
    }

    fs::write(
        out_dir.join("values_low.json"),
        serde_json::to_string(&low_pack).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        out_dir.join("values_mid.json"),
        serde_json::to_string(&mid_pack).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    let mut multi_kind_terms = 0u32;
    for valmap in by_sys_val.values() {
        for terms in valmap.values() {
            for agg in terms.values() {
                if agg.kinds.len() > 1 {
                    multi_kind_terms += 1;
                }
            }
        }
    }
    let book_list: Vec<String> = books.into_iter().filter(|b| !b.is_empty()).collect();
    let stats = serde_json::json!({
        "unique_terms": unique_terms.len(),
        "indexed_term_slots": unique_terms.len(),
        "terms_per_system": terms_per_system,
        "token_occurrences": total_occ,
        "phrases": all_phrases.len(),
        "indexed_values": indexed_values,
        "n_books": book_list.len(),
        "segments": segment_count,
        "seed_term_slots": seed_slots.len(),
        "text_term_slots": text_slots.len(),
        "source_terms": text_slots.len(),
        "seed_terms": seed_slots.len(),
        "multi_kind_terms": multi_kind_terms,
    });

    let manifest = serde_json::json!({
        "buildId": build_id,
        "corpusSchemaVersion": CORPUS_SCHEMA_VERSION,
        "tower": TOWER_VERSION,
        "name": "gematrAI Corpus Tower",
        "valuesBase": "./corpus/generated/values/",
        "phrasesBase": "./corpus/generated/phrases/",
        "bucketSize": cfg.bucket_size,
        "systems": systems,
        "stats": stats,
        "exhaustive": false,
        "disclaimer": "Gematria values are computed by the Rust gematrAI engine from selected text segments and seed lexicon. Numerical equivalence does not establish semantic, theological, historical, or causal relationships.",
        "lowValuesUrl": "./corpus/generated/values_low.json",
        "midValuesUrl": "./corpus/generated/values_mid.json",
        "engine_version": ENGINE_VERSION,
        "cipher_registry_version": gematra_ciphers::CIPHER_REGISTRY_VERSION,
        "unicode_policy_version": UNICODE_POLICY_VERSION,
        "source_inventory": source_inventory,
    });
    fs::write(
        out_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    let build_meta = serde_json::json!({
        "tower": TOWER_VERSION,
        "name": "gematrAI Corpus Tower",
        "source": "fixtures+seed",
        "scope": ["fixtures", "seed"],
        "build": build_id,
        "engine_version": ENGINE_VERSION,
        "cipher_registry_version": gematra_ciphers::CIPHER_REGISTRY_VERSION,
        "unicode_policy_version": UNICODE_POLICY_VERSION,
        "corpus_schema_version": CORPUS_SCHEMA_VERSION,
        "systems": systems,
        "stats": {
            "segments": segment_count,
            "unique_tokens": unique_terms.len(),
            "token_occurrences": total_occ,
            "n_books": book_list.len(),
            "books": book_list,
            "seed_term_slots": seed_slots.len(),
            "text_term_slots": text_slots.len(),
            "multi_kind_terms": multi_kind_terms,
        },
        "disclaimer": manifest["disclaimer"],
    });
    fs::write(
        meta_dir.join("build.json"),
        serde_json::to_string_pretty(&build_meta).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    Ok(manifest)
}
