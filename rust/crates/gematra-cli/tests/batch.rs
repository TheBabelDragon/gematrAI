//! Batch public-API integration tests.
use gematra_cli::{
    load_batch_items_from_str, parse_norm, resolve_systems, run_batch, BatchItem, BatchLimits,
};
use gematra_core::ENGINE_VERSION;
use gematra_unicode::NormalizationForm;
use serde_json::json;

fn item_json(id: Option<&str>, text: &str, systems: Option<Vec<&str>>) -> String {
    let mut v = json!({ "text": text });
    if let Some(i) = id {
        v["id"] = json!(i);
    }
    if let Some(s) = systems {
        v["systems"] = json!(s);
    }
    v.to_string()
}

fn parse_items(s: &str) -> Result<Vec<BatchItem>, String> {
    load_batch_items_from_str(s, &BatchLimits::default())
}

#[test]
fn parse_norm_strict() {
    assert_eq!(parse_norm("nfc").unwrap(), NormalizationForm::Nfc);
    assert_eq!(parse_norm("NFD").unwrap(), NormalizationForm::Nfd);
    assert_eq!(parse_norm("none").unwrap(), NormalizationForm::None);
    assert!(parse_norm("nfkc").is_err());
}

#[test]
fn json_ndjson_empty_malformed() {
    let a = item_json(Some("a"), "שלום", None);
    let b = item_json(Some("b"), "abc", Some(vec!["english-ordinal"]));
    assert_eq!(parse_items(&format!("[{a},{b}]")).unwrap().len(), 2);
    assert_eq!(parse_items(&format!("{a}\n{b}")).unwrap().len(), 2);
    assert!(parse_items("").unwrap().is_empty());
    assert!(parse_items("   ").unwrap().is_empty());
    assert!(parse_items("[").is_err());
    assert!(parse_items("[{}]").is_err());
}

#[test]
fn systems_dedupe_and_totals() {
    let payload = format!(
        "[{}]",
        item_json(None, "abc", Some(vec!["english-ordinal", "english-ordinal"]))
    );
    let items = parse_items(&payload).unwrap();
    assert_eq!(resolve_systems(&items[0]).unwrap().len(), 1);
    let only = run_batch(items.clone(), NormalizationForm::Nfc, true).unwrap();
    let full = run_batch(items, NormalizationForm::Nfc, false).unwrap();
    assert_eq!(
        only["results"][0]["totals"]["english-ordinal"],
        full["results"][0]["totals"]["english-ordinal"]
    );
}

#[test]
fn hebrew_fixture_parity() {
    let payload = format!(
        "[{}]",
        item_json(None, "שלום", Some(vec!["hebrew-standard", "hebrew-gadol"]))
    );
    let out = run_batch(parse_items(&payload).unwrap(), NormalizationForm::Nfc, true).unwrap();
    assert_eq!(out["results"][0]["totals"]["hebrew-standard"], 376);
    assert_eq!(out["results"][0]["totals"]["hebrew-gadol"], 936);
}

#[test]
fn limit_boundaries() {
    let lim = BatchLimits {
        max_input_bytes: 500,
        max_items: 2,
        max_text_bytes: 5,
    };
    assert!(load_batch_items_from_str(&" ".repeat(500), &lim).is_ok());
    assert!(load_batch_items_from_str(&" ".repeat(501), &lim).is_err());
    let a = item_json(None, "a", None);
    let b = item_json(None, "b", None);
    let c = item_json(None, "c", None);
    assert!(load_batch_items_from_str(&format!("[{a},{b}]"), &lim).is_ok());
    assert!(load_batch_items_from_str(&format!("[{a},{b},{c}]"), &lim).is_err());
    assert!(load_batch_items_from_str(&format!("[{}]", item_json(None, "abcde", None)), &lim).is_ok());
    assert!(load_batch_items_from_str(&format!("[{}]", item_json(None, "abcdef", None)), &lim).is_err());
}

#[test]
fn determinism_metadata() {
    let raw = format!(
        "[{},{}]",
        item_json(Some("1"), "שלום", Some(vec!["hebrew-standard"])),
        item_json(Some("2"), "herro prease", Some(vec!["english-ordinal"]))
    );
    let a = run_batch(parse_items(&raw).unwrap(), NormalizationForm::Nfc, true).unwrap();
    let b = run_batch(parse_items(&raw).unwrap(), NormalizationForm::Nfc, true).unwrap();
    assert_eq!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&b).unwrap());
    assert_eq!(a["engine_version"], ENGINE_VERSION);
    assert_eq!(a["results"][1]["totals"]["english-ordinal"], 128);
}

#[test]
fn perf_500() {
    let samples = ["שלום", "herro", "λόγος", "abc"];
    let items: Vec<_> = (0..500)
        .map(|i| BatchItem {
            id: Some(i.to_string()),
            text: samples[i % samples.len()].to_string(),
            systems: None,
        })
        .collect();
    let t = std::time::Instant::now();
    let out = run_batch(items, NormalizationForm::Nfc, true).unwrap();
    assert_eq!(out["result_count"], 500);
    assert!(t.elapsed().as_millis() < 5000);
}
