//! Cipher registry for gematrAI. Registry version: 1
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const CIPHER_REGISTRY_VERSION: &str = "1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SystemId {
    #[serde(rename = "hebrew-standard")] HebrewStandard,
    #[serde(rename = "hebrew-gadol")] HebrewGadol,
    #[serde(rename = "hebrew-katan")] HebrewKatan,
    #[serde(rename = "english-ordinal")] EnglishOrdinal,
    #[serde(rename = "english-reduction")] EnglishReduction,
    #[serde(rename = "english-pythagorean")] EnglishPythagorean,
    #[serde(rename = "greek-isopsephy")] GreekIsopsephy,
}

impl SystemId {
    pub fn as_str(self) -> &'static str {
        match self {
            SystemId::HebrewStandard => "hebrew-standard",
            SystemId::HebrewGadol => "hebrew-gadol",
            SystemId::HebrewKatan => "hebrew-katan",
            SystemId::EnglishOrdinal => "english-ordinal",
            SystemId::EnglishReduction => "english-reduction",
            SystemId::EnglishPythagorean => "english-pythagorean",
            SystemId::GreekIsopsephy => "greek-isopsephy",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "hebrew-standard" => Some(SystemId::HebrewStandard),
            "hebrew-gadol" => Some(SystemId::HebrewGadol),
            "hebrew-katan" => Some(SystemId::HebrewKatan),
            "english-ordinal" => Some(SystemId::EnglishOrdinal),
            "english-reduction" => Some(SystemId::EnglishReduction),
            "english-pythagorean" => Some(SystemId::EnglishPythagorean),
            "greek-isopsephy" => Some(SystemId::GreekIsopsephy),
            _ => None,
        }
    }
    pub fn language(self) -> &'static str {
        match self {
            SystemId::HebrewStandard | SystemId::HebrewGadol | SystemId::HebrewKatan => "hebrew",
            SystemId::EnglishOrdinal | SystemId::EnglishReduction | SystemId::EnglishPythagorean => "english",
            SystemId::GreekIsopsephy => "greek",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            SystemId::HebrewStandard => "Hebrew Mispar Hechrechi (Standard)",
            SystemId::HebrewGadol => "Hebrew Mispar Gadol",
            SystemId::HebrewKatan => "Hebrew Mispar Katan",
            SystemId::EnglishOrdinal => "English Ordinal",
            SystemId::EnglishReduction => "English Reduction",
            SystemId::EnglishPythagorean => "English Pythagorean Reduction",
            SystemId::GreekIsopsephy => "Greek Isopsephy (Standard)",
        }
    }
    pub fn short(self) -> &'static str {
        match self {
            SystemId::HebrewStandard => "Hechrechi",
            SystemId::HebrewGadol => "Gadol",
            SystemId::HebrewKatan => "Katan",
            SystemId::EnglishOrdinal => "Ordinal",
            SystemId::EnglishReduction => "Reduction",
            SystemId::EnglishPythagorean => "Pythagorean",
            SystemId::GreekIsopsephy => "Isopsephy",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            SystemId::HebrewStandard => "Standard values; final letters keep regular values.",
            SystemId::HebrewGadol => "Final letters elevated: ך=500…ץ=900.",
            SystemId::HebrewKatan => "Each letter reduced to single digit (1–9) before summing.",
            SystemId::EnglishOrdinal => "A=1 … Z=26. Case-insensitive.",
            SystemId::EnglishReduction => "A=1…I=9, J=1…R=9, S=1…Z=8.",
            SystemId::EnglishPythagorean => "Identical mapping to English Reduction.",
            SystemId::GreekIsopsephy => "Classical α=1 … ω=800; final sigma ς=200.",
        }
    }
    pub fn all() -> &'static [SystemId] {
        &[
            SystemId::HebrewStandard, SystemId::HebrewGadol, SystemId::HebrewKatan,
            SystemId::EnglishOrdinal, SystemId::EnglishReduction, SystemId::EnglishPythagorean,
            SystemId::GreekIsopsephy,
        ]
    }
}

pub fn digital_root(n: u32) -> u32 {
    if n == 0 { return 0; }
    let r = n % 9;
    if r == 0 { 9 } else { r }
}

