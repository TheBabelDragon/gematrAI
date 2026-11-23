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
