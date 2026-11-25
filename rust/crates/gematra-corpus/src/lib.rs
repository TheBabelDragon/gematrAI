//! Corpus engine — tokenization, Rust-authoritative calculation, reverse index.
//!
//! Numerical values always come from `gematra-core` / `gematra-ciphers` / `gematra-unicode`.

mod build;
mod calc;
mod index;
mod io;
mod models;
mod tokenize;

pub use build::build_corpus;
pub use calc::{calculate_token_systems, process_segment};
pub use index::{build_index, BuiltIndex};
pub use io::{load_segments_from_path, load_segments_from_str};
pub use models::*;
pub use tokenize::*;

#[cfg(test)]
mod tests;
