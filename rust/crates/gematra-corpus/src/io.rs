//! Segment JSON / NDJSON loading.
use std::fs;
use std::path::Path;

use crate::models::SegmentInput;

pub fn load_segments_from_str(text: &str) -> Result<Vec<SegmentInput>, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(vec![]);
    }
    if trimmed.starts_with('[') {
        serde_json::from_str(trimmed).map_err(|e| format!("JSON array: {e}"))
    } else {
        let mut out = Vec::new();
        for (i, line) in trimmed.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let seg: SegmentInput = serde_json::from_str(line)
                .map_err(|e| format!("NDJSON line {}: {e}", i + 1))?;
            out.push(seg);
        }
        Ok(out)
    }
}

pub fn load_segments_from_path(path: &Path) -> Result<Vec<SegmentInput>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    load_segments_from_str(&text)
}
