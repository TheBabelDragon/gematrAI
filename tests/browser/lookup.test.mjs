/**
 * Node tests against the PRODUCTION lookup implementation (js/lookup_core.js).
 */
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import {
  createLookup,
  LookupOutcome,
  fromPack,
  provenanceLabels,
} from '../../js/lookup_core.js';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, '../..');
const GEN = join(ROOT, 'corpus/generated');

function loadJson(name) {
  return JSON.parse(readFileSync(join(GEN, name), 'utf8'));
}

const manifest = {
  ...loadJson('manifest.json'),
  lowValuesUrl: './corpus/generated/values_low.json',
  midValuesUrl: './corpus/generated/values_mid.json',
  valuesBase: './corpus/generated/values/',
  bucketSize: 100,
};
const low = loadJson('values_low.json');
const mid = loadJson('values_mid.json');

function makeFetch(map) {
  return async (url) => {
    if (!(url in map)) throw new Error('network');
    const entry = map[url];
    if (entry.throw) throw new Error('network');
    return {
      ok: entry.status >= 200 && entry.status < 300,
      status: entry.status,
      json: async () => {
        if (entry.invalidJson) throw new SyntaxError('bad json');
        return entry.body;
      },
    };
  };
}

let failed = 0;
async function checkAsync(name, fn) {
  try {
    await fn();
    console.log('OK:', name);
  } catch (e) {
    failed++;
    console.error('FAIL:', name, e.message);
  }
}
function check(name, fn) {
  try {
    fn();
    console.log('OK:', name);
  } catch (e) {
    failed++;
    console.error('FAIL:', name, e.message);
  }
}

const packMap = {
  './corpus/generated/values_low.json': { status: 200, body: low },
  './corpus/generated/values_mid.json': { status: 200, body: mid },
  './corpus/generated/values/greek-isopsephy/0300-0399.json': {
    status: 200,
    body: { 373: mid['greek-isopsephy']?.['373'] || [] },
  },
};

await checkAsync('exact Hebrew חי=18', async () => {
  const { lookupValue } = createLookup(makeFetch(packMap));
  const r = await lookupValue('hebrew-standard', 18, manifest);
  assert.equal(r.outcome, LookupOutcome.MATCHES);
  assert.ok(r.hits.some((h) => h.t === 'חי'));
});

await checkAsync('exact Hebrew שלום=376', async () => {
  const { lookupValue } = createLookup(makeFetch(packMap));
  const r = await lookupValue('hebrew-standard', 376, manifest);
  assert.equal(r.outcome, LookupOutcome.MATCHES);
  assert.ok(r.hits.some((h) => h.t === 'שלום'));
});

await checkAsync('exact Hebrew אהבה=13', async () => {
  const { lookupValue } = createLookup(makeFetch(packMap));
  const r = await lookupValue('hebrew-standard', 13, manifest);
  assert.equal(r.outcome, LookupOutcome.MATCHES);
  assert.ok(r.hits.some((h) => h.t === 'אהבה'));
});

await checkAsync('exact English love=54', async () => {
  const { lookupValue } = createLookup(makeFetch(packMap));
  const r = await lookupValue('english-ordinal', 54, manifest);
  assert.equal(r.outcome, LookupOutcome.MATCHES);
  assert.ok(r.hits.some((h) => h.t === 'love'));
});

await checkAsync('exact Greek λογος=373', async () => {
  const { lookupValue } = createLookup(makeFetch(packMap));
  const r = await lookupValue('greek-isopsephy', 373, manifest);
  assert.equal(r.outcome, LookupOutcome.MATCHES);
  assert.ok(r.hits.some((h) => h.t === 'λογος'));
});

await checkAsync('legitimate empty result', async () => {
  const { lookupValue } = createLookup(makeFetch(packMap));
  const r = await lookupValue('hebrew-standard', 99, manifest);
  assert.equal(r.outcome, LookupOutcome.EMPTY);
  assert.equal(r.hits.length, 0);
});

await checkAsync('missing shard', async () => {
  const map = {
    './corpus/generated/values/hebrew-standard/1000-1099.json': {
      status: 404,
      body: null,
    },
  };
  const { lookupValue } = createLookup(makeFetch(map));
  const r = await lookupValue('hebrew-standard', 1000, {
    bucketSize: 100,
    valuesBase: './corpus/generated/values/',
  });
  assert.equal(r.outcome, LookupOutcome.MISSING_ARTIFACT);
});

await checkAsync('HTTP 500', async () => {
  const map = {
    './corpus/generated/values_low.json': { status: 500, body: null },
  };
  const { lookupValue } = createLookup(makeFetch(map));
  const r = await lookupValue('hebrew-standard', 18, manifest);
  assert.equal(r.outcome, LookupOutcome.FETCH_ERROR);
});

await checkAsync('network exception', async () => {
  const map = {
    './corpus/generated/values_low.json': { throw: true },
  };
  const { lookupValue } = createLookup(makeFetch(map));
  const r = await lookupValue('hebrew-standard', 18, manifest);
  assert.equal(r.outcome, LookupOutcome.FETCH_ERROR);
});

await checkAsync('malformed JSON', async () => {
  const map = {
    './corpus/generated/values_low.json': {
      status: 200,
      body: null,
      invalidJson: true,
    },
  };
  const { lookupValue } = createLookup(makeFetch(map));
  const r = await lookupValue('hebrew-standard', 18, manifest);
  assert.equal(r.outcome, LookupOutcome.INVALID_ARTIFACT);
});

