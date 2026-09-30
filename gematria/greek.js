/**
 * Greek Isopsephy
 * Standard classical values (α=1 … ω=800).
 * Final sigma ς treated as 200 (same as σ).
 * Accented letters are normalized (NFD + strip combining marks) for lookup;
 * original characters are preserved in the breakdown for display.
 */

const GREEK_ISOPSEPHY = {
  'α': 1, 'Α': 1, 'β': 2, 'Β': 2, 'γ': 3, 'Γ': 3, 'δ': 4, 'Δ': 4,
  'ε': 5, 'Ε': 5, 'ζ': 7, 'Ζ': 7, 'η': 8, 'Η': 8, 'θ': 9, 'Θ': 9,
  'ι': 10, 'Ι': 10, 'κ': 20, 'Κ': 20, 'λ': 30, 'Λ': 30, 'μ': 40, 'Μ': 40,
  'ν': 50, 'Ν': 50, 'ξ': 60, 'Ξ': 60, 'ο': 70, 'Ο': 70, 'π': 80, 'Π': 80,
  'ρ': 100, 'Ρ': 100, 'σ': 200, 'Σ': 200, 'ς': 200, 'τ': 300, 'Τ': 300,
  'υ': 400, 'Υ': 400, 'φ': 500, 'Φ': 500, 'χ': 600, 'Χ': 600,
  'ψ': 700, 'Ψ': 700, 'ω': 800, 'Ω': 800
  // Note: digamma ϝ/ϛ historically 6, koppa ϟ 90, sampi ϡ 900 — omitted unless present
};

/** Strip combining diacritics for value lookup while preserving display char */
function baseGreek(ch) {
  return ch.normalize('NFD').replace(/[\u0300-\u036f]/g, '');
}

function isGreekLetter(ch) {
  return baseGreek(ch) in GREEK_ISOPSEPHY || ch in GREEK_ISOPSEPHY;
}

/**
 * @param {string} text
 * @returns {{ total: number, breakdown: Array<{char: string, value: number|null}>, system: string }}
 */
export function calculateGreek(text) {
  const breakdown = [];
  let total = 0;

  for (const ch of text) {
    const base = baseGreek(ch);
    const val = GREEK_ISOPSEPHY[ch] ?? GREEK_ISOPSEPHY[base];
    if (val !== undefined) {
      breakdown.push({ char: ch, value: val });
      total += val;
    } else {
      breakdown.push({ char: ch, value: null });
    }
  }

  return {
    total,
    breakdown,
    system: 'Greek Isopsephy (Standard)',
    variant: 'isopsephy'
  };
}

export function getGreekLetterValue(ch) {
  const base = baseGreek(ch);
  return GREEK_ISOPSEPHY[ch] ?? GREEK_ISOPSEPHY[base] ?? null;
}

export { GREEK_ISOPSEPHY, isGreekLetter };
