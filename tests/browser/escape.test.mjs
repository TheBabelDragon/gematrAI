/**
 * Security regression: HTML escaping and URL safety.
 * Exercises the same helpers used by js/dashboard.js rendering paths.
 */
import assert from 'node:assert/strict';
import { esc, safeHref } from '../../js/escape.js';

const MALICIOUS = [
  '<script>alert(1)</script>',
  '<img src=x onerror=alert(1)>',
  '"> <svg onload=alert(1)>'.replace(' ', ''),
  "' autofocus onfocus=alert(1) '",
  '& < > " \'',
  'javascript:alert(1)',
  '<a href="javascript:alert(1)">x</a>',
  '"> <img src=x onerror=alert(1)>'.replace(' ', ''),
];

let failed = 0;
function check(name, fn) {
  try {
    fn();
    console.log('OK:', name);
  } catch (e) {
    failed++;
    console.error('FAIL:', name, e.message);
  }
}

check('esc encodes ampersand', () => {
  assert.equal(esc('a&b'), 'a&amp;b');
});

check('esc encodes angle brackets', () => {
  assert.equal(esc('<script>'), '&lt;script&gt;');
});

check('esc encodes double and single quotes', () => {
  assert.equal(esc('"quoted"'), '&quot;quoted&quot;');
  assert.equal(esc("it's"), 'it&#39;s');
});

check('esc is single-pass (no double-encode)', () => {
  assert.equal(esc('&amp;'), '&amp;amp;');
});

check('malicious script tag is inert text', () => {
  const out = esc('<script>alert(1)</script>');
  assert.ok(!out.includes('<script'));
  assert.ok(out.includes('&lt;script&gt;'));
});

check('malicious img onerror is inert text', () => {
  const out = esc('<img src=x onerror=alert(1)>');
  assert.ok(!out.includes('<img'));
  assert.ok(out.includes('&lt;img'));
});

check('attribute-breakout payload is escaped', () => {
  const out = esc('"> <svg onload=alert(1)>'.replace(' ', ''));
  assert.ok(out.includes('&quot;'));
  assert.ok(out.includes('&lt;svg'));
});

check('event-handler autofocus payload is escaped', () => {
  const out = esc("' autofocus onfocus=alert(1) '");
  assert.ok(out.includes('&#39;'));
  assert.ok(!out.startsWith("'"));
});

check('all five special chars together', () => {
  assert.equal(esc('&<>"\''), '&amp;&lt;&gt;&quot;&#39;');
});

check('Hebrew and Greek terms preserved', () => {
  assert.equal(esc('חי'), 'חי');
  assert.equal(esc('שלום'), 'שלום');
  assert.equal(esc('λογος'), 'λογος');
  assert.equal(esc('λόγος'), 'λόγος');
});

check('data-term attribute simulation stays inert', () => {
  const term = '"> <img src=x onerror=alert(1)>'.replace(' ', '');
  const html = '<button data-term="' + esc(term) + '">' + esc(term) + '</button>';
  assert.ok(!html.includes('<img'));
  assert.ok(html.includes('&lt;img'));
});

check('history / query term rendering stays inert', () => {
  for (const payload of MALICIOUS) {
    const html = '<span class="hist">' + esc(payload) + '</span>';
    assert.ok(!/<script\b/i.test(html), payload);
    assert.ok(!/<img\b/i.test(html), payload);
    assert.ok(!/<svg\b/i.test(html), payload);
    const attr = 'data-x="' + esc(payload) + '"';
    if (payload.includes('"')) {
      assert.ok(attr.includes('&quot;'), 'quote must be entity in attr for ' + payload);
    }
  }
});

check('provenance and refs rendering stay inert', () => {
  const refs = ['Genesis 1:1', '<script>x</script>', '"> <svg onload=1>'.replace(' ', '')];
  const html = refs.map((r) => esc(r)).join(', ');
  assert.ok(!html.includes('<script'));
  assert.ok(!html.includes('<svg'));
});

check('safeHref allows https', () => {
  const h = safeHref('https://www.sefaria.org/Genesis.1.1');
  assert.ok(h.startsWith('https://'));
});

check('safeHref allows http', () => {
  const h = safeHref('http://example.com/path');
  assert.ok(h.startsWith('http://'));
});

check('safeHref rejects javascript:', () => {
  assert.equal(safeHref('javascript:alert(1)'), '');
});

check('safeHref rejects data:', () => {
  assert.equal(safeHref('data:text/html,<script>alert(1)</script>'), '');
});

check('safeHref rejects empty and non-string', () => {
  assert.equal(safeHref(''), '');
  assert.equal(safeHref(null), '');
  assert.equal(safeHref(undefined), '');
});

check('match-item template with malicious surface stays inert', () => {
  const surface = '"> <svg onload=alert(1)>'.replace(' ', '');
  const labels = ['fixture', 'seed'];
  const html =
    '<div class="match-item" data-term="' +
    esc(surface) +
    '"><div class="match-term">' +
    esc(surface) +
    '</div><div class="match-meta">' +
    esc(labels.join('+')) +
    '</div></div>';
  assert.ok(!html.includes('<svg'));
  assert.ok(html.includes('&lt;svg'));
});

console.log(failed ? `FAILED ${failed}` : 'ALL ESCAPE / URL SAFETY TESTS PASSED');
process.exit(failed ? 1 : 0);
