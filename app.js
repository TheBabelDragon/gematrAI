/**
 * gematrAI - client-side application controller
 * Pure static; no backend, no external AI API.
 */

import { SYSTEMS, calculateAll, calculateOne, getSystem } from './gematria/systems.js';
import { CORPUS, getCorpusSize } from './data/corpus.js';
import { isHebrewLetter } from './gematria/hebrew.js';
import { isGreekLetter } from './gematria/greek.js';
import { isLatinLetter } from './gematria/english.js';

const $ = (sel) => document.querySelector(sel);
const queryInput = $('#query-input');
const resultsEl = $('#results');
const charCountEl = $('#char-count');
const scriptHintEl = $('#script-hint');
const equivPanel = $('#equivalence-panel');
const equivList = $('#equiv-list');
const aiContent = $('#ai-content');
const reverseValue = $('#reverse-value');
const reverseSystem = $('#reverse-system');
const reverseBtn = $('#reverse-btn');
const matchList = $('#match-list');
const corpusSizeEl = $('#corpus-size');

let currentText = '';
let currentResults = [];
let primarySystemId = null;

function detectScripts(text) {
  let h = 0, g = 0, l = 0;
  for (const ch of text) {
    if (isHebrewLetter(ch)) h++;
    else if (isGreekLetter(ch)) g++;
    else if (isLatinLetter(ch)) l++;
  }
  const parts = [];
  if (h) parts.push(`Hebrewx${h}`);
  if (g) parts.push(`Greekx${g}`);
  if (l) parts.push(`Latinx${l}`);
  return parts.length ? parts.join(' · ') : '-';
}

function scriptClass(ch) {
  if (isHebrewLetter(ch)) return 'hebrew';
  if (isGreekLetter(ch)) return 'greek';
  if (isLatinLetter(ch)) return 'latin';
  return '';
}

function escapeHtml(s) {
  return String(s).replace(/&/g, '&').replace(/</g, '<').replace(/>/g, '>').replace(/"/g, '"');
}

function copyText(text) {
  navigator.clipboard.writeText(text).catch(() => {
    const ta = document.createElement('textarea');
    ta.value = text;
    document.body.appendChild(ta);
    ta.select();
    document.execCommand('copy');
    document.body.removeChild(ta);
  });
}

function encodeState() {
  const params = new URLSearchParams();
  if (currentText) params.set('q', currentText);
  if (primarySystemId) params.set('sys', primarySystemId);
  const hash = params.toString();
  history.replaceState(null, '', hash ? `#${hash}` : location.pathname);
}

function restoreState() {
  const hash = location.hash.slice(1);
  if (!hash) return;
  const params = new URLSearchParams(hash);
  const q = params.get('q');
  const sys = params.get('sys');
  if (q) {
    queryInput.value = q;
    if (sys && getSystem(sys)) primarySystemId = sys;
    runCalculation(q);
  }
}

function runCalculation(text) {
  currentText = text;
  charCountEl.textContent = `${[...text].length} chars`;
  scriptHintEl.textContent = detectScripts(text);
  if (!text.trim()) {
    resultsEl.innerHTML = '<div class="empty-state">Enter text to begin calculation</div>';
    equivPanel.style.display = 'none';
    aiContent.innerHTML = '<p class="empty-state" style="padding:0">Awaiting input...</p>';
    encodeState();
    return;
  }
  currentResults = calculateAll(text);
  if (!primarySystemId || !currentResults.find(r => r.system.id === primarySystemId)) {
    primarySystemId = currentResults[0]?.system.id || null;
  }
  renderResults();
  renderEquivalence();
  renderAI();
  encodeState();
}

function renderResults() {
  if (!currentResults.length) {
    resultsEl.innerHTML = '<div class="empty-state">No applicable systems for this input</div>';
    return;
  }
  resultsEl.innerHTML = currentResults.map(({ system, result }) => {
    const expanded = system.id === primarySystemId ? 'expanded' : '';
    const breakdownHtml = result.breakdown.map(b => {
      const cls = scriptClass(b.char);
      const valStr = b.value === null ? '-' : b.value;
      return `<span class="bd-item"><span class="bd-char ${cls}">${escapeHtml(b.char)}</span><span class="bd-val ${b.value === null ? 'null' : ''}">${valStr}</span></span>`;
    }).join('');
    return `
      <div class="result-card ${expanded}" data-sys="${system.id}">
        <div class="result-header" data-toggle="${system.id}">
          <div class="result-sys">${escapeHtml(system.name)}</div>
          <div class="result-total">${result.total}</div>
          <div class="result-actions">
            <button class="btn-icon" data-copy="${system.id}" title="Copy total">C</button>
            <button class="btn-icon" data-expand="${system.id}" title="Toggle details">${expanded ? 'v' : '>'}</button>
          </div>
        </div>
        <div class="result-body">
          <div class="breakdown">${breakdownHtml || '<span class="bd-val">No letter values</span>'}</div>
          <p style="margin-top:0.6rem;font-family:var(--mono);font-size:0.65rem;color:var(--text-dim)">${escapeHtml(system.description)}</p>
        </div>
      </div>`;
  }).join('');

  resultsEl.querySelectorAll('[data-toggle]').forEach(el => {
    el.addEventListener('click', (e) => {
      if (e.target.closest('[data-copy]')) return;
      const id = el.dataset.toggle;
      primarySystemId = id;
      resultsEl.querySelectorAll('.result-card').forEach(c => {
        const isThis = c.dataset.sys === id;
        c.classList.toggle('expanded', isThis);
        const btn = c.querySelector('[data-expand]');
        if (btn) btn.textContent = isThis ? 'v' : '>';
      });
      renderEquivalence();
      renderAI();
      encodeState();
    });
  });

  resultsEl.querySelectorAll('[data-copy]').forEach(btn => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const id = btn.dataset.copy;
      const r = currentResults.find(x => x.system.id === id);
      if (r) {
        copyText(String(r.result.total));
        btn.textContent = 'OK';
        setTimeout(() => { btn.textContent = 'C'; }, 900);
      }
    });
  });
}

