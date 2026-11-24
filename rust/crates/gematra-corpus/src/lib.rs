//! Corpus engine — tokenization, Rust-authoritative calculation, reverse index.
//!
//! Numerical values always come from `gematra-core` / `gematra-ciphers` / `gematra-unicode`.

mod models;
mod tokenize;
mod calc;
mod index;
mod build;
mod io;

pub use models::*;
pub use tokenize::*;
pub use calc::{calculate_token_systems, process_segment};
pub use index::{build_index, BuiltIndex};
pub use build::build_corpus;
pub use io::{load_segments_from_path, load_segments_from_str};

#[cfg(test)]
mod tests;
