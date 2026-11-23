//! Batch public-API and CLI integration tests.
use gematra_cli::{
    load_batch_items_from_str, parse_norm, resolve_systems, run_batch, BatchItem, BatchLimits,
};
use gematra_core::ENGINE_VERSION;
use gematra_unicode::NormalizationForm;
use serde_json::json;
use std::io::Write;
use std::process::{Command, Stdio};

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

fn gematra_bin() -> std::path::PathBuf {
    if let Ok(p) = std::env::var("CARGO_BIN_EXE_gematra") {
        return std::path::PathBuf::from(p);
    }
    let mut p = std::env::current_exe().expect("current_exe");
    p.pop();
    p.pop();
    p.push("gematra");
    p
}

fn run_cli(args: &[&str], stdin_data: Option<&[u8]>) -> (i32, String, String) {
    let mut cmd = Command::new(gematra_bin());
    cmd.args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn gematra");
    if let Some(data) = stdin_data {
        let mut stdin = child.stdin.take().expect("stdin");
        stdin.write_all(data).expect("write stdin");
    }
    let out = child.wait_with_output().expect("wait");
    let code = out.status.code().unwrap_or(-1);
    (
        code,
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn parse_norm_strict() {
    assert_eq!(parse_norm("nfc").unwrap(), NormalizationForm::Nfc);
    assert_eq!(parse_norm("NFD").unwrap(), NormalizationForm::Nfd);
    assert_eq!(parse_norm("none").unwrap(), NormalizationForm::None);
    assert!(parse_norm("nfkc").is_err());
    assert!(parse_norm("").is_err());
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
fn systems_dedupe_selective_omitted_empty() {
    let payload = format!(
        "[{}]",
        item_json(None, "abc", Some(vec!["english-ordinal", "english-ordinal"]))
    );
    let items = parse_items(&payload).unwrap();
    assert_eq!(resolve_systems(&items[0]).unwrap().len(), 1);

    let payload = format!(
        "[{}]",
        item_json(None, "abc", Some(vec!["english-ordinal", "english-reduction"]))
    );
    let out = run_batch(parse_items(&payload).unwrap(), NormalizationForm::Nfc, true).unwrap();
    let totals = out["results"][0]["totals"].as_object().unwrap();
    assert_eq!(totals.len(), 2);
    assert_eq!(totals["english-ordinal"], 6);

    let payload = format!("[{}]", item_json(None, "α", None));
    let out = run_batch(parse_items(&payload).unwrap(), NormalizationForm::Nfc, true).unwrap();
    assert!(out["results"][0]["totals"]
        .as_object()
        .unwrap()
        .contains_key("greek-isopsephy"));

    let payload = format!("[{}]", item_json(None, "α", Some(vec![])));
    let out = run_batch(parse_items(&payload).unwrap(), NormalizationForm::Nfc, true).unwrap();
    assert!(out["results"][0]["totals"]
        .as_object()
        .unwrap()
        .contains_key("greek-isopsephy"));
}

#[test]
fn totals_only_parity_with_full() {
    let payload = format!(
        "[{}]",
        item_json(None, "abc", Some(vec!["english-ordinal"]))
    );
    let items = parse_items(&payload).unwrap();
    let only = run_batch(items.clone(), NormalizationForm::Nfc, true).unwrap();
    let full = run_batch(items, NormalizationForm::Nfc, false).unwrap();
    assert!(only["results"][0].get("calculations").is_none());
    assert!(full["results"][0].get("calculations").is_some());
    assert_eq!(
        only["results"][0]["totals"]["english-ordinal"],
        full["results"][0]["totals"]["english-ordinal"]
    );
    assert_eq!(full["results"][0]["calculations"][0]["total"], 6);
}

#[test]
fn hebrew_greek_latin_mixed_and_empty() {
    let payload = format!(
        "[{}]",
        item_json(None, "שלום", Some(vec!["hebrew-standard", "hebrew-gadol"]))
    );
    let out = run_batch(parse_items(&payload).unwrap(), NormalizationForm::Nfc, true).unwrap();
    assert_eq!(out["results"][0]["totals"]["hebrew-standard"], 376);
    assert_eq!(out["results"][0]["totals"]["hebrew-gadol"], 936);

    let payload = format!("[{}]", item_json(None, "α", Some(vec!["greek-isopsephy"])));
    let out = run_batch(parse_items(&payload).unwrap(), NormalizationForm::Nfc, true).unwrap();
    assert_eq!(out["results"][0]["totals"]["greek-isopsephy"], 1);

    let payload = format!("[{}]", item_json(None, "shalom שלום", None));
    let out = run_batch(parse_items(&payload).unwrap(), NormalizationForm::Nfc, true).unwrap();
    assert!(!out["results"][0]["totals"].as_object().unwrap().is_empty());

    let payload = format!("[{}]", item_json(None, "", Some(vec!["english-ordinal"])));
    let out = run_batch(parse_items(&payload).unwrap(), NormalizationForm::Nfc, true).unwrap();
    assert_eq!(out["results"][0]["totals"]["english-ordinal"], 0);
    assert_eq!(out["results"][0]["original_input"], "");
}

#[test]
fn unsupported_character_accounting() {
    let payload = format!(
        "[{}]",
        item_json(None, "a b!", Some(vec!["english-ordinal"]))
    );
    let out = run_batch(parse_items(&payload).unwrap(), NormalizationForm::Nfc, false).unwrap();
    assert_eq!(out["results"][0]["totals"]["english-ordinal"], 3);
    assert!(out["results"][0]["unsupported_count"].as_u64().unwrap() >= 1);
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
    assert!(a["cipher_registry_version"].is_string());
    assert!(a["unicode_policy_version"].is_string());
    assert_eq!(a["results"][0]["original_input"], "שלום");
    assert_eq!(a["results"][1]["totals"]["english-ordinal"], 128);
}

#[test]
fn batch_scale_smoke() {
    let samples = ["שלום", "herro", "λόγος", "abc"];
    let items: Vec<_> = (0..500)
        .map(|i| BatchItem {
            id: Some(i.to_string()),
            text: samples[i % samples.len()].to_string(),
            systems: None,
        })
        .collect();
    let out = run_batch(items, NormalizationForm::Nfc, true).unwrap();
    assert_eq!(out["result_count"], 500);
}

#[test]
fn cli_batch_json_array() {
    let input = r#"[{"id":"t1","text":"שלום","systems":["hebrew-standard"]},{"id":"t2","text":"herro prease","systems":["english-ordinal"]}]"#;
    let (code, stdout, stderr) = run_cli(&["batch", "--totals-only"], Some(input.as_bytes()));
    assert_eq!(code, 0, "stderr={stderr}");
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("json stdout");
    assert_eq!(v["result_count"], 2);
    assert_eq!(v["results"][0]["totals"]["hebrew-standard"], 376);
    assert_eq!(v["results"][1]["totals"]["english-ordinal"], 128);
}

#[test]
fn cli_batch_ndjson() {
    let input = concat!(
        r#"{"id":"1","text":"שלום","systems":["hebrew-standard"]}"#,
        "\n",
        r#"{"id":"2","text":"abc","systems":["english-ordinal"]}"#,
        "\n"
    );
    let (code, stdout, stderr) = run_cli(&["batch", "--totals-only"], Some(input.as_bytes()));
    assert_eq!(code, 0, "stderr={stderr}");
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("json");
    assert_eq!(v["result_count"], 2);
    assert_eq!(v["results"][0]["totals"]["hebrew-standard"], 376);
    assert_eq!(v["results"][1]["totals"]["english-ordinal"], 6);
}

#[test]
fn cli_invalid_norm_calc_all_batch() {
    for args in [
        vec!["calc", "--system", "english-ordinal", "--norm", "nfkc", "a"],
        vec!["all", "--norm", "nfkc", "a"],
        vec!["batch", "--norm", "nfkc", "--totals-only"],
    ] {
        let stdin = if args[0] == "batch" {
            Some(br#"[{"text":"a"}]"#.as_slice())
        } else {
            None
        };
        let (code, stdout, stderr) = run_cli(&args, stdin);
        assert_ne!(code, 0, "args={args:?} should fail");
        assert!(stdout.trim().is_empty(), "no partial stdout for {args:?}: {stdout}");
        assert!(stderr.contains("invalid normalization") || stderr.contains("nfkc"), "stderr={stderr}");
    }
}

#[test]
fn cli_invalid_system_atomic() {
    let input = r#"[{"text":"abc","systems":["not-a-system"]}]"#;
    let (code, stdout, stderr) = run_cli(&["batch", "--totals-only"], Some(input.as_bytes()));
    assert_ne!(code, 0);
    assert!(stdout.trim().is_empty(), "no partial stdout: {stdout}");
    assert!(!stderr.is_empty());
}

#[test]
fn cli_malformed_json_atomic() {
    let (code, stdout, _stderr) = run_cli(&["batch", "--totals-only"], Some(b"["));
    assert_ne!(code, 0);
    assert!(stdout.trim().is_empty(), "no partial stdout: {stdout}");
}

#[test]
fn cli_invalid_utf8_atomic() {
    let (code, stdout, stderr) = run_cli(&["batch", "--totals-only"], Some(&[0xff, 0xfe, 0xfd]));
    assert_ne!(code, 0);
    assert!(stdout.trim().is_empty(), "no partial stdout: {stdout}");
    assert!(stderr.contains("UTF-8") || stderr.contains("utf"), "stderr={stderr}");
}

#[test]
fn cli_max_text_bytes_atomic() {
    let input = r#"[{"text":"abcdef"}]"#;
    let (code, stdout, stderr) = run_cli(
        &["batch", "--totals-only", "--max-text-bytes", "5"],
        Some(input.as_bytes()),
    );
    assert_ne!(code, 0);
    assert!(stdout.trim().is_empty(), "no partial stdout: {stdout}");
    assert!(stderr.contains("max_text_bytes"), "stderr={stderr}");
}

#[test]
fn cli_fixtures_still_ok() {
    let (code, stdout, stderr) = run_cli(&["fixtures"], None);
    assert_eq!(code, 0, "stderr={stderr}");
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("json");
    assert_eq!(v["all_ok"], true);
}