function getCorpusValues(systemId) {
  const sys = getSystem(systemId);
  if (!sys) return [];
  return CORPUS.map(entry => {
    const res = sys.calculate(entry.term);
    return { ...entry, total: res.total, breakdown: res.breakdown };
  });
}

function renderEquivalence() {
  if (!currentText.trim() || !primarySystemId) {
    equivPanel.style.display = 'none';
    return;
  }
  const primary = currentResults.find(r => r.system.id === primarySystemId);
  if (!primary) { equivPanel.style.display = 'none'; return; }
  const target = primary.result.total;
  const matches = getCorpusValues(primarySystemId)
    .filter(e => e.total === target && e.term !== currentText.trim());
  equivPanel.style.display = 'block';
  if (!matches.length) {
    equivList.innerHTML = '<span style="font-family:var(--mono);font-size:0.75rem;color:var(--text-dim)">No other local-corpus matches for this value.</span>';
    return;
  }
  equivList.innerHTML = matches.slice(0, 24).map(m =>
    `<button class="equiv-chip" data-term="${escapeHtml(m.term)}" title="${escapeHtml(m.notes || m.language)}">${escapeHtml(m.term)}</button>`
  ).join('');
  equivList.querySelectorAll('[data-term]').forEach(chip => {
    chip.addEventListener('click', () => {
      queryInput.value = chip.dataset.term;
      runCalculation(chip.dataset.term);
      queryInput.focus();
    });
  });
}

