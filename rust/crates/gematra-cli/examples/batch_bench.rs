//! Reproducible batch throughput diagnostic.
//! Run: cargo run -p gematra-cli --example batch_bench --release
use gematra_cli::{run_batch, BatchItem};
use gematra_unicode::NormalizationForm;
use std::time::Instant;

fn bench(n: usize, totals_only: bool) {
    let samples = ["שלום", "בראשית", "herro", "prease", "λόγος", "abc"];
    let items: Vec<_> = (0..n)
        .map(|i| BatchItem {
            id: Some(i.to_string()),
            text: samples[i % samples.len()].to_string(),
            systems: None, // all applicable
        })
        .collect();
    let text_payload_bytes: usize = items.iter().map(|i| i.text.len()).sum();
    // Approximate serialized batch input (JSON array of {id,text})
    let serialized_input_bytes: usize = items
        .iter()
        .map(|i| {
            // rough: {"id":"...","text":"..."} plus commas
            20 + i.id.as_ref().map(|s| s.len()).unwrap_or(0) + i.text.len()
        })
        .sum::<usize>()
        + 2; // []
    let start = Instant::now();
    let out = run_batch(items, NormalizationForm::Nfc, totals_only).expect("batch");
    let elapsed = start.elapsed();
    let count = out["result_count"].as_u64().unwrap_or(0);
    let secs = elapsed.as_secs_f64().max(1e-9);
    println!(
        "items={n} text_payload_bytes={text_payload_bytes} \
         serialized_input_bytes≈{serialized_input_bytes} systems=applicable \
         totals_only={totals_only} elapsed={elapsed:?} \
         throughput={:.0} items/s result_count={count}",
        n as f64 / secs
    );
}

fn main() {
    for &n in &[500usize, 10_000, 100_000] {
        bench(n, true);
    }
    bench(500, false);
}
