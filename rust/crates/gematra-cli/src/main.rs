//! gematra CLI — native execution of the same calculation engine.
use clap::{Parser, Subcommand};
use gematra_ciphers::all_system_meta;
use gematra_core::{calculate, calculate_applicable, parse_system, ENGINE_VERSION};
use gematra_unicode::NormalizationForm;
use std::io::{self, Read};
use std::process;

#[derive(Parser)]
#[command(name = "gematra", version = ENGINE_VERSION, about = "gematrAI calculation engine (Rust authority)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Calc {
        text: Option<String>,
        #[arg(short, long)]
        system: String,
        #[arg(long, default_value = "nfc")]
        norm: String,
        #[arg(long)]
        pretty: bool,
    },
    All {
        text: Option<String>,
        #[arg(long, default_value = "nfc")]
        norm: String,
        #[arg(long)]
        pretty: bool,
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

fn parse_norm(s: &str) -> NormalizationForm {
    match s.to_ascii_lowercase().as_str() {
        "none" => NormalizationForm::None,
        "nfd" => NormalizationForm::Nfd,
        _ => NormalizationForm::Nfc,
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Calc { text, system, norm, pretty } => {
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
                println!("cipher_registry={}", gematra_ciphers::CIPHER_REGISTRY_VERSION);
                println!("unicode_policy={}", gematra_unicode::UNICODE_POLICY_VERSION);
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
