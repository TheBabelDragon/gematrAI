/**
 * Dashboard UI — calculation authority is WASM via window.__gematra.
 * Reverse lookup uses shared js/lookup_core.js over Rust-generated artifacts only.
 */
import {
  LookupOutcome,
  createLookup,
  provenanceLabels,
} from './lookup_core.js';

const {
  calculateAllForDashboard,
  getSystem,
  SYSTEMS,
  isHebrewLetter,
  isGreekLetter,
  isLatinLetter,
} = window.__gematra;

const $ = (s) => document.querySelector(s);
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
const corpusMatchesPanel = $('#corpus-matches-panel');
const corpusMatchesSummary = $('#corpus-matches-summary');
const corpusMatchesList = $('#corpus-matches-list');
const historyEl = $('#explore-history');

let currentText = '';
let currentResults = [];
let primarySystemId = null;
let towerManifest = null;
let exploreHistory = [];
const MAX_HISTORY = 12;
const INITIAL_TERMS = 25;
const HEBREW_SYS = ['hebrew-standard', 'hebrew-gadol', 'hebrew-katan'];
const SEED_SYS = ['english-ordinal', 'english-reduction', 'english-pythagorean', 'greek-isopsephy'];
const UI_SYS_ORDER = ['hebrew-standard', 'hebrew-gadol', 'hebrew-katan', 'greek-isopsephy', 'english-ordinal', 'english-reduction', 'english-pythagorean'];

