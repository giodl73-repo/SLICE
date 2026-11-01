const $ = id => document.getElementById(id);
$('static-help').hidden = true;
const editors = ['records', 'expression', 'catalog'];
const defaults = Object.fromEntries(editors.map(id => [id, $(id).defaultValue]));
let worker, generation = 0, revision = 0, serial = 0, active = null;
let ready = false, busy = false, reading = false, startupTimer, computeTimer;
let result = null, canonical = null, expressionOffset = null, fileSerial = 0;
const STARTUP_MS = 12000, COMPUTE_MS = 10000;

function controls() {
  $('run').disabled = !ready || busy || reading;
  $('download').disabled = !result;
  $('report-download').disabled = !result;
}
function clearResults() {
  result = canonical = null;
  $('results').hidden = true;
  $('matches').textContent = $('explain').textContent = $('requirements').textContent = '';
  controls();
}
function clearError() {
  $('error').hidden = true;
  expressionOffset = null;
  $('locate-error').hidden = true;
}
function diagnostic(phase, detail) {
  clearResults();
  $('error').hidden = false;
  $('error-title').textContent = `${phase === 'expression' ? 'Selector' : phase[0].toUpperCase() + phase.slice(1)} needs attention`;
  $('error-message').textContent = detail.message || 'The request could not be evaluated.';
  $('diagnostic').textContent = JSON.stringify(detail, null, 2);
  expressionOffset = phase === 'expression' && Number.isInteger(detail.offset) ? detail.offset : null;
  $('locate-error').hidden = expressionOffset === null;
}
function engineError(message, detail = '') {
  generation++; // Fence queued events from the worker being terminated.
  ready = busy = false;
  active = null;
  clearTimeout(startupTimer); clearTimeout(computeTimer);
  worker?.terminate(); worker = null;
  diagnostic('engine', { kind: 'engine', message, ...(detail ? { detail } : {}) });
  $('status').textContent = 'Engine unavailable. Your drafts are retained; source downloads still work.';
  $('retry').hidden = false;
  controls();
}
function invalidate() {
  revision++;
  clearResults(); clearError();
  if (ready) $('status').textContent = busy ? 'Draft changed. Waiting for the previous query to finish.' : 'Draft changed. Run the query to refresh results.';
  // Loading and active-job deadlines intentionally survive edits.
}
function run() {
  if (!ready || busy || reading) return;
  clearResults(); clearError();
  for (const [id, limit] of [['records', 1048576], ['expression', 2048], ['catalog', 16384]]) {
    const value = $(id).value;
    if (value.length > limit || new TextEncoder().encode(value).length > limit) {
      diagnostic('limit', { message: `${id} exceeds its ${limit.toLocaleString()}-byte limit.` });
      $('status').textContent = 'Reduce the input and run again.';
      return;
    }
  }
  busy = true;
  active = { id: ++serial, revision };
  $('status').textContent = 'Evaluating in Rust…';
  controls();
  const id = active.id, epoch = generation;
  computeTimer = setTimeout(() => {
    if (generation === epoch && active?.id === id) engineError('The query exceeded its deadline. Retry the engine, then run your retained draft.');
  }, COMPUTE_MS);
  try {
    worker.postMessage({ type: 'evaluate', id, request: JSON.stringify({ records_json: $('records').value, expression: $('expression').value, catalog_json: $('catalog').value }) });
  } catch (error) { engineError('The worker could not accept the request. Retry with your retained inputs.', String(error)); }
}
function start() {
  generation++;
  const epoch = generation;
  worker?.terminate(); clearTimeout(startupTimer); clearTimeout(computeTimer);
  ready = busy = false; active = null;
  clearResults(); clearError();
  $('retry').hidden = true;
  $('status').textContent = 'Loading Rust engine…';
  startupTimer = setTimeout(() => { if (epoch === generation && !ready) engineError('The Rust engine did not start before its deadline. Retry to load it again.'); }, STARTUP_MS);
  try { worker = new Worker(new URL('./worker.js', import.meta.url), { type: 'module' }); }
  catch (error) { engineError('The browser could not start the Rust worker.', String(error)); return; }
  worker.onerror = event => { event.preventDefault(); if (epoch === generation) engineError('The Rust worker failed. Retry to keep working with your current inputs.', event.message); };
  worker.onmessage = ({ data }) => {
    if (epoch !== generation) return;
    if (data.type === 'startup-error') { engineError('The Rust engine could not be downloaded or initialized.', data.message); return; }
    if (data.type === 'ready') {
      clearTimeout(startupTimer); ready = true;
      $('status').textContent = 'Rust engine ready. Run your query.';
      controls(); if (revision === 0) run(); return;
    }
    if (!active || data.id !== active.id) return;
    clearTimeout(computeTimer);
    const submittedRevision = active.revision;
    busy = false; active = null;
    if (data.type === 'compute-error') { engineError('The Rust worker could not finish this query.', data.message); return; }
    if (submittedRevision !== revision) { $('status').textContent = 'Draft changed. Run the query to refresh results.'; controls(); return; }
    let response;
    try { response = JSON.parse(data.canonical); }
    catch { engineError('The Rust worker returned an unreadable response.'); return; }
    if (!response.ok) { diagnostic(response.phase, response.diagnostic); $('status').textContent = 'Fix the input and run again.'; controls(); return; }
    result = response; canonical = data.canonical;
    $('results').hidden = false;
    $('counts').textContent = `${response.match_count} of ${response.input_count} records`;
    $('row-indices').textContent = `Matching input row indices (zero-based): ${response.match_indices.join(', ') || 'none'}`;
    $('empty').hidden = response.match_count !== 0;
    $('matches').textContent = JSON.stringify(response.matches, null, 2);
    $('explain').textContent = JSON.stringify(response.explain, null, 2);
    $('requirements').textContent = response.requirements ? JSON.stringify(response.requirements, null, 2) : 'No typed catalog supplied. Add a catalog to inspect typed field requirements.';
    $('status').textContent = 'Query complete. Results reflect the current inputs.';
    controls();
  };
}
function download(name, text) {
  const url = URL.createObjectURL(new Blob([text], { type: 'application/json' }));
  const link = document.createElement('a'); link.href = url; link.download = name; link.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
$('query-form').addEventListener('submit', event => { event.preventDefault(); run(); });
for (const id of editors) $(id).addEventListener('input', invalidate);
for (const button of document.querySelectorAll('[data-query]')) button.addEventListener('click', () => { $('expression').value = button.dataset.query; invalidate(); $('expression').focus(); });
$('reset').addEventListener('click', () => { fileSerial++; reading = false; $('file').value = ''; for (const id of editors) $(id).value = defaults[id]; invalidate(); $('expression').focus(); });
$('retry').addEventListener('click', start);
$('download').addEventListener('click', () => { if (result) download('slice-matches.json', JSON.stringify(result.matches, null, 2) + '\n'); });
$('report-download').addEventListener('click', () => { if (result) download('slice-report.json', canonical + '\n'); });
$('locate-error').addEventListener('click', () => {
  if (expressionOffset === null) return;
  let bytes = 0, units = 0;
  for (const char of $('expression').value) {
    if (bytes >= expressionOffset) break;
    bytes += new TextEncoder().encode(char).length; units += char.length;
  }
  $('expression').focus(); $('expression').setSelectionRange(units, Math.min(units + 1, $('expression').value.length));
});
$('file').addEventListener('change', async () => {
  const file = $('file').files[0]; if (!file) return;
  invalidate(); const token = ++fileSerial, atRevision = revision; reading = false;
  if (file.size > 1048576) { diagnostic('file', { message: 'Choose a JSON file no larger than 1 MiB.' }); $('file').value = ''; return; }
  reading = true; controls();
  try {
    const text = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(await file.arrayBuffer());
    if (token === fileSerial && atRevision === revision) { $('records').value = text; invalidate(); }
  } catch { if (token === fileSerial) diagnostic('file', { message: 'The local file could not be read as valid UTF-8 text. Your draft is retained.' }); }
  finally { if (token === fileSerial) { reading = false; $('file').value = ''; controls(); } }
});
start();
