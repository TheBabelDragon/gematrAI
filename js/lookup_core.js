/**
 * Shared corpus reverse-lookup core.
 * Used by production dashboard (js/dashboard.js) and Node tests.
 * Numerical values come only from Rust-generated index artifacts — never computed here.
 */

export const LookupOutcome = Object.freeze({
  MATCHES: 'MATCHES',
  EMPTY: 'EMPTY',
  MISSING_ARTIFACT: 'MISSING_ARTIFACT',
  FETCH_ERROR: 'FETCH_ERROR',
  INVALID_ARTIFACT: 'INVALID_ARTIFACT',
});

/** Sort hits by occurrence desc, then lexical. */
export function sortTerms(hits) {
  return [...hits].sort((a, b) => {
    const ca = a.c || 0;
    const cb = b.c || 0;
    if (cb !== ca) return cb - ca;
    return (a.s || a.t || '').localeCompare(b.s || b.t || '', 'he');
  });
}

/**
 * Extract sorted hits for a system/value from a pack object.
 * @returns {Array|null} null if system absent or empty list
 */
export function fromPack(pack, systemId, value) {
  if (!pack || typeof pack !== 'object' || Array.isArray(pack)) return null;
  if (!pack[systemId]) return null;
  const hit = pack[systemId][String(value)] || pack[systemId][value];
  if (!hit || !Array.isArray(hit) || !hit.length) return null;
  return sortTerms(hit);
}

/** Validate pack/shard JSON structure. */
export function isValidPackShape(data) {
  return data && typeof data === 'object' && !Array.isArray(data);
}

/**
 * Create a lookup session bound to a fetch implementation and optional cache.
 * @param {(url: string) => Promise<{ok: boolean, status: number, json: () => Promise<any>}>} fetchImpl
 * @param {Map} [cache]
 */
export function createLookup(fetchImpl, cache = new Map()) {
  async function loadPackResult(url, cacheKey) {
    if (!url) return { status: 'missing', data: null };
    if (cache.has(cacheKey)) {
      const cached = cache.get(cacheKey);
      if (cached && cached.status === 'ok') return cached;
    }
    try {
      const res = await fetchImpl(url);
      if (res.status === 404) {
        return { status: 'missing', data: null, httpStatus: 404 };
      }
      if (!res.ok) {
        return { status: 'fetch_error', data: null, httpStatus: res.status };
      }
      let data;
      try {
        data = typeof res.json === 'function' ? await res.json() : res.body;
      } catch (_) {
        return { status: 'invalid', data: null, httpStatus: res.status };
      }
      if (!isValidPackShape(data)) {
        return { status: 'invalid', data: null, httpStatus: res.status };
      }
      const out = { status: 'ok', data, httpStatus: res.status };
      cache.set(cacheKey, out);
      return out;
    } catch (_) {
      return { status: 'fetch_error', data: null };
    }
  }

  async function lookupValue(systemId, value, manifest) {
    if (!manifest || typeof manifest !== 'object') {
      return {
        outcome: LookupOutcome.FETCH_ERROR,
        hits: [],
        detail: 'manifest unavailable',
      };
    }

    const tryPack = async (url, key) => {
      if (!url) return null;
      const r = await loadPackResult(url, key);
      if (r.status === 'fetch_error') {
        return {
          outcome: LookupOutcome.FETCH_ERROR,
          hits: [],
          detail: 'pack fetch failed',
        };
      }
      if (r.status === 'invalid') {
        return {
          outcome: LookupOutcome.INVALID_ARTIFACT,
          hits: [],
          detail: 'malformed pack JSON',
        };
      }
      if (r.status === 'missing') return null;
      if (r.status === 'ok') {
        const hit = fromPack(r.data, systemId, value);
        if (hit && hit.length) {
          return { outcome: LookupOutcome.MATCHES, hits: hit };
        }
        return { outcome: LookupOutcome.EMPTY, hits: [], _packOk: true };
      }
      return null;
    };

    if (value <= 99 && manifest.lowValuesUrl) {
      const r = await tryPack(manifest.lowValuesUrl, '__low__');
      if (r && r.outcome === LookupOutcome.MATCHES) return r;
      if (
        r &&
        (r.outcome === LookupOutcome.FETCH_ERROR ||
          r.outcome === LookupOutcome.INVALID_ARTIFACT)
      ) {
        return r;
      }
      if (r && r._packOk) {
        return { outcome: LookupOutcome.EMPTY, hits: [] };
      }
    }

    if (value <= 999 && manifest.midValuesUrl) {
      const r = await tryPack(manifest.midValuesUrl, '__mid__');
      if (r && r.outcome === LookupOutcome.MATCHES) return r;
      if (
        r &&
        (r.outcome === LookupOutcome.FETCH_ERROR ||
          r.outcome === LookupOutcome.INVALID_ARTIFACT)
      ) {
        return r;
      }
      if (r && r._packOk) {
        return { outcome: LookupOutcome.EMPTY, hits: [] };
      }
    }

    const size = manifest.bucketSize || 100;
    const lo = Math.floor(value / size) * size;
    const b =
      String(lo).padStart(4, '0') +
      '-' +
      String(lo + size - 1).padStart(4, '0');
    const key = systemId + '|' + b;
    const url =
      (manifest.valuesBase || './corpus/generated/values/') +
      systemId +
      '/' +
      b +
      '.json';
    const r = await loadPackResult(url, key);
    if (r.status === 'fetch_error') {
      return {
        outcome: LookupOutcome.FETCH_ERROR,
        hits: [],
        detail: 'shard fetch failed',
      };
    }
    if (r.status === 'invalid') {
      return {
        outcome: LookupOutcome.INVALID_ARTIFACT,
        hits: [],
        detail: 'malformed shard JSON',
      };
    }
    if (r.status === 'missing') {
      return {
        outcome: LookupOutcome.MISSING_ARTIFACT,
        hits: [],
        detail: 'shard not found',
      };
    }
    const raw = r.data[String(value)] || r.data[value] || [];
    const hit = sortTerms(Array.isArray(raw) ? raw : []);
    if (hit.length) return { outcome: LookupOutcome.MATCHES, hits: hit };
    return { outcome: LookupOutcome.EMPTY, hits: [] };
  }

  function clearCache() {
    cache.clear();
  }

  return { loadPackResult, lookupValue, cache, clearCache };
}

/**
 * Format provenance labels from kind + kinds array.
 * @returns {string[]} unique source classification labels
 */
export function provenanceLabels(item) {
  const labels = new Set();
  const kinds = Array.isArray(item.kinds) ? item.kinds : [];
  for (const k of kinds) {
    if (k) labels.add(String(k));
  }
  if (item.kind) labels.add(String(item.kind));
  return [...labels];
}
