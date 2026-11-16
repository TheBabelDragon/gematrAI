//! Shared deterministic fixtures.
//!
//! These fixtures are the contract between Rust unit tests, CLI tests,
//! WASM tests, and browser integration tests.

use gematra_ciphers::SystemId;
use gematra_core::{calculate_nfc, CalculationResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureCase {
    pub name: &'static str,
    pub input: &'static str,
    pub system: &'static str,
    pub expected_total: u32,
    pub notes: &'static str,
}

pub fn mandatory_cases() -> Vec<FixtureCase> {
    vec![
        FixtureCase {
            name: "herro_prease_ordinal",
            input: "herro prease",
            system: "english-ordinal",
            expected_total: 128,
            notes: "HERRO=64 PREASE=64 TOTAL=128 — mandatory regression",
        },
        FixtureCase {
            name: "herro_prease_upper",
            input: "HERRO PREASE",
            system: "english-ordinal",
            expected_total: 128,
            notes: "case-insensitive",
        },
        FixtureCase {
            name: "abc_ordinal",
            input: "abc",
            system: "english-ordinal",
            expected_total: 6,
            notes: "A=1 B=2 C=3",
        },
        FixtureCase {
            name: "shalom_standard",
            input: "שלום",
            system: "hebrew-standard",
            expected_total: 376,
            notes: "ש=300 ל=30 ו=6 ם=40",
        },
        FixtureCase {
            name: "final_mem_gadol",
            input: "ם",
            system: "hebrew-gadol",
            expected_total: 600,
            notes: "final mem elevated",
        },
        FixtureCase {
            name: "final_mem_standard",
            input: "ם",
            system: "hebrew-standard",
            expected_total: 40,
            notes: "final mem regular",
        },
        FixtureCase {
            name: "greek_alpha",
            input: "α",
            system: "greek-isopsephy",
            expected_total: 1,
            notes: "alpha",
        },
        FixtureCase {
            name: "greek_final_sigma",
            input: "ς",
            system: "greek-isopsephy",
            expected_total: 200,
            notes: "final sigma = 200",
        },
        FixtureCase {
            name: "empty",
            input: "",
            system: "english-ordinal",
            expected_total: 0,
            notes: "empty input",
        },
        FixtureCase {
            name: "whitespace_only",
            input: "   ",
            system: "english-ordinal",
            expected_total: 0,
            notes: "whitespace contributes nothing",
        },
        FixtureCase {
            name: "reduction_a",
            input: "a",
            system: "english-reduction",
            expected_total: 1,
            notes: "A=1",
        },
        FixtureCase {
            name: "pythagorean_z",
            input: "z",
            system: "english-pythagorean",
            expected_total: 8,
            notes: "Z=8 under reduction map",
        },
    ]
}

/// Run all mandatory fixtures and return (name, ok, expected, actual).
pub fn run_mandatory() -> Vec<(String, bool, u32, u32)> {
    let mut out = Vec::new();
    for case in mandatory_cases() {
        let sys = SystemId::from_str(case.system).expect("fixture system id");
        let result: CalculationResult = calculate_nfc(case.input, sys);
        let ok = result.total == case.expected_total;
        out.push((
            case.name.to_string(),
            ok,
            case.expected_total,
            result.total,
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_mandatory_pass() {
        let results = run_mandatory();
        for (name, ok, expected, actual) in &results {
            assert!(
                *ok,
                "fixture {name} failed: expected {expected}, got {actual}"
            );
        }
    }
}
