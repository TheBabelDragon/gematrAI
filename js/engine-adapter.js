/**
 * Narrow adapter between the dashboard (app.js) and the Rust/WASM engine.
 * WASM is the sole authoritative calculation engine. No silent JS fallback.
 */

let wasm = null;
let initPromise = null;
let initError = null;

export function engineReady() {
  return wasm !== null && initError === null;
}

export function engineError() {
  return initError;
}

export async function initEngine(wasmBase = './wasm/') {
  if (wasm) return wasm;
  if (initPromise) return initPromise;

  initPromise = (async () => {
    try {
      const mod = await import(wasmBase + 'gematra_wasm.js');
      await mod.default(wasmBase + 'gematra_wasm_bg.wasm');
      if (typeof mod.health !== 'function' || mod.health() !== 'ok') {
        throw new Error('WASM module loaded but health() did not return ok');
      }
      wasm = mod;
      initError = null;
      return wasm;
    } catch (e) {
      initError = e;
      wasm = null;
      throw e;
    }
  })();

  return initPromise;
}

function requireWasm() {
  if (!wasm) {
    throw new Error(
      initError
        ? 'WASM engine failed to initialize: ' + initError.message
        : 'WASM engine not initialized — call initEngine() first'
    );
  }
  return wasm;
}

export function versions() {
  return JSON.parse(requireWasm().versions_json());
}

export function listSystems() {
  return JSON.parse(requireWasm().list_systems_json());
}

export function calculateOne(text, systemId, norm = 'nfc') {
  return JSON.parse(requireWasm().calculate_json(text, systemId, norm));
}

export function calculateAll(text, norm = 'nfc') {
  return JSON.parse(requireWasm().calculate_all_json(text, norm));
}

export function calculateAllForDashboard(text, norm = 'nfc') {
  const results = calculateAll(text, norm);
  const systems = listSystems();
  const byId = Object.fromEntries(systems.map(s => [s.id, s]));

  return results.map(r => {
    const meta = byId[r.system_id] || {
      id: r.system_id,
      name: r.system_name,
      short: r.system_id,
      language: '',
      description: '',
    };
    const breakdown = (r.characters || []).map(c => ({
      char: c.original,
      value: c.value === undefined ? null : c.value,
    }));
    return {
      system: meta,
      result: {
        total: r.total,
        breakdown,
        system: r.system_name,
        variant: r.system_id,
        digitalRoot: r.digital_root,
        _engine: {
          engine_version: r.engine_version,
          cipher_registry_version: r.cipher_registry_version,
          unicode_policy_version: r.unicode_policy_version,
          unsupported: r.unsupported,
          words: r.words,
        },
      },
    };
  });
}