fn hebrew_standard() -> HashMap<char, u32> {
    [('א',1),('ב',2),('ג',3),('ד',4),('ה',5),('ו',6),('ז',7),('ח',8),('ט',9),
     ('י',10),('כ',20),('ך',20),('ל',30),('מ',40),('ם',40),('נ',50),('ן',50),
     ('ס',60),('ע',70),('פ',80),('ף',80),('צ',90),('ץ',90),('ק',100),('ר',200),
     ('ש',300),('ת',400)].into_iter().collect()
}
fn hebrew_gadol() -> HashMap<char, u32> {
    let mut m = hebrew_standard();
    m.insert('ך',500); m.insert('ם',600); m.insert('ן',700); m.insert('ף',800); m.insert('ץ',900);
    m
}
fn english_ordinal() -> HashMap<char, u32> {
    let mut m = HashMap::new();
    for i in 0..26u32 {
        m.insert(char::from_u32(65+i).unwrap(), i+1);
        m.insert(char::from_u32(97+i).unwrap(), i+1);
    }
    m
}
fn english_reduction() -> HashMap<char, u32> {
    let mut m = HashMap::new();
    for i in 0..26u32 {
        let r = (i % 9) + 1;
        m.insert(char::from_u32(65+i).unwrap(), r);
        m.insert(char::from_u32(97+i).unwrap(), r);
    }
    m
}
fn greek_map() -> HashMap<char, u32> {
    [('α',1),('Α',1),('β',2),('Β',2),('γ',3),('Γ',3),('δ',4),('Δ',4),
     ('ε',5),('Ε',5),('ζ',7),('Ζ',7),('η',8),('Η',8),('θ',9),('Θ',9),
     ('ι',10),('Ι',10),('κ',20),('Κ',20),('λ',30),('Λ',30),('μ',40),('Μ',40),
     ('ν',50),('Ν',50),('ξ',60),('Ξ',60),('ο',70),('Ο',70),('π',80),('Π',80),
     ('ρ',100),('Ρ',100),('σ',200),('Σ',200),('ς',200),('τ',300),('Τ',300),
     ('υ',400),('Υ',400),('φ',500),('Φ',500),('χ',600),('Χ',600),
     ('ψ',700),('Ψ',700),('ω',800),('Ω',800)].into_iter().collect()
}

pub fn is_hebrew_letter(ch: char) -> bool { hebrew_standard().contains_key(&ch) }
pub fn is_latin_letter(ch: char) -> bool { ch.is_ascii_alphabetic() }
pub fn is_greek_letter(ch: char) -> bool {
    let map = greek_map();
    if map.contains_key(&ch) { return true; }
    let base: String = gematra_unicode::strip_combining_marks(&ch.to_string());
    base.chars().next().map(|c| map.contains_key(&c)).unwrap_or(false)
}

pub fn letter_value(system: SystemId, ch: char) -> Option<u32> {
    match system {
        SystemId::HebrewStandard => hebrew_standard().get(&ch).copied(),
        SystemId::HebrewGadol => hebrew_gadol().get(&ch).copied(),
        SystemId::HebrewKatan => hebrew_standard().get(&ch).copied().map(digital_root),
        SystemId::EnglishOrdinal => english_ordinal().get(&ch).copied(),
        SystemId::EnglishReduction | SystemId::EnglishPythagorean => english_reduction().get(&ch).copied(),
        SystemId::GreekIsopsephy => {
            let map = greek_map();
            if let Some(v) = map.get(&ch) { return Some(*v); }
            let base = gematra_unicode::strip_combining_marks(&ch.to_string());
            base.chars().next().and_then(|c| map.get(&c).copied())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMeta {
    pub id: String, pub name: String, pub short: String, pub language: String, pub description: String,
}

pub fn system_meta(id: SystemId) -> SystemMeta {
    SystemMeta {
        id: id.as_str().to_string(), name: id.name().to_string(), short: id.short().to_string(),
        language: id.language().to_string(), description: id.description().to_string(),
    }
}
pub fn all_system_meta() -> Vec<SystemMeta> {
    SystemId::all().iter().map(|s| system_meta(*s)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn herro_letters() {
        assert_eq!(letter_value(SystemId::EnglishOrdinal, 'H'), Some(8));
        assert_eq!(letter_value(SystemId::EnglishOrdinal, 'e'), Some(5));
    }
    #[test]
    fn finals() {
        assert_eq!(letter_value(SystemId::HebrewStandard, 'ם'), Some(40));
        assert_eq!(letter_value(SystemId::HebrewGadol, 'ם'), Some(600));
    }
}
