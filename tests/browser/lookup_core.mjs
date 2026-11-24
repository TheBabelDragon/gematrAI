/**
 * Re-export the production lookup core so Node tests exercise the same code path
 * as js/dashboard.js.
 */
export {
  LookupOutcome,
  sortTerms,
  fromPack,
  isValidPackShape,
  createLookup,
  provenanceLabels,
} from '../../js/lookup_core.js';
