//! Language classification, canonical forms, and tokenization.
use gematra_unicode::{analyze, NormalizationForm};

/// Classify primary language of a string from script content + optional hint.
pub fn classify_language(text: &str, hint: Option<&str>) -> String {
    let analysis = analyze(text, NormalizationForm::Nfc);
    if analysis.has_hebrew {
        return "hebrew".into();
    }
    if analysis.has_greek {
        return "greek".into();
    }
    if analysis.has_latin {
        return "english".into();
    }
    match hint.map(|h| h.to_ascii_lowercase()).as_deref() {
        Some("he") | Some("hebrew") => "hebrew".into(),
        Some("el") | Some("greek") => "greek".into(),
        Some("en") | Some("english") | Some("la") | Some("latin") => "english".into(),
        _ => "unknown".into(),
    }
}

fn is_hebrew_letter(c: char) -> bool {
    matches!(c, '\u{05D0}'..='\u{05EA}')
}
fn is_greek_letter(c: char) -> bool {
    matches!(c, '\u{0370}'..='\u{03FF}' | '\u{1F00}'..='\u{1FFF}')
}
/// Strip niqqud / cantillation / combining marks for canonical lookup form.
pub fn canonical_form(text: &str, language: &str) -> String {
    let analysis = analyze(text, NormalizationForm::Nfc);
    let mut out = String::new();
    for info in &analysis.characters {
        for ch in info.normalized.chars() {
            let cp = ch as u32;
            if (0x0591..=0x05C7).contains(&cp) || cp == 0xFB1E {
                continue;
            }
            if (0x0300..=0x036F).contains(&cp) {
                continue;
            }
            if matches!(cp, 0x200B..=0x200F | 0x202A..=0x202E | 0x2060..=0x2064 | 0xFEFF) {
                continue;
            }
            match language {
                "english" => {
                    if ch.is_ascii_alphabetic() {
                        out.push(ch.to_ascii_lowercase());
                    }
                }
                "hebrew" => {
                    if is_hebrew_letter(ch) {
                        out.push(ch);
                    }
                }
                "greek" => {
                    if is_greek_letter(ch) {
                        // NFD + strip combining so ό → ο for lookup identity
                        let lower: String = ch.to_lowercase().collect();
                        let base = gematra_unicode::strip_combining_marks(&lower);
                        for c in base.chars() {
                            if is_greek_letter(c) {
                                out.push(c);
                            }
                        }
                    }
                }
                _ => {
                    if is_hebrew_letter(ch) {
                        out.push(ch);
                    } else if is_greek_letter(ch) {
                        let lower: String = ch.to_lowercase().collect();
                        let base = gematra_unicode::strip_combining_marks(&lower);
                        for c in base.chars() {
                            if is_greek_letter(c) {
                                out.push(c);
                            }
                        }
                    } else if ch.is_ascii_alphabetic() {
                        out.push(ch.to_ascii_lowercase());
                    }
                }
            }
        }
    }
    out
}

/// Tokenize text into (surface, normalized) pairs.
pub fn tokenize(text: &str, language: &str) -> Vec<(String, String)> {
    let mut tokens = Vec::new();
    let mut current_surface = String::new();
    let mut current_letters = String::new();

    let push_current =
        |surf: &mut String, letters: &mut String, out: &mut Vec<(String, String)>| {
            if letters.is_empty() {
                surf.clear();
                return;
            }
            let norm = canonical_form(letters, language);
            if !norm.is_empty() {
                let surface = if surf.is_empty() {
                    letters.clone()
                } else {
                    surf.clone()
                };
                out.push((surface, norm));
            }
            surf.clear();
            letters.clear();
        };

    for ch in text.chars() {
        let is_letter = match language {
            "hebrew" => is_hebrew_letter(ch) || (0x0591..=0x05C7).contains(&(ch as u32)),
            "greek" => is_greek_letter(ch) || (0x0300..=0x036F).contains(&(ch as u32)),
            "english" => ch.is_ascii_alphabetic() || ch == '\'',
            _ => {
                is_hebrew_letter(ch)
                    || is_greek_letter(ch)
                    || ch.is_ascii_alphabetic()
                    || (0x0591..=0x05C7).contains(&(ch as u32))
                    || (0x0300..=0x036F).contains(&(ch as u32))
            }
        };
        let is_boundary = matches!(
            ch,
            ' ' | '\t'
                | '\n'
                | '\r'
                | '\u{05BE}'
                | '-'
                | '\u{2010}'
                | '\u{2011}'
                | '\u{2012}'
                | '\u{2013}'
                | '\u{2014}'
        ) || (ch.is_ascii_punctuation() && ch != '\'');

        if is_letter && !is_boundary {
            current_surface.push(ch);
            current_letters.push(ch);
        } else {
            // boundary, whitespace, or other punctuation
            push_current(&mut current_surface, &mut current_letters, &mut tokens);
        }
    }
    push_current(&mut current_surface, &mut current_letters, &mut tokens);
    tokens
}
