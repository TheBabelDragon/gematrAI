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

/**
 * Validate a single term record.
 * @returns {boolean}
 */
function isValidTermRecord(rec) {
  if (!rec || typeof rec !== 'object' || Array.isArray(rec)) return false;
  const surface = rec.s ?? rec.t;
  if (typeof surface !== 'string' || !surface) return false;
  const c = rec.c;
  if (c !== undefined && c !== null) {
    if (typeof c !== 'number' || !Number.isFinite(c) || c < 0 || Math.floor(c) !== c) {
      return false;
    }
  }
  if (rec.kinds !== undefined && !Array.isArray(rec.kinds)) return false;
  if (rec.kind !== undefined && typeof rec.kind !== 'string') return false;
  if (rec.r !== undefined && !Array.isArray(rec.r)) return false;
  if (rec.b !== undefined && !Array.isArray(rec.b)) return false;
  if (rec.lic !== undefined && typeof rec.lic !== 'string') return false;
  return true;
}

/**
 * Structural validation of pack or shard JSON.
 * - Root is a non-array object.
 * - Pack form: systemId -> { valueKey -> [termRecords] }
 * - Shard form: valueKey -> [termRecords]
 * Value keys must be numeric strings. Term records must have surface/term
 * and valid optional occurrence / provenance fields.
 * Rejects malformed structure with INVALID_ARTIFACT.
 * Compatible with all valid Rust-generated artifacts.
 */
export function isValidPackShape(data) {
  if (!data || typeof data !== 'object' || Array.isArray(data)) return false;
  const entries = Object.entries(data);
  if (entries.length === 0) return true; // empty pack ok
  // Detect form: if first value is an array, treat as shard; else pack
  const firstVal = entries[0][1];
  const isShard = Array.isArray(firstVal);
  for (const [key, val] of entries) {
    if (typeof key !== 'string' || !key) return false;
    if (isShard) {
      // valueKey -> [terms]
      if (!/^\d+$/.test(key)) return false;
      if (!Array.isArray(val)) return false;
      for (const rec of val) {
        if (!isValidTermRecord(rec)) return false;
      }
    } else {
      // systemId -> valueMap
      if (val === null || typeof val !== 'object' || Array.isArray(val)) return false;
      for (const [valKey, terms] of Object.entries(val)) {
        if (!/^\d+$/.test(String(valKey))) return false;
        if (!Array.isArray(terms)) return false;
        for (const rec of terms) {
          if (!isValidTermRecord(rec)) return false;
        }
      }
    }
  }
  return true;
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

  /**
   * @returns {Promise<{outcome: string, hits: Array, detail?: string}>}
   */
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
        // Pack present and valid; value simply has no entries
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

    // Shard path for values outside low/mid coverage or when packs missing
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

  /** Clear cache entry (for tests). */
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
