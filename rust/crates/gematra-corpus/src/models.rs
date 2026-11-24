//! Segment, token, phrase, and index record types.
use serde::{Deserialize, Serialize};

pub const CORPUS_SCHEMA_VERSION: &str = "1";
pub const TOWER_VERSION: &str = "0.3";

/// Input segment for corpus ingestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentInput {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub r#ref: Option<String>,
    #[serde(alias = "ref")]
    #[serde(default)]
    pub reference: Option<String>,
    #[serde(default)]
    pub book: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(alias = "text_original")]
    pub text: String,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default, alias = "sourceUrl")]
    pub source_url: Option<String>,
    /// "seed" | "text" | "fixture"
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

impl SegmentInput {
    pub fn reference_str(&self) -> String {
        self.r#ref
            .clone()
            .or_else(|| self.reference.clone())
            .unwrap_or_default()
    }
    pub fn kind_str(&self) -> &str {
        self.kind.as_deref().unwrap_or("text")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRecord {
    pub normalized: String,
    pub surface: String,
    pub language: String,
    pub system: String,
    pub value: u32,
    pub r#ref: String,
    pub book: String,
    pub license: String,
    #[serde(rename = "sourceUrl")]
    pub source_url: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhraseRecord {
    pub normalized: String,
    pub surface: String,
    pub system: String,
    pub value: u32,
    pub r#ref: String,
    pub book: String,
    pub n: u32,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexItem {
    /// Canonical term
    pub t: String,
    /// Display surface
    pub s: String,
    /// Occurrence count
    pub c: u32,
    /// Source references (capped)
    pub r: Vec<String>,
    /// Books
    pub b: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub lic: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub kind: String,
}

#[derive(Debug, Clone)]
pub struct CorpusBuildConfig {
    pub phrase_max_n: u32,
    pub bucket_size: u32,
    pub max_refs_per_term: usize,
}

impl Default for CorpusBuildConfig {
    fn default() -> Self {
        Self {
            phrase_max_n: 2,
            bucket_size: 100,
            max_refs_per_term: 8,
        }
    }
}
