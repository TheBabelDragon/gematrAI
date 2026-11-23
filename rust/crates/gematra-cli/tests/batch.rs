//! Integration tests for gematra batch calculation interface.
use gematra_cli::{
    load_batch_items_from_str, resolve_systems, run_batch, BatchItem, BatchLimits,
    DEFAULT_MAX_TEXT_BYTES,
};
use gematra_ciphers::SystemId;
use gematra_core::ENGINE_VERSION;
use gematra_unicode::NormalizationForm;

fn parse_items(s: &str) -> Result<Vec<BatchItem>, String> {
    load_batch_items_from_str(s, &BatchLimits::default())
}

#[test]
fn json_array_input() {
    let items = parse_items(r#"[{\"id\":\"a\",\"text\":\"שלום\"},{\"id\":\"b\",\"text\":\"abc\"}]"#).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].id.as_deref(), Some("a"));
    assert_eq!(items[0].text, "שלום");
}

#[test]
fn ndjson_input() {
    let items = parse_items(
        r#"{\"id\":\"1\",\"text\":\"שלום\"}
{\"id\":\"2\",\"text\":\"abc\",\"systems\":[\"english-ordinal\"]}"#,
    )
    .unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[1].systems.as_ref().unwrap()[0], "english-ordinal");
}

#[test]
fn empty_input() {
    assert!(parse_items("").unwrap().is_empty());
    assert!(parse_items("   \n  ").unwrap().is_empty());
}

#[test]
fn whitespace_only_ndjson_lines_skipped() {
    let items = parse_items("\n\n{\"text\":\"x\"}\n\n").unwrap();
    assert_eq!(items.len(), 1);
}

#[test]
fn malformed_json_array() {
    let err = parse_items(r#"[{\"text\":\"ok\"},{\"text\":]"#).unwrap_err();
    assert!(err.contains("parse"), "{err}");
}

#[test]
fn malformed_ndjson() {
    let err = parse_items("{\"text\":\"ok\"}\n{not json}").unwrap_err();
    assert!(err.contains("NDJSON") || err.contains("parse"), "{err}");
}

#[test]
fn missing_text_field() {
    let err = parse_items(r#"[{\"id\":\"no-text\"}]"#).unwrap_err();
    assert!(err.contains("parse") || err.contains("missing"), "{err}");
}

#[test]
fn invalid_system_identifier() {
    let items = parse_items(r#"[{\"text\":\"abc\",\"systems\":[\"not-a-real-system\"]}]"#).unwrap();
    let err = resolve_systems(&items[0]).unwrap_err();
    assert!(err.contains("invalid system") || err.contains("unknown"), "{err}");
}

#[test]
fn selective_systems() {
    let items =
        parse_items(r#"[{\"text\":\"abc\",\"systems\":[\"english-ordinal\",\"english-reduction\"]}]"#)
            .unwrap();
    let sys = resolve_systems(&items[0]).unwrap();
    assert_eq!(sys.len(), 2);
    assert_eq!(sys[0], SystemId::EnglishOrdinal);
    assert_eq!(sys[1], SystemId::EnglishReduction);
}

#[test]
fn all_applicable_when_systems_omitted() {
    let items = parse_items(r#"[{\"text\":\"abc\"}]"#).unwrap();
    let sys = resolve_systems(&items[0]).unwrap();
    assert!(sys.is_empty());
    let out = run_batch(items, NormalizationForm::Nfc, true).unwrap();
    let results = out["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    let totals = results[0]["totals"].as_object().unwrap();
    assert!(totals.contains_key("english-ordinal"));
    assert!(totals.contains_key("english-reduction"));
    assert!(totals.contains_key("english-pythagorean"));
}

#[test]
fn empty_systems_list_means_applicable() {
    let items = parse_items(r#"[{\"text\":\"α\",\"systems\":[]}]"#).unwrap();
    let sys = resolve_systems(&items[0]).unwrap();
    assert!(sys.is_empty());
    let out = run_batch(items, NormalizationForm::Nfc, true).unwrap();
    let totals = out["results"][0]["totals"].as_object().unwrap();
    assert!(totals.contains_key("greek-isopsephy"));
}

#[test]
fn duplicate_systems_deduped() {
    let items = parse_items(
        r#"[{\"text\":\"abc\",\"systems\":[\"english-ordinal\",\"english-ordinal\",\"english-reduction\"]}]"#,
    )
    .unwrap();
    let sys = resolve_systems(&items[0]).unwrap();
    assert_eq!(sys.len(), 2);
}

#[test]
fn totals_only_omits_calculations() {
    let items = parse_items(r#"[{\"id\":\"t\",\"text\":\"abc\",\"systems\":[\"english-ordinal\"]}]"#).unwrap();
    let out = run_batch(items, NormalizationForm::Nfc, true).unwrap();
    assert!(out["results"][0].get("calculations").is_none());
    assert_eq!(out["results"][0]["totals"]["english-ordinal"], 6);
    assert_eq!(out["results"][0]["id"], "t");
    assert_eq!(out["results"][0]["original_input"], "abc");
}

#[test]
fn full_calculations_when_not_totals_only() {
    let items = parse_items(r#"[{\"text\":\"abc\",\"systems\":[\"english-ordinal\"]}]"#).unwrap();
    let out = run_batch(items, NormalizationForm::Nfc, false).unwrap();
    let calcs = out["results"][0]["calculations"].as_array().unwrap();
    assert_eq!(calcs.len(), 1);
    assert_eq!(calcs[0]["total"], 6);
    assert_eq!(calcs[0]["system_id"], "english-ordinal");
}

#[test]
fn unicode_hebrew() {
    let items = parse_items(r#"[{\"text\":\"שלום\",\"systems\":[\"hebrew-standard\",\"hebrew-gadol\"]}]"#)
        .unwrap();
    let out = run_batch(items, NormalizationForm::Nfc, true).unwrap();
    assert_eq!(out["results"][0]["totals"]["hebrew-standard"], 376);
    assert_eq!(out["results"][0]["totals"]["hebrew-gadol"], 936);
}

#[test]
fn mixed_script() {
    let items = parse_items(r#"[{\"text\":\"shalom שלום\"}]"#).unwrap();
    let out = run_batch(items, NormalizationForm::Nfc, true).unwrap();
    let totals = out["results"][0]["totals"].as_object().unwrap();
    assert!(!totals.is_empty());
}

#[test]
fn unsupported_characters_reported() {
    let items = parse_items(r#"[{\"text\":\"a b!\",\"systems\":[\"english-ordinal\"]}]"#).unwrap();
    let out = run_batch(items, NormalizationForm::Nfc, false).unwrap();
    let _uc = out["results"][0]["unsupported_count"].as_u64().unwrap();
    assert_eq!(out["results"][0]["totals"]["english-ordinal"], 3);
}

#[test]
fn stable_metadata() {
    let items = parse_items(r#"[{\"text\":\"x\"}]"#).unwrap();
    let out = run_batch(items, NormalizationForm::Nfc, true).unwrap();
    assert_eq!(out["engine_version"], ENGINE_VERSION);
    assert!(out["cipher_registry_version"].is_string());
    assert!(out["unicode_policy_version"].is_string());
    assert_eq!(out["result_count"], 1);
    assert!(out["results"][0]["normalized_form"].is_string());
}

#[test]
fn deterministic_repeated_runs() {
    let raw = r#"[{\"id\":\"1\",\"text\":\"שלום\",\"systems\":[\"hebrew-standard\"]},{\"id\":\"2\",\"text\":\"herro prease\",\"systems\":[\"english-ordinal\"]}]"#;
    let a = run_batch(parse_items(raw).unwrap(), NormalizationForm::Nfc, true).unwrap();
    let b = run_batch(parse_items(raw).unwrap(), NormalizationForm::Nfc, true).unwrap();
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
    assert_eq!(a["results"][0]["totals"]["hebrew-standard"], 376);
    assert_eq!(a["results"][1]["totals"]["english-ordinal"], 128);
}

#[test]
fn max_text_bytes_enforced() {
    let limits = BatchLimits {
        max_input_bytes: 10_000,
        max_items: 10,
        max_text_bytes: 5,
    };
    let err = load_batch_items_from_str(r#"[{\"text\":\"abcdef\"}]"#, &limits).unwrap_err();
    assert!(err.contains("max_text_bytes"), "{err}");
    assert_eq!(DEFAULT_MAX_TEXT_BYTES, 8192);
}

#[test]
fn fixture_parity_via_batch() {
    let cases = gematra_fixtures::mandatory_cases();
    let mut batch = Vec::new();
    for (i, c) in cases.iter().enumerate() {
        if c.input.is_empty() {
            continue;
        }
        batch.push(BatchItem {
            id: Some(format!("fix-{i}")),
            text: c.input.to_string(),
            systems: Some(vec![c.system.to_string()]),
        });
    }
    let out = run_batch(batch, NormalizationForm::Nfc, true).unwrap();
    let results = out["results"].as_array().unwrap();
    for (i, c) in cases.iter().enumerate() {
        if c.input.is_empty() {
            continue;
        }
        let id = format!("fix-{i}");
        let row = results
            .iter()
            .find(|r| r["id"].as_str() == Some(&id))
            .expect("result present");
        assert_eq!(
            row["totals"][c.system].as_u64().unwrap() as u32,
            c.expected_total,
            "fixture {} mismatch",
            c.name
        );
    }
}

#[test]
fn performance_representative_batch() {
    let mut items = Vec::new();
    let samples = ["שלום", "בראשית", "אלהים", "herro", "prease", "λόγος", "αβγ"];
    for i in 0..500 {
        let t = samples[i % samples.len()];
        items.push(BatchItem {
            id: Some(format!("{i}")),
            text: t.to_string(),
            systems: None,
        });
    }
    let start = std::time::Instant::now();
    let out = run_batch(items, NormalizationForm::Nfc, true).unwrap();
    let elapsed = start.elapsed();
    assert_eq!(out["result_count"], 500);
    assert!(
        elapsed.as_millis() < 5_000,
        "batch of 500 took too long: {elapsed:?}"
    );
    eprintln!("batch_perf: 500 items in {elapsed:?}");
}
