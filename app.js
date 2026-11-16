import { SYSTEMS, getSystem } from './gematria/systems.js';
import { CORPUS } from './data/corpus.js';
import { isHebrewLetter } from './gematria/hebrew.js';
import { isGreekLetter } from './gematria/greek.js';
import { isLatinLetter } from './gematria/english.js';
import { initEngine, calculateAllForDashboard, calculateOne, engineReady } from './js/engine-adapter.js';

/** True only after WASM init succeeds. No silent JS calculation fallback. */
let engineOk = false;

window.__gematra = {
  get engineOk() { return engineOk; },
  calculateAllForDashboard,
  calculateOne,
  getSystem,
  SYSTEMS,
  CORPUS,
  isHebrewLetter,
  isGreekLetter,
  isLatinLetter,
};

const resultsEl = document.querySelector('#results');
const queryInput = document.querySelector('#query-input');
if (resultsEl) resultsEl.innerHTML = '<div class="empty-state">Loading calculation engine…</div>';
if (queryInput) queryInput.disabled = true;

initEngine('../wasm/')
  .then(() => {
    if (!engineReady()) throw new Error('engine not ready after init');
    engineOk = true;
    window.__gematra.engineOkFlag = true;
    return import('./js/dashboard.js');
  })
  .then(() => {
    if (queryInput) queryInput.disabled = false;
  })
  .catch((e) => {
    console.error('gematrAI engine init', e);
    const msg = (e && e.message) ? e.message : String(e);
    if (resultsEl) {
      resultsEl.innerHTML =
        '<div class="empty-state">WASM engine failed to load — no silent JS fallback.<br>' +
        '<span style="font-size:0.75rem;opacity:0.8">' +
        msg.replace(/&/g, '&').replace(/</g, '<') +
        '</span></div>';
    }
  });
