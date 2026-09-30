/**
 * Unified gematria systems registry.
 * Each system exposes: id, name, language, calculate(text) → result
 */

import { calculateHebrew, isHebrewLetter } from './hebrew.js';
import { calculateGreek, isGreekLetter } from './greek.js';
import { calculateEnglish, isLatinLetter } from './english.js';

export const SYSTEMS = [
  {
    id: 'hebrew-standard',
    name: 'Hebrew Mispar Hechrechi (Standard)',
    short: 'Hechrechi',
    language: 'hebrew',
    description: 'Standard values; final letters (ךםןףץ) keep regular values (20/40/50/80/90).',
    calculate: (text) => calculateHebrew(text, 'standard')
  },
  {
    id: 'hebrew-gadol',
    name: 'Hebrew Mispar Gadol',
    short: 'Gadol',
    language: 'hebrew',
    description: 'Final letters elevated: ך=500, ם=600, ן=700, ף=800, ץ=900.',
    calculate: (text) => calculateHebrew(text, 'gadol')
  },
  {
    id: 'hebrew-katan',
    name: 'Hebrew Mispar Katan',
    short: 'Katan',
    language: 'hebrew',
    description: 'Each letter reduced to single digit (1–9) before summing.',
    calculate: (text) => calculateHebrew(text, 'katan')
  },
  {
    id: 'english-ordinal',
    name: 'English Ordinal',
    short: 'Ordinal',
    language: 'english',
    description: 'A=1, B=2 … Z=26. Case-insensitive.',
    calculate: (text) => calculateEnglish(text, 'ordinal')
  },
  {
    id: 'english-reduction',
    name: 'English Reduction',
    short: 'Reduction',
    language: 'english',
    description: 'A=1…I=9, J=1…R=9, S=1…Z=8 (mod-9 style).',
    calculate: (text) => calculateEnglish(text, 'reduction')
  },
  {
    id: 'english-pythagorean',
    name: 'English Pythagorean Reduction',
    short: 'Pythagorean',
    language: 'english',
    description: 'Identical mapping to English Reduction (classic Pythagorean).',
    calculate: (text) => calculateEnglish(text, 'pythagorean')
  },
  {
    id: 'greek-isopsephy',
    name: 'Greek Isopsephy (Standard)',
    short: 'Isopsephy',
    language: 'greek',
    description: 'Classical values α=1 … ω=800. Final sigma ς = 200.',
    calculate: (text) => calculateGreek(text)
  }
];

export function getSystem(id) {
  return SYSTEMS.find(s => s.id === id) || null;
}

/**
 * Detect which scripts are present and return applicable systems.
 * Always returns all systems that can produce a meaningful total
 * (i.e. systems whose language letters appear, plus English for any Latin).
 */
export function applicableSystems(text) {
  let hasHebrew = false, hasGreek = false, hasLatin = false;
  for (const ch of text) {
    if (isHebrewLetter(ch)) hasHebrew = true;
    if (isGreekLetter(ch)) hasGreek = true;
    if (isLatinLetter(ch)) hasLatin = true;
  }
  return SYSTEMS.filter(s => {
    if (s.language === 'hebrew') return hasHebrew;
    if (s.language === 'greek') return hasGreek;
    if (s.language === 'english') return hasLatin || (!hasHebrew && !hasGreek);
    return true;
  });
}

/**
 * Calculate all applicable systems for a text.
 * @returns {Array<{system: object, result: object}>}
 */
export function calculateAll(text) {
  const apps = applicableSystems(text);
  // If pure Latin or mixed empty, still show English systems
  const list = apps.length ? apps : SYSTEMS.filter(s => s.language === 'english');
  return list.map(sys => ({
    system: sys,
    result: sys.calculate(text)
  }));
}

export function calculateOne(text, systemId) {
  const sys = getSystem(systemId);
  if (!sys) return null;
  return { system: sys, result: sys.calculate(text) };
}