await checkAsync('invalid artifact structure (array root)', async () => {
  const map = {
    './corpus/generated/values_low.json': { status: 200, body: [1, 2, 3] },
  };
  const { lookupValue } = createLookup(makeFetch(map));
  const r = await lookupValue('hebrew-standard', 18, manifest);
  assert.equal(r.outcome, LookupOutcome.INVALID_ARTIFACT);
});

await checkAsync('manifest unavailable', async () => {
  const { lookupValue } = createLookup(makeFetch({}));
  const r = await lookupValue('hebrew-standard', 18, null);
  assert.equal(r.outcome, LookupOutcome.FETCH_ERROR);
});

await checkAsync('successful cache reuse', async () => {
  let calls = 0;
  const fetchImpl = async () => {
    calls++;
    return { ok: true, status: 200, json: async () => low };
  };
  const { lookupValue } = createLookup(fetchImpl);
  const r1 = await lookupValue('hebrew-standard', 18, manifest);
  const r2 = await lookupValue('hebrew-standard', 18, manifest);
  assert.equal(r1.outcome, LookupOutcome.MATCHES);
  assert.equal(r2.outcome, LookupOutcome.MATCHES);
  assert.equal(calls, 1);
});

await checkAsync('retry after transient failure', async () => {
  let calls = 0;
  const fetchImpl = async () => {
    calls++;
    if (calls === 1) {
      return { ok: false, status: 503, json: async () => null };
    }
    return { ok: true, status: 200, json: async () => low };
  };
  const { lookupValue } = createLookup(fetchImpl);
  const r1 = await lookupValue('hebrew-standard', 18, manifest);
  assert.equal(r1.outcome, LookupOutcome.FETCH_ERROR);
  const r2 = await lookupValue('hebrew-standard', 18, manifest);
  assert.equal(r2.outcome, LookupOutcome.MATCHES);
});

await checkAsync('multi-source provenance on λογος', async () => {
  const { lookupValue } = createLookup(makeFetch(packMap));
  const r = await lookupValue('greek-isopsephy', 373, manifest);
  assert.equal(r.outcome, LookupOutcome.MATCHES);
  const logos = r.hits.find((h) => h.t === 'λογος');
  assert.ok(logos, 'λογος present');
  const labels = provenanceLabels(logos);
  assert.ok(Array.isArray(logos.kinds) && logos.kinds.length >= 2,
    'expected kinds[] with multi-source, got ' + JSON.stringify(logos.kinds));
  assert.ok(labels.includes('fixture'), 'expected fixture in ' + JSON.stringify(labels));
  assert.ok(labels.includes('seed'), 'expected seed in ' + JSON.stringify(labels));
  assert.ok(logos.c >= 1);
});

await checkAsync('HTTP 404 on low pack falls through then empty/missing', async () => {
  const map = {
    './corpus/generated/values_low.json': { status: 404, body: null },
    './corpus/generated/values_mid.json': { status: 200, body: mid },
  };
  const { lookupValue } = createLookup(makeFetch(map));
  const r = await lookupValue('hebrew-standard', 18, {
    ...manifest,
    lowValuesUrl: './corpus/generated/values_low.json',
    midValuesUrl: './corpus/generated/values_mid.json',
  });
  assert.ok(
    r.outcome === LookupOutcome.EMPTY || r.outcome === LookupOutcome.MATCHES,
    'unexpected outcome ' + r.outcome
  );
});

await checkAsync('malformed nested system (non-object)', async () => {
  const map = {
    './corpus/generated/values_low.json': {
      status: 200,
      body: { 'hebrew-standard': null },
    },
  };
  const { lookupValue } = createLookup(makeFetch(map));
  const r = await lookupValue('hebrew-standard', 18, manifest);
  assert.equal(r.outcome, LookupOutcome.INVALID_ARTIFACT);
});

await checkAsync('malformed term array (string instead of array)', async () => {
  const map = {
    './corpus/generated/values_low.json': {
      status: 200,
      body: { 'hebrew-standard': { '18': 'not-an-array' } },
    },
  };
  const { lookupValue } = createLookup(makeFetch(map));
  const r = await lookupValue('hebrew-standard', 18, manifest);
  assert.equal(r.outcome, LookupOutcome.INVALID_ARTIFACT);
});

await checkAsync('invalid occurrence count (negative)', async () => {
  const map = {
    './corpus/generated/values_low.json': {
      status: 200,
      body: {
        'hebrew-standard': {
          '18': [{ t: 'חי', s: 'חי', c: -1 }],
        },
      },
    },
  };
  const { lookupValue } = createLookup(makeFetch(map));
  const r = await lookupValue('hebrew-standard', 18, manifest);
  assert.equal(r.outcome, LookupOutcome.INVALID_ARTIFACT);
});

await checkAsync('malformed provenance kinds (not array)', async () => {
  const map = {
    './corpus/generated/values_low.json': {
      status: 200,
      body: {
        'hebrew-standard': {
          '18': [{ t: 'חי', s: 'חי', c: 1, kinds: 'seed' }],
        },
      },
    },
  };
  const { lookupValue } = createLookup(makeFetch(map));
  const r = await lookupValue('hebrew-standard', 18, manifest);
  assert.equal(r.outcome, LookupOutcome.INVALID_ARTIFACT);
});

check('fromPack exact', () => {
  const hit = fromPack(low, 'hebrew-standard', 18);
  assert.ok(hit.some((h) => h.t === 'חי'));
});

check('imports production module path', () => {
  assert.ok(LookupOutcome.MATCHES === 'MATCHES');
});

console.log(failed ? `FAILED ${failed}` : 'ALL PRODUCTION LOOKUP TESTS PASSED');
process.exit(failed ? 1 : 0);
