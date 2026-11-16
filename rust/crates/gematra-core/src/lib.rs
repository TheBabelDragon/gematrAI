//! Deterministic calculation engine. Engine version: 0.1.0
use gematra_ciphers::{is_greek_letter, is_hebrew_letter, is_latin_letter, letter_value, system_meta, SystemId, CIPHER_REGISTRY_VERSION};
use gematra_unicode::{analyze, ExclusionReason, NormalizationForm, ScriptClass, UNICODE_POLICY_VERSION};
use serde::{Deserialize, Serialize};

pub const ENGINE_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharBreakdown {
    pub original: String,
    pub scalar_values: Vec<u32>,
    pub normalized: String,
    pub script: ScriptClass,
    pub system_id: String,
    pub value: Option<u32>,
    pub included: bool,
    pub exclusion_reason: Option<ExclusionReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordBreakdown {
    pub word: String,
    pub total: u32,
    pub characters: Vec<CharBreakdown>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnicodeDiagnostics {
    pub normalization: NormalizationForm,
    pub grapheme_count: usize,
    pub has_hebrew: bool,
    pub has_greek: bool,
    pub has_latin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsupportedChar {
    pub original: String,
    pub reason: ExclusionReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationResult {
    pub original_input: String,
    pub system_id: String,
    pub system_name: String,
    pub total: u32,
    pub digital_root: Option<u32>,
    pub characters: Vec<CharBreakdown>,
    pub words: Vec<WordBreakdown>,
    pub unicode: UnicodeDiagnostics,
    pub unsupported: Vec<UnsupportedChar>,
    pub engine_version: String,
    pub cipher_registry_version: String,
    pub unicode_policy_version: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CalcError {
    #[error("unknown system id: {0}")]
    UnknownSystem(String),
}

pub fn calculate(text: &str, system_id: SystemId, norm: NormalizationForm) -> CalculationResult {
    let analysis = analyze(text, norm);
    let meta = system_meta(system_id);
    let mut characters = Vec::new();
    let mut unsupported = Vec::new();
    let mut total: u32 = 0;

    for info in &analysis.characters {
        let ch = info.original.chars().next().unwrap_or('\0');
        let (value, included, reason) = classify_and_value(system_id, info, ch);
        if let Some(v) = value { total = total.saturating_add(v); }
        if !included {
            if let Some(r) = reason.clone() {
                unsupported.push(UnsupportedChar { original: info.original.clone(), reason: r });
            }
        }
        characters.push(CharBreakdown {
            original: info.original.clone(),
            scalar_values: info.scalar_values.clone(),
            normalized: info.normalized.clone(),
            script: info.script,
            system_id: system_id.as_str().to_string(),
            value, included, exclusion_reason: reason,
        });
    }

    let words = build_words(text, &characters);
    let digital_root = match system_id {
        SystemId::EnglishOrdinal | SystemId::EnglishReduction | SystemId::EnglishPythagorean =>
            Some(gematra_ciphers::digital_root(total)),
        _ => None,
    };

    CalculationResult {
        original_input: text.to_string(),
        system_id: system_id.as_str().to_string(),
        system_name: meta.name,
        total, digital_root, characters, words,
        unicode: UnicodeDiagnostics {
            normalization: analysis.normalization,
            grapheme_count: analysis.grapheme_count,
            has_hebrew: analysis.has_hebrew,
            has_greek: analysis.has_greek,
            has_latin: analysis.has_latin,
        },
        unsupported,
        engine_version: ENGINE_VERSION.to_string(),
        cipher_registry_version: CIPHER_REGISTRY_VERSION.to_string(),
        unicode_policy_version: UNICODE_POLICY_VERSION.to_string(),
    }
}

pub fn calculate_nfc(text: &str, system_id: SystemId) -> CalculationResult {
    calculate(text, system_id, NormalizationForm::Nfc)
}

pub fn calculate_applicable(text: &str, norm: NormalizationForm) -> Vec<CalculationResult> {
    let analysis = analyze(text, norm);
    let mut results = Vec::new();
    for &sys in SystemId::all() {
        let applicable = match sys.language() {
            "hebrew" => analysis.has_hebrew,
            "greek" => analysis.has_greek,
            "english" => analysis.has_latin || (!analysis.has_hebrew && !analysis.has_greek),
            _ => true,
        };
        if applicable { results.push(calculate(text, sys, norm)); }
    }
    if results.is_empty() {
        for &sys in SystemId::all() {
            if sys.language() == "english" { results.push(calculate(text, sys, norm)); }
        }
    }
    results
}

fn classify_and_value(
    system: SystemId,
    info: &gematra_unicode::CharUnicodeInfo,
    ch: char,
) -> (Option<u32>, bool, Option<ExclusionReason>) {
    if info.is_format { return (None, false, Some(ExclusionReason::FormatControl)); }
    if info.is_hebrew_niqqud && !is_hebrew_letter(ch) {
        return (None, false, Some(ExclusionReason::HebrewNiqqud));
    }
    if info.is_combining && !is_hebrew_letter(ch) && !is_greek_letter(ch) && !is_latin_letter(ch) {
        if info.is_greek_diacritic { return (None, false, Some(ExclusionReason::GreekDiacritic)); }
        return (None, false, Some(ExclusionReason::NonLetter));
    }
    if let Some(v) = letter_value(system, ch) { return (Some(v), true, None); }
    if ch.is_whitespace() || ch.is_ascii_punctuation() {
        return (None, false, Some(ExclusionReason::NonLetter));
    }
    let reason = match (system.language(), info.script) {
        ("hebrew", ScriptClass::Hebrew) | ("greek", ScriptClass::Greek) | ("english", ScriptClass::Latin) => ExclusionReason::Unmapped,
        ("hebrew", _) | ("greek", _) | ("english", _) => ExclusionReason::WrongScript,
        _ => ExclusionReason::Unsupported,
    };
    (None, false, Some(reason))
}

fn build_words(text: &str, chars: &[CharBreakdown]) -> Vec<WordBreakdown> {
    let mut words = Vec::new();
    for word in text.split_whitespace() {
        let mut word_chars = Vec::new();
        let mut word_total = 0u32;
        for info in chars {
            if word.contains(&info.original) {
                word_chars.push(info.clone());
                if let Some(v) = info.value { word_total = word_total.saturating_add(v); }
            }
        }
        let mut seq_total = 0u32;
        let mut seq_chars = Vec::new();
        for ch in word.chars() {
            if let Some(bd) = chars.iter().find(|c| c.original == ch.to_string()) {
                seq_chars.push(bd.clone());
                if let Some(v) = bd.value { seq_total = seq_total.saturating_add(v); }
            }
        }
        if !seq_chars.is_empty() {
            words.push(WordBreakdown { word: word.to_string(), total: seq_total, characters: seq_chars });
        } else {
            words.push(WordBreakdown { word: word.to_string(), total: word_total, characters: word_chars });
        }
    }
    words
}

pub fn parse_system(id: &str) -> Result<SystemId, CalcError> {
    SystemId::from_str(id).ok_or_else(|| CalcError::UnknownSystem(id.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn herro_prease_english_ordinal() {
        let r = calculate_nfc("herro prease", SystemId::EnglishOrdinal);
        assert_eq!(r.total, 128);
        assert_eq!(r.words.len(), 2);
        assert_eq!(r.words[0].total, 64);
        assert_eq!(r.words[1].total, 64);
    }
    #[test]
    fn shalom() {
        let r = calculate_nfc("שלום", SystemId::HebrewStandard);
        assert_eq!(r.total, 376);
    }
}
