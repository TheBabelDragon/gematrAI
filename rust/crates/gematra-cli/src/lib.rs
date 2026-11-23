//! Batch calculation library surface for the gematra CLI.
//! Used by the binary and by unit tests.
use gematra_ciphers::SystemId;
use gematra_core::{
    calculate, calculate_applicable, parse_system, CalculationResult, ENGINE_VERSION,
};
use gematra_unicode::{analyze, NormalizationForm};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::{self, Read};
use std::path::PathBuf;

/// Default safety limits for batch mode (corpus-scale safe).
pub const DEFAULT_MAX_INPUT_BYTES: usize = 32 * 1024 * 1024;
pub const DEFAULT_MAX_ITEMS: usize = 100_000;
pub const DEFAULT_MAX_TEXT_BYTES: usize = 8_192;

#[derive(Debug, Deserialize)]
pub struct BatchItem {
    #[serde(default)]
    pub id: Option<String>,
    pub text: String,
    #[serde(default)]
    pub systems: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct BatchResult {
    pub id: Option<String>,
    pub original_input: String,
    /// NFC form via the same unicode engine used by calculations.
    pub normalized_form: String,
    pub totals: serde_json::Map<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calculations: Option<Vec<CalculationResult>>,
    pub unsupported_count: usize,
}

#[derive(Debug, Clone)]
pub struct BatchLimits {
    pub max_input_bytes: usize,
    pub max_items: usize,
    pub max_text_bytes: usize,
}

impl Default for BatchLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: DEFAULT_MAX_INPUT_BYTES,
            max_items: DEFAULT_MAX_ITEMS,
            max_text_bytes: DEFAULT_MAX_TEXT_BYTES,
        }
    }
}

pub fn parse_norm(s: &str) -> NormalizationForm {
    match s.to_ascii_lowercase().as_str() {
        "none" => NormalizationForm::None,
        "nfd" => NormalizationForm::Nfd,
        _ => NormalizationForm::Nfc,
    }
}

/// Load and validate batch items. Atomic: any error rejects the entire batch.
pub fn load_batch_items_from_str(
    text: &str,
    limits: &BatchLimits,
) -> Result<Vec<BatchItem>, String> {
    let raw_len = text.len();
    if raw_len > limits.max_input_bytes {
        return Err(format!(
            "input exceeds max_input_bytes ({} > {})",
            raw_len, limits.max_input_bytes
        ));
    }
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(vec![]);
    }

    let items: Vec<BatchItem> = if trimmed.starts_with('[') {
        serde_json::from_str(trimmed).map_err(|e| {
            format!(
                "JSON array parse error at line {}, column {}: {e}",
                e.line(),
                e.column()
            )
        })?
    } else {
        let mut items = Vec::new();
        for (i, line) in trimmed.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let item: BatchItem = serde_json::from_str(line).map_err(|e| {
                format!(
                    "NDJSON line {} parse error (col {}): {e}",
                    i + 1,
                    e.column()
                )
            })?;
            items.push(item);
        }
        items
    };

    if items.len() > limits.max_items {
        return Err(format!(
            "batch item count exceeds max_items ({} > {})",
            items.len(),
            limits.max_items
        ));
    }

    for (i, item) in items.iter().enumerate() {
        let text_bytes = item.text.len();
        if text_bytes > limits.max_text_bytes {
            return Err(format!(
                "item[{}] id={:?} text exceeds max_text_bytes ({} > {})",
                i, item.id, text_bytes, limits.max_text_bytes
            ));
        }
    }

    Ok(items)
}

pub fn load_batch_items(
    path: Option<PathBuf>,
    limits: &BatchLimits,
) -> Result<Vec<BatchItem>, String> {
    let raw = if let Some(p) = path {
        std::fs::read(&p).map_err(|e| format!("read {}: {}", p.display(), e))?
    } else {
        let mut buf = Vec::new();
        io::stdin()
            .read_to_end(&mut buf)
            .map_err(|e| format!("read stdin: {e}"))?;
        buf
    };

    if raw.len() > limits.max_input_bytes {
        return Err(format!(
            "input exceeds max_input_bytes ({} > {})",
            raw.len(),
            limits.max_input_bytes
        ));
    }

    let text = String::from_utf8(raw).map_err(|e| format!("input is not valid UTF-8: {e}"))?;
    load_batch_items_from_str(&text, limits)
}

/// Resolve systems: empty/None => applicable; otherwise parse & dedupe preserving order.
pub fn resolve_systems(item: &BatchItem) -> Result<Vec<SystemId>, String> {
    match &item.systems {
        None => Ok(vec![]),
        Some(list) if list.is_empty() => Ok(vec![]),
        Some(list) => {
            let mut out = Vec::with_capacity(list.len());
            let mut seen = HashSet::new();
            for id in list {
                let sys = parse_system(id)
                    .map_err(|e| format!("item id={:?}: invalid system '{id}': {e}", item.id))?;
                if seen.insert(sys) {
                    out.push(sys);
                }
            }
            Ok(out)
        }
    }
}

/// NFC form via the same unicode engine used by calculations (no second normalizer).
pub fn normalized_form_for(text: &str, norm: NormalizationForm) -> String {
    let analysis = analyze(text, norm);
    if analysis.characters.is_empty() {
        return text.to_string();
    }
    analysis
        .characters
        .iter()
        .map(|c| c.normalized.as_str())
        .collect()
}

pub fn run_batch(
    items: Vec<BatchItem>,
    norm: NormalizationForm,
    totals_only: bool,
) -> Result<serde_json::Value, String> {
    let mut results = Vec::with_capacity(items.len());
    for item in items {
        let systems = resolve_systems(&item)?;
        let calcs: Vec<CalculationResult> = if systems.is_empty() {
            calculate_applicable(&item.text, norm)
        } else {
            systems
                .into_iter()
                .map(|sys| calculate(&item.text, sys, norm))
                .collect()
        };
        let mut totals = serde_json::Map::new();
        let mut unsupported_count = 0usize;
        for c in &calcs {
            totals.insert(c.system_id.clone(), serde_json::json!(c.total));
            unsupported_count += c.unsupported.len();
        }
        let normalized_form = normalized_form_for(&item.text, norm);
        results.push(BatchResult {
            id: item.id,
            original_input: item.text,
            normalized_form,
            totals,
            calculations: if totals_only { None } else { Some(calcs) },
            unsupported_count,
        });
    }
    Ok(serde_json::json!({
        "engine_version": ENGINE_VERSION,
        "cipher_registry_version": gematra_ciphers::CIPHER_REGISTRY_VERSION,
        "unicode_policy_version": gematra_unicode::UNICODE_POLICY_VERSION,
        "result_count": results.len(),
        "results": results,
    }))
}

pub fn engine_metadata() -> serde_json::Value {
    serde_json::json!({
        "engine_version": ENGINE_VERSION,
        "cipher_registry_version": gematra_ciphers::CIPHER_REGISTRY_VERSION,
        "unicode_policy_version": gematra_unicode::UNICODE_POLICY_VERSION,
    })
}