const esc = (s) => String(s).replace(/&/g, '&').replace(/</g, '<').replace(/>/g, '>').replace(/"/g, '"');
function scriptClass(ch) {
  if (isHebrewLetter(ch)) return 'hebrew';
  if (isGreekLetter(ch)) return 'greek';
  if (isLatinLetter(ch)) return 'latin';
  return '';
}
function detectScripts(text) {
  let h = 0, g = 0, l = 0;
  for (const ch of text) {
    if (isHebrewLetter(ch)) h++;
    else if (isGreekLetter(ch)) g++;
    else if (isLatinLetter(ch)) l++;
  }
  const p = [];
  if (h) p.push('Hebrew×' + h);
  if (g) p.push('Greek×' + g);
  if (l) p.push('Latin×' + l);
  return p.length ? p.join(' · ') : '—';
}
function termDir(term) {
  for (const ch of term) {
    if (isHebrewLetter(ch)) return 'rtl';
    if (isGreekLetter(ch) || isLatinLetter(ch)) return 'ltr';
  }
  return 'auto';
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
function norm(t) { return (t || '').trim(); }
function sameTerm(a, b) { return norm(a) === norm(b); }

function encodeState() {
  const p = new URLSearchParams();
  if (currentText) p.set('q', currentText);
  if (primarySystemId) p.set('sys', primarySystemId);
  if (exploreHistory.length > 1) p.set('path', exploreHistory.slice(0, -1).join('\u001f'));
  const h = p.toString();
  history.replaceState(null, '', h ? '#' + h : location.pathname);
}
function restoreState() {
  const hash = location.hash.slice(1);
  if (!hash) return;
  const p = new URLSearchParams(hash);
  const q = p.get('q'), sys = p.get('sys'), path = p.get('path');
  if (path) exploreHistory = path.split('\u001f').filter(Boolean);
  if (q) {
    queryInput.value = q;
    if (sys && getSystem(sys)) primarySystemId = sys;
    runCalculation(q, { fromHistory: true });
  }
}
function towerSet(id, v) { const el = $(id); if (el) el.textContent = v; }
function defaultManifest() {
  return {
    systems: [...HEBREW_SYS, ...SEED_SYS],
    bucketSize: 100,
    lowValuesUrl: './corpus/generated/values_low.json',
    midValuesUrl: './corpus/generated/values_mid.json',
    valuesBase: './corpus/generated/values/',
  };
}

const browserFetch = (url) => fetch(url, { cache: 'no-cache' });
const lookupSession = createLookup(browserFetch);

async function loadTowerManifest() {
  try {
    const res = await fetch('./corpus/generated/manifest.json', { cache: 'no-cache' });
    if (!res.ok) throw new Error('manifest ' + res.status);
    const m = await res.json();
    if (!m || typeof m !== 'object') throw new Error('invalid manifest');
    towerManifest = { ...defaultManifest(), ...m };
    const st = towerManifest.stats || {};
    towerSet('#tower-name', towerManifest.name || 'Corpus');
    towerSet('#tower-ver', towerManifest.tower || '0.3');
    towerSet('#tower-build', towerManifest.buildId || '—');
    towerSet('#tower-segments', st.segments != null ? String(st.segments) : '—');
    towerSet('#tower-tokens', st.unique_terms != null ? String(st.unique_terms) : '—');
    towerSet('#tower-books', st.n_books != null ? String(st.n_books) : '—');
    const scope = (towerManifest.systems || []).join(', ') || 'hebrew';
    towerSet('#tower-scope', scope + (st.seed_terms ? ' · seed ' + st.seed_terms : ''));
  } catch (e) {
    console.error('manifest load', e);
    towerSet('#tower-name', 'manifest unavailable');
    towerSet('#tower-ver', '—');
    towerSet('#tower-build', 'error');
    towerSet('#tower-segments', '—');
    towerSet('#tower-tokens', '—');
    towerSet('#tower-books', '—');
    towerSet('#tower-scope', '—');
    towerManifest = null;
  }
}

function systemInCorpus(sid) {
  if (HEBREW_SYS.includes(sid) || SEED_SYS.includes(sid)) return true;
  if (!towerManifest) return false;
  return (towerManifest.systems || []).includes(sid);
}

async function fetchShard(systemId, value) {
  if (!towerManifest) return [];
  const r = await lookupSession.lookupValue(systemId, value, towerManifest);
  return r.hits || [];
}

function pushHistory(term) {
  const t = norm(term);
  if (!t) return;
  if (exploreHistory.length && exploreHistory[exploreHistory.length - 1] === t) return;
  exploreHistory.push(t);
  if (exploreHistory.length > MAX_HISTORY) exploreHistory = exploreHistory.slice(-MAX_HISTORY);
  renderHistory();
}
function renderHistory() {
  if (!historyEl) return;
  if (exploreHistory.length <= 1) {
    historyEl.style.display = 'none';
    historyEl.innerHTML = '';
    return;
  }
  historyEl.style.display = 'flex';
  historyEl.innerHTML = exploreHistory.map((term, i) => {
    const last = i === exploreHistory.length - 1;
    const dir = termDir(term);
    return (i ? '<span class="hist-sep">→</span>' : '') +
      (last
        ? '<span class="hist-current" dir="' + dir + '">' + esc(term) + '</span>'
        : '<button type="button" class="hist-link" data-hist-idx="' + i + '" dir="' + dir + '">' + esc(term) + '</button>');
  }).join('');
  historyEl.querySelectorAll('[data-hist-idx]').forEach((btn) =>
    btn.addEventListener('click', () => {
      const idx = +btn.dataset.histIdx;
      const term = exploreHistory[idx];
      exploreHistory = exploreHistory.slice(0, idx + 1);
      queryInput.value = term;
      runCalculation(term, { fromHistory: true });
    })
  );
}

function loadTerm(term, sysId) {
  if (sysId && getSystem(sysId)) primarySystemId = sysId;
  queryInput.value = term;
  runCalculation(term);
  queryInput.focus();
  window.scrollTo({ top: 0, behavior: 'smooth' });
}

function runCalculation(text, opts = {}) {
  currentText = text;
  charCountEl.textContent = [...text].length + ' chars';
  scriptHintEl.textContent = detectScripts(text);
  if (!text.trim()) {
    resultsEl.innerHTML = '<div class="empty-state">Enter text to begin calculation</div>';
    equivPanel.style.display = 'none';
    if (corpusMatchesPanel) corpusMatchesPanel.style.display = 'none';
    aiContent.innerHTML = '<p class="empty-state" style="padding:0">Awaiting input…</p>';
    encodeState();
    return;
  }
  if (!opts.fromHistory) pushHistory(text);
  else renderHistory();
  if (!window.__gematra || !window.__gematra.engineOkFlag) {
    resultsEl.innerHTML = '<div class="empty-state">Calculation engine not ready</div>';
    encodeState();
    return;
  }
  currentResults = calculateAllForDashboard(text);
  if (!primarySystemId || !currentResults.find((r) => r.system.id === primarySystemId)) {
    primarySystemId = currentResults[0]?.system.id || null;
  }
  renderResults();
  renderEquivalenceExplorer();
  renderAI();
  encodeState();
}

function renderResults() {
  if (!currentResults.length) {
    resultsEl.innerHTML = '<div class="empty-state">No applicable systems</div>';
    return;
  }
  resultsEl.innerHTML = currentResults.map(({ system, result }) => {
    const expanded = system.id === primarySystemId ? 'expanded' : '';
    const bd = result.breakdown.map((b) =>
      '<span class="bd-item"><span class="bd-char ' + scriptClass(b.char) + '">' + esc(b.char) +
      '</span><span class="bd-val ' + (b.value === null ? 'null' : '') + '">' +
      (b.value === null ? '—' : b.value) + '</span></span>'
    ).join('');
    return '<div class="result-card ' + expanded + '" data-sys="' + system.id +
      '"><div class="result-header" data-toggle="' + system.id + '"><div class="result-sys">' +
      esc(system.name) + '</div><div class="result-total">' + result.total +
      '</div><div class="result-actions"><button class="btn-icon" data-copy="' + system.id +
      '" title="Copy total" aria-label="Copy total">C</button><button class="btn-icon" data-expand="' +
      system.id + '" aria-label="Expand">' + (expanded ? 'v' : '>') +
      '</button></div></div><div class="result-body"><div class="breakdown">' + bd +
      '</div><p style="margin-top:0.6rem;font-family:var(--mono);font-size:0.65rem;color:var(--text-dim)">' +
      esc(system.description) + '</p></div></div>';
  }).join('');
  resultsEl.querySelectorAll('[data-toggle]').forEach((el) =>
    el.addEventListener('click', (e) => {
      if (e.target.closest('[data-copy]')) return;
      primarySystemId = el.dataset.toggle;
      resultsEl.querySelectorAll('.result-card').forEach((c) => {
        const is = c.dataset.sys === primarySystemId;
        c.classList.toggle('expanded', is);
        const btn = c.querySelector('[data-expand]');
        if (btn) btn.textContent = is ? 'v' : '>';
      });
      renderEquivalenceExplorer();
      renderAI();
      encodeState();
    })
  );
  resultsEl.querySelectorAll('[data-copy]').forEach((btn) =>
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const r = currentResults.find((x) => x.system.id === btn.dataset.copy);
      if (r) {
        copyText(String(r.result.total));
        btn.textContent = 'OK';
        setTimeout(() => (btn.textContent = 'C'), 900);
      }
    })
  );
}

