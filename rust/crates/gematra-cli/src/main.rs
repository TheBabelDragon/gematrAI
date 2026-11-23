//! gematra CLI — native execution of the same calculation engine.
//!
//! Numerical authority for gematrAI. Batch mode is the preferred interface
//! for corpus builders so values are produced by the same engine as WASM.
use clap::{Parser, Subcommand};
use gematra_ciphers::all_system_meta;
use gematra_core::{
    calculate, calculate_applicable, parse_system, CalculationResult, ENGINE_VERSION,
};
use gematra_unicode::NormalizationForm;
use serde::{Deserialize, Serialize};
use std::io::{self, Read};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(
    name = "gematra",
    version = ENGINE_VERSION,
    about = "gematrAI calculation engine (Rust authority)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Calculate one system for a single text
    Calc {
        text: Option<String>,
        #[arg(short, long)]
        system: String,
        #[arg(long, default_value = "nfc")]
        norm: String,
        #[arg(long)]
        pretty: bool,
    },
    /// Calculate all applicable systems for a single text
    All {
        text: Option<String>,
        #[arg(long, default_value = "nfc")]
        norm: String,
        #[arg(long)]
        pretty: bool,
    },
    /// Batch calculate many inputs in one process invocation.
    ///
    /// Input (stdin or --input file): JSON array of BatchItem, or NDJSON lines.
    /// Each item: { "id": "...", "text": "...", "systems": ["hebrew-standard"]? }
    /// If systems omitted or empty, all applicable systems are computed.
    ///
    /// Output: JSON object with engine/registry versions and results array.
    Batch {
        /// Path to JSON/NDJSON input (default: stdin)
        #[arg(short, long)]
        input: Option<PathBuf>,
        #[arg(long, default_value = "nfc")]
        norm: String,
        #[arg(long)]
        pretty: bool,
        /// Emit only totals map per item (smaller corpus-friendly payload)
        #[arg(long)]
        totals_only: bool,
    },
    Systems {
        #[arg(long)]
        pretty: bool,
    },
    Fixtures {
        #[arg(long)]
        pretty: bool,
    },
    Version {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Deserialize)]
struct BatchItem {
    /// Caller-supplied identifier (preserved in output)
    id: Option<String>,
    text: String,
    /// Optional list of system ids. Empty / missing => all applicable.
    systems: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct BatchResult {
    id: Option<String>,
    original_input: String,
    /// Per-system totals (always present)
    totals: serde_json::Map<String, serde_json::Value>,
    /// Full CalculationResult objects when not totals_only
    #[serde(skip_serializing_if = "Option::is_none")]
    calculations: Option<Vec<CalculationResult>>,
    unsupported_count: usize,
}

fn read_text(arg: Option<String>) -> String {
    if let Some(t) = arg {
        return t;
    }
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).expect("read stdin");
    buf
}

fn parse_norm(s: &str) -> NormalizationForm {
    match s.to_ascii_lowercase().as_str() {
        "none" => NormalizationForm::None,
        "nfd" => NormalizationForm::Nfd,
        _ => NormalizationForm::Nfc,
    }
}

fn load_batch_items(path: Option<PathBuf>) -> Result<Vec<BatchItem>, String> {
    let raw = if let Some(p) = path {
        std::fs::read_to_string(&p).map_err(|e| format!("read {}: {}", p.display(), e))?
    } else {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("read stdin: {e}"))?;
        buf
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(vec![]);
    }
    // Prefer JSON array
    if trimmed.starts_with('[') {
        let items: Vec<BatchItem> =
            serde_json::from_str(trimmed).map_err(|e| format!("JSON array parse: {e}"))?;
        return Ok(items);
    }
    // NDJSON fallback
    let mut items = Vec::new();
    for (i, line) in trimmed.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let item: BatchItem =
            serde_json::from_str(line).map_err(|e| format!("NDJSON line {}: {e}", i + 1))?;
        items.push(item);
    }
    Ok(items)
}

fn resolve_systems(item: &BatchItem) -> Result<Vec<gematra_ciphers::SystemId>, String> {
    match &item.systems {
        None => Ok(vec![]), // empty means applicable
        Some(list) if list.is_empty() => Ok(vec![]),
        Some(list) => {
            let mut out = Vec::with_capacity(list.len());
            for id in list {
                out.push(
                    parse_system(id)
                        .map_err(|e| format!("item {:?}: {e}", item.id))?,
                );
            }
            Ok(out)
        }
    }
}

fn run_batch(
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
        results.push(BatchResult {
            id: item.id,
            original_input: item.text,
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

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Calc {
            text,
            system,
            norm,
            pretty,
        } => {
            let text = read_text(text);
            let sys = match parse_system(&system) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{e}");
                    process::exit(1);
                }
            };
            let result = calculate(&text, sys, parse_norm(&norm));
            print_json(&result, pretty);
        }
        Commands::All { text, norm, pretty } => {
            let text = read_text(text);
            let results = calculate_applicable(&text, parse_norm(&norm));
            print_json(&results, pretty);
        }
        Commands::Batch {
            input,
            norm,
            pretty,
            totals_only,
        } => {
            let items = match load_batch_items(input) {
                Ok(i) => i,
                Err(e) => {
                    eprintln!("batch input error: {e}");
                    process::exit(2);
                }
            };
            match run_batch(items, parse_norm(&norm), totals_only) {
                Ok(v) => print_json(&v, pretty),
                Err(e) => {
                    eprintln!("batch error: {e}");
                    process::exit(1);
                }
            }
        }
        Commands::Systems { pretty } => {
            print_json(&all_system_meta(), pretty);
        }
        Commands::Fixtures { pretty } => {
            let results = gematra_fixtures::run_mandatory();
            let summary: Vec<_> = results
                .iter()
                .map(|(name, ok, expected, actual)| {
                    serde_json::json!({
                        "name": name,
                        "ok": ok,
                        "expected": expected,
                        "actual": actual,
                    })
                })
                .collect();
            let all_ok = results.iter().all(|(_, ok, _, _)| *ok);
            print_json(
                &serde_json::json!({ "all_ok": all_ok, "cases": summary }),
                pretty,
            );
            if !all_ok {
                process::exit(1);
            }
        }
        Commands::Version { json } => {
            if json {
                print_json(
                    &serde_json::json!({
                        "engine_version": ENGINE_VERSION,
                        "cipher_registry_version": gematra_ciphers::CIPHER_REGISTRY_VERSION,
                        "unicode_policy_version": gematra_unicode::UNICODE_POLICY_VERSION,
                    }),
                    true,
                );
            } else {
                println!("engine={}", ENGINE_VERSION);
                println!(
                    "cipher_registry={}",
                    gematra_ciphers::CIPHER_REGISTRY_VERSION
                );
                println!(
                    "unicode_policy={}",
                    gematra_unicode::UNICODE_POLICY_VERSION
                );
            }
        }
    }
}

fn print_json<T: serde::Serialize>(value: &T, pretty: bool) {
    if pretty {
        println!("{}", serde_json::to_string_pretty(value).unwrap());
    } else {
        println!("{}", serde_json::to_string(value).unwrap());
    }
}
