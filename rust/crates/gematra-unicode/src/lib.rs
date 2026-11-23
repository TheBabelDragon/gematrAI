//! Unicode processing for gematrAI.
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;
use unicode_script::{Script, UnicodeScript};
use unicode_segmentation::UnicodeSegmentation;

pub const UNICODE_POLICY_VERSION: &str = "1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum NormalizationForm {
    None,
    #[default]
    Nfc,
    Nfd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScriptClass {
    Hebrew,
    Greek,
    Latin,
    Common,
    Other,
}

impl From<Script> for ScriptClass {
    fn from(s: Script) -> Self {
        match s {
            Script::Hebrew => ScriptClass::Hebrew,
            Script::Greek => ScriptClass::Greek,
            Script::Latin => ScriptClass::Latin,
            Script::Common | Script::Inherited => ScriptClass::Common,
            _ => ScriptClass::Other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExclusionReason {
    NonLetter,
    HebrewNiqqud,
    GreekDiacritic,
    WrongScript,
    Unmapped,
    Unsupported,
    FormatControl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharUnicodeInfo {
    pub original: String,
    pub scalar_values: Vec<u32>,
    pub normalized: String,
    pub script: ScriptClass,
    pub is_combining: bool,
    pub is_hebrew_niqqud: bool,
    pub is_greek_diacritic: bool,
    pub is_format: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnicodeAnalysis {
    pub original_input: String,
    pub policy_version: String,
    pub normalization: NormalizationForm,
    pub characters: Vec<CharUnicodeInfo>,
    pub has_hebrew: bool,
    pub has_greek: bool,
    pub has_latin: bool,
    pub grapheme_count: usize,
}

fn is_hebrew_mark(cp: u32) -> bool {
    (0x0591..=0x05C7).contains(&cp) || cp == 0xFB1E
}
fn is_greek_combining(cp: u32) -> bool {
    (0x0300..=0x036F).contains(&cp)
}
fn is_format_control(cp: u32) -> bool {
    matches!(cp, 0x200B..=0x200F | 0x202A..=0x202E | 0x2060..=0x2064 | 0xFEFF)
}

pub fn strip_combining_marks(s: &str) -> String {
    s.nfd()
        .filter(|c| !(0x0300..=0x036F).contains(&(*c as u32)))
        .collect()
}

pub fn analyze(input: &str, form: NormalizationForm) -> UnicodeAnalysis {
    let mut characters = Vec::new();
    let mut has_hebrew = false;
    let mut has_greek = false;
    let mut has_latin = false;
    for grapheme in input.graphemes(true) {
        let scalar_values: Vec<u32> = grapheme.chars().map(|c| c as u32).collect();
        let primary_cp = scalar_values.first().copied().unwrap_or(0);
        let primary_char = char::from_u32(primary_cp).unwrap_or('\u{FFFD}');
        let script = ScriptClass::from(primary_char.script());
        match script {
            ScriptClass::Hebrew => has_hebrew = true,
            ScriptClass::Greek => has_greek = true,
            ScriptClass::Latin => has_latin = true,
            _ => {}
        }
        let is_combining = grapheme.chars().any(|c| {
            let cp = c as u32;
            (0x0300..=0x036F).contains(&cp) || is_hebrew_mark(cp)
        });
        let is_hebrew_niqqud = scalar_values.iter().any(|&cp| is_hebrew_mark(cp));
        let is_greek_diacritic = scalar_values.iter().any(|&cp| is_greek_combining(cp));
        let is_format = scalar_values.iter().any(|&cp| is_format_control(cp));
        let normalized = match form {
            NormalizationForm::None => grapheme.to_string(),
            NormalizationForm::Nfc => grapheme.nfc().collect::<String>(),
            NormalizationForm::Nfd => grapheme.nfd().collect::<String>(),
        };
        characters.push(CharUnicodeInfo {
            original: grapheme.to_string(),
            scalar_values,
            normalized,
            script,
            is_combining,
            is_hebrew_niqqud,
            is_greek_diacritic,
            is_format,
        });
    }
    UnicodeAnalysis {
        original_input: input.to_string(),
        policy_version: UNICODE_POLICY_VERSION.to_string(),
        normalization: form,
        grapheme_count: characters.len(),
        characters,
        has_hebrew,
        has_greek,
        has_latin,
    }
}

pub fn analyze_nfc(input: &str) -> UnicodeAnalysis {
    analyze(input, NormalizationForm::Nfc)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn policy_version_is_set() {
        assert_eq!(UNICODE_POLICY_VERSION, "1");
    }
    #[test]
    fn herro_scripts() {
        let a = analyze_nfc("herro prease");
        assert!(a.has_latin);
    }
}