async function collectTerms(sid, val, inputNorm) {
  let tower = [];
  if (systemInCorpus(sid) && towerManifest) {
    const r = await lookupSession.lookupValue(sid, val, towerManifest);
    if (r.outcome === LookupOutcome.MATCHES) tower = r.hits || [];
  }
  tower = tower.filter((h) => !sameTerm(h.s || h.t, inputNorm) && !sameTerm(h.t, inputNorm));
  return tower.map((h) => {
    const labels = provenanceLabels(h);
    const source = labels.includes('seed') && labels.length === 1 ? 'seed' : 'tower';
    return { surface: h.s || h.t, c: h.c || 0, r: h.r || [], b: h.b || [], source, kinds: labels, notes: '' };
  });
}

async function renderEquivalenceExplorer() {
  if (!currentText.trim() || !currentResults.length) {
    equivPanel.style.display = 'none';
    if (corpusMatchesPanel) corpusMatchesPanel.style.display = 'none';
    return;
  }
  equivPanel.style.display = 'block';
  const inputNorm = norm(currentText);
  const systemsToShow = currentResults.map((r) => r.system.id).filter((id, i, a) => a.indexOf(id) === i)
    .sort((a, b) => (UI_SYS_ORDER.indexOf(a) < 0 ? 99 : UI_SYS_ORDER.indexOf(a)) - (UI_SYS_ORDER.indexOf(b) < 0 ? 99 : UI_SYS_ORDER.indexOf(b)));
  let html = '';
  const primary = currentResults.find((r) => r.system.id === primarySystemId) || currentResults[0];
  for (const sid of systemsToShow) {
    const row = currentResults.find((r) => r.system.id === sid);
    if (!row) continue;
    const val = row.result.total, label = row.system.name, isPrimary = sid === primarySystemId;
    const allTerms = await collectTerms(sid, val, inputNorm);
    const totalOcc = allTerms.reduce((s, t) => s + (t.c || 0), 0);
    const shown = allTerms.slice(0, INITIAL_TERMS);
    const rest = allTerms.length - shown.length;
    html += '<div class="eq-sys' + (isPrimary ? ' eq-sys-primary' : '') + '" data-eq-sys="' + sid +
      '"><div class="eq-sys-head"><span class="eq-sys-name">' + esc(label) + '</span><span class="eq-sys-val">' +
      val + '</span></div><div class="eq-sys-meta">' + allTerms.length + ' unique term' +
      (allTerms.length === 1 ? '' : 's') + (totalOcc > allTerms.length ? ' · ' + totalOcc + ' occurrences' : '') +
      ' · sorted by frequency, then lexical</div>';
    if (!allTerms.length) html += '<div class="eq-empty">No other corpus terms with this value</div>';
    else {
      html += '<div class="eq-term-grid">' + shown.map((t) => {
        const kindLabel = (t.kinds || []).length > 1 ? ' · multi-source' : t.source === 'seed' ? ' · seed' : '';
        return '<button type="button" class="eq-term" data-term="' + esc(t.surface) + '" data-sys="' + sid +
          '" dir="' + termDir(t.surface) + '"><span class="eq-term-word">' + esc(t.surface) +
          '</span><span class="eq-term-meta">' + val + (t.c > 1 ? ' · ' + t.c + ' occ' : '') + kindLabel + '</span></button>';
      }).join('') + '</div>';
      if (rest > 0) html += '<button type="button" class="eq-show-all" data-sys="' + sid + '" data-val="' + val +
        '">Show all ' + allTerms.length + ' terms</button><div class="eq-full-list" data-full-sys="' + sid + '" style="display:none"></div>';
    }
    html += '</div>';
  }
  equivList.innerHTML = html || '<span class="eq-empty">No systems applicable</span>';
  equivList.querySelectorAll('.eq-term').forEach((btn) =>
    btn.addEventListener('click', () => loadTerm(btn.dataset.term, btn.dataset.sys))
  );
  equivList.querySelectorAll('.eq-show-all').forEach((btn) =>
    btn.addEventListener('click', async () => {
      const sid = btn.dataset.sys, val = +btn.dataset.val;
      const all = await collectTerms(sid, val, norm(currentText));
      const host = equivList.querySelector('[data-full-sys="' + sid + '"]');
      if (!host) return;
      host.style.display = 'block';
      host.innerHTML = '<div class="eq-term-grid">' + all.map((t) =>
        '<button type="button" class="eq-term" data-term="' + esc(t.surface) + '" data-sys="' + sid +
        '" dir="' + termDir(t.surface) + '"><span class="eq-term-word">' + esc(t.surface) +
        '</span><span class="eq-term-meta">' + val + (t.c > 1 ? ' · ' + t.c + ' occ' : '') + '</span></button>'
      ).join('') + '</div>';
      host.querySelectorAll('.eq-term').forEach((b) =>
        b.addEventListener('click', () => loadTerm(b.dataset.term, b.dataset.sys))
      );
      btn.style.display = 'none';
    })
  );
  if (corpusMatchesPanel) {
    const sid = primary.system.id, val = primary.result.total;
    if (HEBREW_SYS.includes(sid) && towerManifest) {
      const others = await collectTerms(sid, val, inputNorm);
      corpusMatchesPanel.style.display = 'block';
      const occ = others.reduce((s, h) => s + (h.c || 0), 0);
      const works = new Set();
      others.forEach((h) => (h.b || []).forEach((b) => works.add(b)));
      corpusMatchesSummary.textContent = primary.system.short + ' · ' + val + ' · ' + others.length +
        ' unique terms · ' + occ + ' occurrences · ' + works.size + ' works';
      corpusMatchesList.innerHTML = others.length
        ? others.slice(0, 30).map((h) =>
            '<div class="match-item" data-term="' + esc(h.surface) + '"><div class="match-term" dir="rtl">' +
            esc(h.surface) + '</div><div class="match-meta">' + (h.c || 0) + ' occ · ' +
            esc((h.b || []).slice(0, 3).join(', ')) +
            ((h.kinds || []).length ? ' · ' + esc((h.kinds || []).join('+')) : '') + '</div>' +
            (h.r && h.r[0] ? '<div class="match-refs">' + esc(h.r.slice(0, 3).join(', ')) + '</div>' : '') + '</div>'
          ).join('')
        : '<div class="empty-state">No other tower terms</div>';
      corpusMatchesList.querySelectorAll('[data-term]').forEach((el) =>
        el.addEventListener('click', () => loadTerm(el.dataset.term, sid))
      );
    } else corpusMatchesPanel.style.display = 'none';
  }
}

