/**
 * gematrAI calculation engine tests
 * Open tests/runner.html in a browser (via local server) for results.
 */
import { calculateHebrew, digitalRoot } from '../gematria/hebrew.js';
import { calculateGreek } from '../gematria/greek.js';
import { calculateEnglish } from '../gematria/english.js';
import { calculateAll } from '../gematria/systems.js';

const results = [];
let passed = 0;
let failed = 0;

function assert(name, condition, detail = '') {
  if (condition) { passed++; results.push({ name, ok: true }); console.log('  OK', name); }
  else { failed++; results.push({ name, ok: false, detail }); console.error('  FAIL', name, detail); }
}
function assertEq(name, actual, expected) {
  assert(name, actual === expected, 'got ' + actual + ', expected ' + expected);
}

console.log('=== Hebrew Standard ===');
assertEq('alef=1', calculateHebrew('\u05D0', 'standard').total, 1);
assertEq('chai=18', calculateHebrew('\u05D7\u05D9', 'standard').total, 18);
assertEq('ahava=13', calculateHebrew('\u05D0\u05D4\u05D1\u05D4', 'standard').total, 13);
assertEq('final kaf=20', calculateHebrew('\u05DA', 'standard').total, 20);
assertEq('final mem=40', calculateHebrew('\u05DD', 'standard').total, 40);

console.log('=== Hebrew Gadol ===');
assertEq('final kaf gadol=500', calculateHebrew('\u05DA', 'gadol').total, 500);
assertEq('final mem gadol=600', calculateHebrew('\u05DD', 'gadol').total, 600);

console.log('=== Hebrew Katan ===');
assertEq('yod katan=1', calculateHebrew('\u05D9', 'katan').total, 1);
assertEq('chai katan=9', calculateHebrew('\u05D7\u05D9', 'katan').total, 9);

console.log('=== English ===');
assertEq('love ordinal=54', calculateEnglish('love', 'ordinal').total, 54);
assertEq('love reduction=18', calculateEnglish('love', 'reduction').total, 18);

console.log('=== Greek ===');
assertEq('final sigma=200', calculateGreek('\u03C2').total, 200);
assertEq('omega=800', calculateGreek('\u03C9').total, 800);

console.log('=== digitalRoot ===');
assertEq('dr18=9', digitalRoot(18), 9);
assertEq('dr9=9', digitalRoot(9), 9);

console.log('=== Systems ===');
const all = calculateAll('\u05D0\u05D4\u05D1\u05D4');
assert('hebrew systems present', all.filter(r => r.system.language === 'hebrew').length >= 3);

console.log('Passed:', passed, 'Failed:', failed);
export { passed, failed, results };
