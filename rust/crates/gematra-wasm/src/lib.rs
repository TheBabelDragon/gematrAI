//! WASM surface for the gematrAI calculation engine.
//!
//! Stable JavaScript-facing interface:
//! - init / version stamps
//! - list systems
//! - calculate text (one system)
//! - calculate all applicable systems
//! - character breakdown is embedded in the result JSON

use gematra_ciphers::{all_system_meta, SystemId};
use gematra_core::{
    calculate, calculate_applicable, calculate_nfc, parse_system, ENGINE_VERSION,
};
use gematra_unicode::{NormalizationForm, UNICODE_POLICY_VERSION};
use wasm_bindgen::prelude::*;

/// Initialize the WASM module (install panic hook for better console errors).
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Engine version string.
#[wasm_bindgen]
pub fn engine_version() -> String {
    ENGINE_VERSION.to_string()
}

/// Cipher registry version.
#[wasm_bindgen]
pub fn cipher_registry_version() -> String {
    gematra_ciphers::CIPHER_REGISTRY_VERSION.to_string()
}

/// Unicode policy version.
#[wasm_bindgen]
pub fn unicode_policy_version() -> String {
    UNICODE_POLICY_VERSION.to_string()
}

/// Combined version object as JSON.
#[wasm_bindgen]
pub fn versions_json() -> String {
    serde_json::json!({
        "engine_version": ENGINE_VERSION,
        "cipher_registry_version": gematra_ciphers::CIPHER_REGISTRY_VERSION,
        "unicode_policy_version": UNICODE_POLICY_VERSION,
    })
    .to_string()
}

/// List all registered systems as JSON array of metadata objects.
#[wasm_bindgen]
pub fn list_systems_json() -> String {
    serde_json::to_string(&all_system_meta()).unwrap_or_else(|_| "[]".into())
}

fn parse_norm_js(s: &str) -> NormalizationForm {
    match s.to_ascii_lowercase().as_str() {
        "none" => NormalizationForm::None,
        "nfd" => NormalizationForm::Nfd,
        _ => NormalizationForm::Nfc,
    }
}

/// Calculate one system. Returns JSON CalculationResult.
/// `norm` is "nfc" | "nfd" | "none" (default nfc).
#[wasm_bindgen]
pub fn calculate_json(text: &str, system_id: &str, norm: &str) -> Result<String, JsValue> {
    let sys = parse_system(system_id).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let result = calculate(text, sys, parse_norm_js(norm));
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Calculate all applicable systems. Returns JSON array of CalculationResult.
#[wasm_bindgen]
pub fn calculate_all_json(text: &str, norm: &str) -> String {
    let results = calculate_applicable(text, parse_norm_js(norm));
    serde_json::to_string(&results).unwrap_or_else(|_| "[]".into())
}

/// Convenience: NFC + single system.
#[wasm_bindgen]
pub fn calculate_nfc_json(text: &str, system_id: &str) -> Result<String, JsValue> {
    let sys = parse_system(system_id).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let result = calculate_nfc(text, sys);
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Health check — returns "ok" if the module loaded correctly.
#[wasm_bindgen]
pub fn health() -> String {
    "ok".to_string()
}
