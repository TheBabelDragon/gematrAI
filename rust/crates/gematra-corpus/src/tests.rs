use super::*;
use std::collections::HashMap;
use std::fs;

#[test]
fn hebrew_shalom_standard() {
    let vals = calculate_token_systems("\u{05e9}\u{05dc}\u{05d5}\u{05dd}", "hebrew");
    let m: HashMap<_, _> = vals.into_iter().collect();
    assert_eq!(m.get("hebrew-standard"), Some(&376));
}

#[test]
fn hebrew_chai_18() {
    let vals = calculate_token_systems("\u{05d7}\u{05d9}", "hebrew");
    let m: HashMap<_, _> = vals.into_iter().collect();
    assert_eq!(m.get("hebrew-standard"), Some(&18));
}

#[test]
fn hebrew_ahava_13() {
    let vals = calculate_token_systems("\u{05d0}\u{05d4}\u{05d1}\u{05d4}", "hebrew");
    let m: HashMap<_, _> = vals.into_iter().collect();
    assert_eq!(m.get("hebrew-standard"), Some(&13));
}

#[test]
fn english_love_54() {
    let vals = calculate_token_systems("love", "english");
    let m: HashMap<_, _> = vals.into_iter().collect();
    assert_eq!(m.get("english-ordinal"), Some(&54));
}

#[test]
fn greek_logos_373() {
    let vals = calculate_token_systems("\u{03bb}\u{03cc}\u{03b3}\u{03bf}\u{03c2}", "greek");
    let m: HashMap<_, _> = vals.into_iter().collect();
    assert_eq!(m.get("greek-isopsephy"), Some(&373));
}

#[test]
fn tokenize_hebrew_with_niqqud() {
    let text = "\u{05d1}\u{05bc}\u{05b0}\u{05e8}\u{05b5}\u{05d0}\u{05e9}\u{05c1}\u{05b4}\u{0596}\u{05d9}\u{05ea}";
    let toks = tokenize(text, "hebrew");
    assert!(!toks.is_empty());
    assert_eq!(
        toks[0].1,
        "\u{05d1}\u{05e8}\u{05d0}\u{05e9}\u{05d9}\u{05ea}"
    );
}

#[test]
fn final_mem_gadol() {
    let vals = calculate_token_systems("\u{05dd}", "hebrew");
    let m: HashMap<_, _> = vals.into_iter().collect();
    assert_eq!(m.get("hebrew-standard"), Some(&40));
    assert_eq!(m.get("hebrew-gadol"), Some(&600));
}

#[test]
fn seed_segment_produces_records() {
    let seg = SegmentInput {
        id: None,
        r#ref: None,
        reference: None,
        book: None,
        language: Some("hebrew".into()),
        text: "\u{05e9}\u{05dc}\u{05d5}\u{05dd}".into(),
        license: None,
        source_url: None,
        kind: Some("seed".into()),
        notes: Some("peace".into()),
    };
    let (tokens, _) = process_segment(&seg, &CorpusBuildConfig::default());
    assert!(!tokens.is_empty());
    assert!(tokens
        .iter()
        .any(|t| t.system == "hebrew-standard" && t.value == 376));
}

#[test]
fn build_includes_acceptance_lookups() {
    let mut segs = vec![
        SegmentInput {
            id: None,
            r#ref: None,
            reference: None,
            book: None,
            language: Some("hebrew".into()),
            text: "\u{05d7}\u{05d9}".into(),
            license: None,
            source_url: None,
            kind: Some("seed".into()),
            notes: None,
        },
        SegmentInput {
            id: None,
            r#ref: None,
            reference: None,
            book: None,
            language: Some("hebrew".into()),
            text: "\u{05e9}\u{05dc}\u{05d5}\u{05dd}".into(),
            license: None,
            source_url: None,
            kind: Some("seed".into()),
            notes: None,
        },
        SegmentInput {
            id: None,
            r#ref: None,
            reference: None,
            book: None,
            language: Some("hebrew".into()),
            text: "\u{05d0}\u{05d4}\u{05d1}\u{05d4}".into(),
            license: None,
            source_url: None,
            kind: Some("seed".into()),
            notes: None,
        },
        SegmentInput {
            id: None,
            r#ref: None,
            reference: None,
            book: None,
            language: Some("english".into()),
            text: "love".into(),
            license: None,
            source_url: None,
            kind: Some("seed".into()),
            notes: None,
        },
        SegmentInput {
            id: None,
            r#ref: None,
            reference: None,
            book: None,
            language: Some("greek".into()),
            text: "\u{03bb}\u{03cc}\u{03b3}\u{03bf}\u{03c2}".into(),
            license: None,
            source_url: None,
            kind: Some("seed".into()),
            notes: None,
        },
    ];
    segs.push(SegmentInput {
        id: None,
        r#ref: Some("Genesis 1:1".into()),
        reference: None,
        book: Some("Genesis".into()),
        language: Some("he".into()),
        text: "\u{05d1}\u{05e8}\u{05d0}\u{05e9}\u{05d9}\u{05ea} \u{05d1}\u{05e8}\u{05d0} \u{05d0}\u{05dc}\u{05d4}\u{05d9}\u{05dd}".into(),
        license: Some("Public Domain".into()),
        source_url: Some("https://www.sefaria.org/Genesis.1.1".into()),
        kind: Some("fixture".into()),
        notes: None,
    });

    let dir = std::env::temp_dir().join("gematra_corpus_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let manifest = build_corpus(&segs, &dir, &CorpusBuildConfig::default(), "test-build").unwrap();
    assert_eq!(manifest["tower"], TOWER_VERSION);

    let low: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.join("values_low.json")).unwrap()).unwrap();
    let chai = &low["hebrew-standard"]["18"];
    assert!(chai.is_array(), "expected array for 18, got {chai}");
    let terms: Vec<&str> = chai
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|x| x["t"].as_str())
        .collect();
    assert!(terms.contains(&"\u{05d7}\u{05d9}"), "terms={terms:?}");

    let mid: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.join("values_mid.json")).unwrap()).unwrap();
    let shalom = &mid["hebrew-standard"]["376"];
    assert!(shalom
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["t"] == "\u{05e9}\u{05dc}\u{05d5}\u{05dd}"));

    let love = &low["english-ordinal"]["54"];
    assert!(love.as_array().unwrap().iter().any(|x| x["t"] == "love"));

    let logos = &mid["greek-isopsephy"]["373"];
    assert!(
        logos.as_array().unwrap().iter().any(|x| {
            let t = x["t"].as_str().unwrap_or("");
            t == "\u{03bb}\u{03cc}\u{03b3}\u{03bf}\u{03c2}"
                || t == "\u{03bb}\u{03bf}\u{03b3}\u{03bf}\u{03c2}"
        }),
        "logos entries={logos}"
    );

    let vals =
        calculate_token_systems("\u{05d1}\u{05e8}\u{05d0}\u{05e9}\u{05d9}\u{05ea}", "hebrew");
    let std = vals.iter().find(|(s, _)| s == "hebrew-standard").unwrap().1;
    assert!(std > 0);
}
