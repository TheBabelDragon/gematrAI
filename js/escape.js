/**
 * Shared HTML escaping and URL safety for dashboard rendering.
 * Do not interpolate untrusted values into HTML without these helpers.
 */

/** HTML text/attribute escape — five sensitive characters, single-pass. */
export function esc(value) {
  return String(value).replace(/[&<>"']/g, (char) =>
    ({
      '&': '&amp;',
      '<': '&lt;',
      '>': '&gt;',
      '"': '&quot;',
      "'": '&#39;',
    })[char]
  );
}

/**
 * Allow only http(s) URLs for href attributes.
 * Rejects javascript:, data:, and other unexpected schemes.
 */
export function safeHref(url) {
  if (!url || typeof url !== 'string') return '';
  const t = url.trim();
  try {
    const base =
      typeof window !== 'undefined' && window.location && window.location.origin
        ? window.location.origin
        : 'https://example.invalid';
    const u = new URL(t, base);
    if (u.protocol === 'https:' || u.protocol === 'http:') return u.href;
  } catch (_) {
    /* invalid */
  }
  return '';
}
