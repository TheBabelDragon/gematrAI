/**
 * Hebrew Gematria systems
 * Exact mappings documented; finals handled per system variant.
 */

// Standard (Mispar Hechrechi) — final letters keep regular values
const HEBREW_STANDARD = {
  'א': 1, 'ב': 2, 'ג': 3, 'ד': 4, 'ה': 5, 'ו': 6, 'ז': 7, 'ח': 8, 'ט': 9,
  'י': 10, 'כ': 20, 'ך': 20, 'ל': 30, 'מ': 40, 'ם': 40, 'נ': 50, 'ן': 50,
  'ס': 60, 'ע': 70, 'פ': 80, 'ף': 80, 'צ': 90, 'ץ': 90, 'ק': 100, 'ר': 200,
  'ש': 300, 'ת': 400
};

// Mispar Gadol — final letters receive elevated values
const HEBREW_GADOL = {
  'א': 1, 'ב': 2, 'ג': 3, 'ד': 4, 'ה': 5, 'ו': 6, 'ז': 7, 'ח': 8, 'ט': 9,
  'י': 10, 'כ': 20, 'ך': 500, 'ל': 30, 'מ': 40, 'ם': 600, 'נ': 50, 'ן': 700,
  'ס': 60, 'ע': 70, 'פ': 80, 'ף': 800, 'צ': 90, 'ץ': 900, 'ק': 100, 'ר': 200,
  'ש': 300, 'ת': 400
};

// Mispar Katan base (same as standard, then reduce)
const HEBREW_KATAN_BASE = { ...HEBREW_STANDARD };

function isHebrewLetter(ch) {
  return ch in HEBREW_STANDARD;
}

/**
 * Reduce a number to single digit (1-9), preserving 9 as 9.
 * Digital root style: n % 9 || 9
 */
function digitalRoot(n) {
  if (n === 0) return 0;
  const r = n % 9;
  return r === 0 ? 9 : r;
}

/**
 * Calculate Hebrew values for a string.
 * @param {string} text
 * @param {'standard'|'gadol'|'katan'} variant
 * @returns {{ total: number, breakdown: Array<{char: string, value: number|null}>, system: string }}
 */
export function calculateHebrew(text, variant = 'standard') {
  const map = variant === 'gadol' ? HEBREW_GADOL : HEBREW_STANDARD;
  const systemName =
    variant === 'gadol' ? 'Hebrew Mispar Gadol' :
    variant === 'katan' ? 'Hebrew Mispar Katan' :
    'Hebrew Mispar Hechrechi (Standard)';

  const breakdown = [];
  let total = 0;

  for (const ch of text) {
    if (isHebrewLetter(ch)) {
      let val = map[ch];
      if (variant === 'katan') {
        val = digitalRoot(val);
      }
      breakdown.push({ char: ch, value: val });
      total += val;
    } else {
      // Non-Hebrew letters: skip contribution but record for display
      breakdown.push({ char: ch, value: null });
    }
  }

  if (variant === 'katan') {
    // Final total also reduced in classic Mispar Katan of the whole word
    // (some traditions reduce letter-wise only; we do letter-wise + report both)
    // Here we keep the summed reduced-letter total as primary.
  }

  return { total, breakdown, system: systemName, variant };
}

export function getHebrewLetterValue(ch, variant = 'standard') {
  const map = variant === 'gadol' ? HEBREW_GADOL : HEBREW_STANDARD;
  if (!(ch in map)) return null;
  let v = map[ch];
  if (variant === 'katan') v = digitalRoot(v);
  return v;
}

export { HEBREW_STANDARD, HEBREW_GADOL, isHebrewLetter, digitalRoot };
