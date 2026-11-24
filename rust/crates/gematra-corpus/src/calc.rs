//! Rust-authoritative per-token calculation and segment processing.
use gematra_ciphers::SystemId;
use std::collections::HashMap;
use gematra_core::{calculate, calculate_applicable};
use gematra_unicode::NormalizationForm;

use crate::models::{CorpusBuildConfig, PhraseRecord, SegmentInput, TokenRecord};
use crate::tokenize::{canonical_form, classify_language, tokenize};

fn systems_for_language(language: &str) -> Vec<SystemId> {
    match language {
        "hebrew" => vec![
            SystemId::HebrewStandard,
            SystemId::HebrewGadol,
            SystemId::HebrewKatan,
        ],
        "greek" => vec![SystemId::GreekIsopsephy],
        "english" => vec![
            SystemId::EnglishOrdinal,
            SystemId::EnglishReduction,
            SystemId::EnglishPythagorean,
        ],
        _ => vec![],
    }
}

/// Calculate token under all applicable systems using Rust engine.
pub fn calculate_token_systems(normalized: &str, language: &str) -> Vec<(String, u32)> {
    let systems = systems_for_language(language);
    if systems.is_empty() {
        return calculate_applicable(normalized, NormalizationForm::Nfc)
            .into_iter()
            .map(|r| (r.system_id, r.total))
            .collect();
    }
    systems
        .into_iter()
        .map(|sys| {
            let r = calculate(normalized, sys, NormalizationForm::Nfc);
            (r.system_id, r.total)
        })
        .collect()
}

/// Process one segment into token and phrase records.
pub fn process_segment(
    seg: &SegmentInput,
    cfg: &CorpusBuildConfig,
) -> (Vec<TokenRecord>, Vec<PhraseRecord>) {
    let lang = classify_language(&seg.text, seg.language.as_deref());
    if lang == "unknown" && seg.kind_str() != "seed" {
        return (vec![], vec![]);
    }
    let language = if lang == "unknown" {
        match seg.language.as_deref().map(|s| s.to_ascii_lowercase()).as_deref() {
            Some("he") | Some("hebrew") => "hebrew".to_string(),
            Some("el") | Some("greek") => "greek".to_string(),
            Some("en") | Some("english") => "english".to_string(),
            _ => return (vec![], vec![]),
        }
    } else {
        lang
    };

    let kind = seg.kind_str().to_string();
    let reference = seg.reference_str();
    let book = seg.book.clone().unwrap_or_default();
    let license = seg.license.clone().unwrap_or_default();
    let source_url = seg.source_url.clone().unwrap_or_default();

    let pairs = if kind == "seed" {
        let norm = canonical_form(&seg.text, &language);
        if norm.is_empty() {
            vec![]
        } else {
            vec![(seg.text.clone(), norm)]
        }
    } else {
        tokenize(&seg.text, &language)
    };

    let mut tokens = Vec::new();
    let mut norms_for_phrases: Vec<(String, String)> = Vec::new();

    for (surface, norm) in &pairs {
        if norm.is_empty() {
            continue;
        }
        norms_for_phrases.push((surface.clone(), norm.clone()));
        for (sys, val) in calculate_token_systems(norm, &language) {
            tokens.push(TokenRecord {
                normalized: norm.clone(),
                surface: surface.clone(),
                language: language.clone(),
                system: sys,
                value: val,
                r#ref: if kind == "seed" {
                    String::new()
                } else {
                    reference.clone()
                },
                book: if kind == "seed" {
                    "seed".into()
                } else {
                    book.clone()
                },
                license: license.clone(),
                source_url: source_url.clone(),
                kind: kind.clone(),
            });
        }
    }

    let mut phrases = Vec::new();
    if cfg.phrase_max_n >= 2 && norms_for_phrases.len() >= 2 && kind != "seed" {
        for i in 0..norms_for_phrases.len() - 1 {
            let (s0, n0) = &norms_for_phrases[i];
            let (s1, n1) = &norms_for_phrases[i + 1];
            let pn = format!("{n0} {n1}");
            let ps = format!("{s0} {s1}");
            let vals0 = calculate_token_systems(n0, &language);
            let vals1: HashMap<String, u32> = calculate_token_systems(n1, &language)
                .into_iter()
                .collect();
            for (sys, v0) in vals0 {
                if let Some(&v1) = vals1.get(&sys) {
                    phrases.push(PhraseRecord {
                        normalized: pn.clone(),
                        surface: ps.clone(),
                        system: sys,
                        value: v0.saturating_add(v1),
                        r#ref: reference.clone(),
                        book: book.clone(),
                        n: 2,
                        kind: kind.clone(),
                    });
                }
            }
        }
    }

    (tokens, phrases)
}