async function renderAI() {
  if (!currentText.trim() || !currentResults.length) {
    aiContent.innerHTML = '<p class="empty-state" style="padding:0">Awaiting input…</p>';
    return;
  }
  const primary = currentResults.find((r) => r.system.id === primarySystemId) || currentResults[0];
  const total = primary.result.total, sid = primary.system.id;
  const others = await collectTerms(sid, total, norm(currentText));
  const uniqueCount = others.length;
  const occ = others.reduce((s, h) => s + (h.c || 0), 0);
  const obs = [];
  obs.push('This input has value <strong>' + total + '</strong> under <strong>' + esc(primary.system.name) + '</strong>.');
  if (uniqueCount > 0)
    obs.push('The current corpus contains <strong>' + uniqueCount + '</strong> other unique term' +
      (uniqueCount === 1 ? '' : 's') + ' with the same value' +
      (occ > uniqueCount ? ' (' + occ + ' textual occurrences)' : '') + '.');
  else obs.push('No other terms in the indexed corpus share this value under the selected system.');
  obs.push('This represents numerical equivalence in the indexed corpus; it does not by itself establish a semantic, theological, historical, or causal relationship.');
  aiContent.innerHTML = obs.map((o) => '<p>' + o + '</p>').join('') +
    '<p class="disclaimer">Deterministic client-side analysis · not an oracle</p>';
}

