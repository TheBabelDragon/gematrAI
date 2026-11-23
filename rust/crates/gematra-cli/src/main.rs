//! gematra CLI — native execution of the same calculation engine.
//!
//! See `docs/RUST_ENGINE.md` for the batch output schema and safety limits.
use clap::{Parser, Subcommand};
use gematra_ciphers::all_system_meta;
use gematra_cli::{
    load_batch_items, parse_norm, run_batch, BatchLimits, DEFAULT_MAX_INPUT_BYTES,
    DEFAULT_MAX_ITEMS, DEFAULT_MAX_TEXT_BYTES,
};
use gematra_core::{calculate, calculate_applicable, parse_system, ENGINE_VERSION};
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
        /// Normalization: nfc (default), nfd, or none
        #[arg(long, default_value = "nfc")]
        norm: String,
        #[arg(long)]
        pretty: bool,
    },
    /// Calculate all applicable systems for a single text
    All {
        text: Option<String>,
        /// Normalization: nfc (default), nfd, or none
        #[arg(long, default_value = "nfc")]
        norm: String,
        #[arg(long)]
        pretty: bool,
    },
    /// Batch calculate many inputs in one process invocation.
    ///
    /// Input (stdin or --input file): JSON array or NDJSON.
    /// Failure is atomic: no partial JSON on stdout.
    Batch {
        #[arg(short, long)]
        input: Option<PathBuf>,
        /// Normalization: nfc (default), nfd, or none
        #[arg(long, default_value = "nfc")]
        norm: String,
        #[arg(long)]
        pretty: bool,
        #[arg(long)]
        totals_only: bool,
        #[arg(long, default_value_t = DEFAULT_MAX_INPUT_BYTES)]
        max_input_bytes: usize,
        #[arg(long, default_value_t = DEFAULT_MAX_ITEMS)]
        max_items: usize,
        #[arg(long, default_value_t = DEFAULT_MAX_TEXT_BYTES)]
        max_text_bytes: usize,
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

fn read_text(arg: Option<String>) -> String {
    if let Some(t) = arg {
        return t;
    }
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).expect("read stdin");
    buf
}

fn require_norm(s: &str) -> gematra_unicode::NormalizationForm {
    match parse_norm(s) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{e}");
            process::exit(2);
        }
    }
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
            let result = calculate(&text, sys, require_norm(&norm));
            print_json(&result, pretty);
        }
        Commands::All { text, norm, pretty } => {
            let text = read_text(text);
            let results = calculate_applicable(&text, require_norm(&norm));
            print_json(&results, pretty);
        }
        Commands::Batch {
            input,
            norm,
            pretty,
            totals_only,
            max_input_bytes,
            max_items,
            max_text_bytes,
        } => {
            let limits = BatchLimits {
                max_input_bytes,
                max_items,
                max_text_bytes,
            };
            let norm = require_norm(&norm);
            let items = match load_batch_items(input, &limits) {
                Ok(i) => i,
                Err(e) => {
                    eprintln!("batch input error: {e}");
                    process::exit(2);
                }
            };
            match run_batch(items, norm, totals_only) {
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
                print_json(&gematra_cli::engine_metadata(), true);
            } else {
                let m = gematra_cli::engine_metadata();
                println!("engine={}", m["engine_version"].as_str().unwrap_or(""));
                println!(
                    "cipher_registry={}",
                    m["cipher_registry_version"].as_str().unwrap_or("")
                );
                println!(
                    "unicode_policy={}",
                    m["unicode_policy_version"].as_str().unwrap_or("")
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