function renderAI() {
  if (!currentText.trim() || !currentResults.length) {
    aiContent.innerHTML = '<p class="empty-state" style="padding:0">Awaiting input...</p>';
    return;
  }
  const primary = currentResults.find(r => r.system.id === primarySystemId) || currentResults[0];
  const total = primary.result.total;
  const sysName = primary.system.short;
  const breakdown = primary.result.breakdown.filter(b => b.value !== null);
  const maxItem = breakdown.reduce((a, b) => (b.value > (a?.value ?? -1) ? b : a), null);
  const letterCount = breakdown.length;
  const matches = getCorpusValues(primary.system.id).filter(e => e.total === total);
  const matchCount = matches.length;
  let crossNote = '';
  if (primary.system.language === 'hebrew') {
    const katan = currentResults.find(r => r.system.id === 'hebrew-katan');
    const gadol = currentResults.find(r => r.system.id === 'hebrew-gadol');
    if (katan && primary.system.id !== 'hebrew-katan') {
      crossNote = `Mispar Katan yields ${katan.result.total} (delta ${Math.abs(total - katan.result.total)} from ${sysName}).`;
    } else if (gadol && primary.system.id === 'hebrew-standard') {
      crossNote = `Mispar Gadol yields ${gadol.result.total} (delta ${Math.abs(total - gadol.result.total)}).`;
    }
  }
  const observations = [];
  observations.push(`Under <strong>${escapeHtml(primary.system.name)}</strong> the numerical total is <strong>${total}</strong>.`);
  if (letterCount) observations.push(`The calculation processes ${letterCount} valued character${letterCount === 1 ? '' : 's'}.`);
  if (maxItem) observations.push(`Largest single contribution: <span dir="auto">${escapeHtml(maxItem.char)}</span> = ${maxItem.value}.`);
  observations.push(matchCount === 0
    ? 'No entries in the included local corpus share this value under the selected system.'
    : `This value matches <strong>${matchCount}</strong> entr${matchCount === 1 ? 'y' : 'ies'} in the included local corpus.`);
  if (crossNote) observations.push(crossNote);
  observations.push('Numerical equivalence is a mathematical property of the chosen mapping. It does not, by itself, establish a semantic, historical, or causal relationship between terms.');
  aiContent.innerHTML = observations.map(o => `<p>${o}</p>`).join('') +
    '<p class="disclaimer">Deterministic client-side analysis · not an oracle · not a prediction engine</p>';
}

function populateReverseSystems() {
  reverseSystem.innerHTML = SYSTEMS.map(s =>
    `<option value="${s.id}">${escapeHtml(s.short)} - ${escapeHtml(s.name)}</option>`
  ).join('');
}

function runReverseLookup() {
  const val = parseInt(reverseValue.value, 10);
  const sysId = reverseSystem.value;
  if (isNaN(val) || val < 0) {
    matchList.innerHTML = '<div class="empty-state">Enter a non-negative integer</div>';
    return;
  }
  const matches = getCorpusValues(sysId).filter(e => e.total === val);
  if (!matches.length) {
    matchList.innerHTML = '<div class="empty-state">No local-corpus matches</div>';
    return;
  }
  matchList.innerHTML = matches.map(m => `
    <div class="match-item" data-term="${escapeHtml(m.term)}">
      <div class="match-term">${escapeHtml(m.term)}</div>
      <div class="match-meta">${m.total} · ${escapeHtml(m.language)}${m.notes ? ' · ' + escapeHtml(m.notes) : ''}</div>
    </div>
  `).join('');
  matchList.querySelectorAll('[data-term]').forEach(el => {
    el.addEventListener('click', () => {
      queryInput.value = el.dataset.term;
      primarySystemId = sysId;
      runCalculation(el.dataset.term);
      window.scrollTo({ top: 0, behavior: 'smooth' });
    });
  });
}

function init() {
  corpusSizeEl.textContent = getCorpusSize();
  populateReverseSystems();
  let debounce;
  queryInput.addEventListener('input', () => {
    clearTimeout(debounce);
    debounce = setTimeout(() => runCalculation(queryInput.value), 60);
  });
  reverseBtn.addEventListener('click', runReverseLookup);
  reverseValue.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') runReverseLookup();
  });
  restoreState();
  if (!queryInput.value) {
    resultsEl.innerHTML = '<div class="empty-state">Enter Latin, Hebrew, or Greek text</div>';
  }
}

init();
