/**
 * English / Latin gematria systems
 * - Ordinal: A=1 … Z=26
 * - Reduction (Pythagorean / simple): A=1…I=9, J=1…R=9, S=1…Z=8
 * - Full reduction of total also available via digital root
 */

const ORDINAL = {};
const REDUCTION = {};

for (let i = 0; i < 26; i++) {
  const upper = String.fromCharCode(65 + i);
  const lower = String.fromCharCode(97 + i);
  const ord = i + 1;
  const red = ((i % 9) + 1);
  ORDINAL[upper] = ord;
  ORDINAL[lower] = ord;
  REDUCTION[upper] = red;
  REDUCTION[lower] = red;
}

function isLatinLetter(ch) {
  return /[A-Za-z]/.test(ch);
}

function digitalRoot(n) {
  if (n === 0) return 0;
  const r = n % 9;
  return r === 0 ? 9 : r;
}

/**
 * @param {string} text
 * @param {'ordinal'|'reduction'|'pythagorean'} variant
 * @returns {{ total: number, breakdown: Array<{char: string, value: number|null}>, system: string }}
 */
export function calculateEnglish(text, variant = 'ordinal') {
  const map = variant === 'ordinal' ? ORDINAL : REDUCTION;
  const systemName =
    variant === 'ordinal' ? 'English Ordinal' :
    variant === 'pythagorean' ? 'English Pythagorean Reduction' :
    'English Reduction';

  const breakdown = [];
  let total = 0;

  for (const ch of text) {
    if (isLatinLetter(ch)) {
      const val = map[ch];
      breakdown.push({ char: ch, value: val });
      total += val;
    } else {
      breakdown.push({ char: ch, value: null });
    }
  }

  return {
    total,
    breakdown,
    system: systemName,
    variant,
    digitalRoot: digitalRoot(total)
  };
}

export function getEnglishLetterValue(ch, variant = 'ordinal') {
  if (!isLatinLetter(ch)) return null;
  return (variant === 'ordinal' ? ORDINAL : REDUCTION)[ch];
}

export { ORDINAL, REDUCTION, isLatinLetter, digitalRoot };