function populateReverseSystems() {
  reverseSystem.innerHTML = SYSTEMS.map((s) =>
    '<option value="' + s.id + '">' + esc(s.short) + ' — ' + esc(s.name) + '</option>'
  ).join('');
}

async function runReverseLookup() {
  const val = parseInt(reverseValue.value, 10);
  const sysId = reverseSystem.value;
  if (isNaN(val) || val < 0) {
    matchList.innerHTML = '<div class="empty-state">Enter a non-negative integer</div>';
    return;
  }
  matchList.innerHTML = '<div class="empty-state">Searching…</div>';
  try {
    if (!towerManifest) await loadTowerManifest();
    if (!towerManifest) {
      matchList.innerHTML = '<div class="empty-state">Corpus manifest unavailable</div>';
      return;
    }
    if (!systemInCorpus(sysId)) {
      matchList.innerHTML = '<div class="empty-state">Requested system unavailable in corpus</div>';
      return;
    }
    const result = await lookupSession.lookupValue(sysId, val, towerManifest);
    if (result.outcome === LookupOutcome.FETCH_ERROR) {
      matchList.innerHTML = '<div class="empty-state">Fetch failure — corpus artifact unavailable</div>';
      return;
    }
    if (result.outcome === LookupOutcome.INVALID_ARTIFACT) {
      matchList.innerHTML = '<div class="empty-state">Invalid corpus artifact JSON</div>';
      return;
    }
    if (result.outcome === LookupOutcome.MISSING_ARTIFACT) {
      matchList.innerHTML = '<div class="empty-state">Corpus shard missing for value ' + val + '</div>';
      return;
    }
    const towerHits = result.hits || [];
    if (result.outcome === LookupOutcome.EMPTY || !towerHits.length) {
      matchList.innerHTML = '<div class="empty-state">No indexed matches for ' + val + ' under ' + esc(sysId) + '</div>';
      return;
    }
    const occ = towerHits.reduce((s, h) => s + (h.c || 0), 0);
    const multiN = towerHits.filter((h) => provenanceLabels(h).length > 1).length;
    const seedN = towerHits.filter((h) => provenanceLabels(h).includes('seed')).length;
    let html = '<div class="sys-block-meta">' + towerHits.length + ' unique · ' + occ + ' occ' +
      (seedN ? ' · ' + seedN + ' with seed' : '') + (multiN ? ' · ' + multiN + ' multi-source' : '') + '</div>';
    html += towerHits.slice(0, 60).map((h) => {
      const surface = h.s || h.t;
      const labels = provenanceLabels(h);
      const metaParts = [(h.c || 0) + ' occ'];
      if ((h.b || []).length) metaParts.push(esc((h.b || []).slice(0, 3).join(', ')));
      if (labels.length) metaParts.push(esc(labels.join('+')));
      return '<div class="match-item" data-term="' + esc(surface) + '" data-sys="' + sysId +
        '"><div class="match-term" dir="' + termDir(surface) + '">' + esc(surface) +
        '</div><div class="match-meta">' + metaParts.join(' · ') + '</div>' +
        (h.r && h.r[0] ? '<div class="match-refs">' + esc(h.r.slice(0, 3).join(', ')) + '</div>' : '') + '</div>';
    }).join('');
    matchList.innerHTML = html;
    matchList.querySelectorAll('[data-term]').forEach((el) =>
      el.addEventListener('click', () => loadTerm(el.dataset.term, el.dataset.sys || sysId))
    );
  } catch (err) {
    matchList.innerHTML = '<div class="empty-state">Lookup failed — try again</div>';
    console.error('reverse lookup', err);
  }
}

function init() {
  populateReverseSystems();
  loadTowerManifest();
  let debounce;
  queryInput.addEventListener('input', () => {
    clearTimeout(debounce);
    debounce = setTimeout(() => {
      const t = queryInput.value;
      exploreHistory = t.trim() ? [norm(t)] : [];
      runCalculation(t, { fromHistory: true });
    }, 80);
  });
  reverseBtn.addEventListener('click', runReverseLookup);
  reverseValue.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') runReverseLookup();
  });
  document.addEventListener('keydown', (e) => {
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault();
      queryInput.focus();
      queryInput.select();
    }
  });
  restoreState();
  if (!queryInput.value)
    resultsEl.innerHTML = '<div class="empty-state">Enter Latin, Hebrew, or Greek text</div>';
}

if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', init);
else init();
