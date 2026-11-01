import { test, expect } from '@playwright/test';
import { readFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
const references = JSON.parse(readFileSync('.cache/native-references.json', 'utf8'));
const workerSource = readFileSync('web/worker.js', 'utf8');

async function ready(page) {
  await page.goto('./');
  await expect(page.locator('#counts')).toHaveText('2 of 4 records');
  await expect(page.locator('#status')).toContainText('Query complete');
}
async function submit(page, request) {
  await page.locator('#records').fill(request.records_json);
  await page.locator('#catalog').fill(request.catalog_json || '');
  await page.locator('#expression').fill(request.expression);
  await page.getByRole('button', { name: 'Run query', exact: true }).click();
}
async function downloaded(page, id) {
  const [download] = await Promise.all([page.waitForEvent('download'), page.locator(id).click()]);
  return readFileSync(await download.path(), 'utf8');
}

for (const ref of references) test(`native/WASM canonical parity: ${ref.name}`, async ({ page }) => {
  const errors = []; page.on('pageerror', e => errors.push(e.message));
  await ready(page); await submit(page, ref.request);
  const expected = JSON.parse(ref.canonical);
  if (expected.ok) {
    await expect(page.locator('#counts')).toHaveText(`${expected.match_count} of ${expected.input_count} records`);
    await expect(page.locator('#status')).toContainText('Query complete');
    expect((await downloaded(page, '#report-download')).trimEnd()).toBe(ref.canonical);
    expect(JSON.parse(await downloaded(page, '#download'))).toEqual(expected.matches);
    if (expected.match_count === 0) await expect(page.locator('#empty')).toBeVisible();
  } else {
    await expect(page.locator('#error')).toBeVisible();
    await expect(page.locator('#status')).toContainText('Fix the input');
    expect(JSON.parse(await page.locator('#diagnostic').textContent())).toEqual(expected.diagnostic);
    await expect(page.locator('#results')).toBeHidden();
    await expect(page.locator('#download')).toBeDisabled();
  }
  expect(errors).toEqual([]);
});

test('edits and invalid records clear stale results and exports', async ({ page }) => {
  await ready(page);
  await page.locator('#records').fill('[invalid');
  await expect(page.locator('#results')).toBeHidden();
  await expect(page.locator('#report-download')).toBeDisabled();
  await page.locator('#run').click();
  await expect(page.locator('#error')).toBeVisible();
  await page.locator('#reset').click();
  await expect(page.locator('#error')).toBeHidden();
  await page.locator('#run').click();
  await expect(page.locator('#counts')).toHaveText('2 of 4 records');
});

test('Rust byte offsets navigate Unicode selector errors correctly', async ({ page }) => {
  await ready(page);
  const expression = "title eq 'café' and missing eq 'x'";
  await page.locator('#expression').fill(expression); await page.locator('#run').click();
  await expect(page.locator('#locate-error')).toBeVisible();
  await page.locator('#locate-error').click();
  expect(await page.locator('#expression').evaluate(el => el.selectionStart)).toBe(expression.indexOf('missing'));
  await expect(page.locator('#expression')).toBeFocused();
});

test('local file input stays local and HTML-looking records remain text', async ({ page }) => {
  const uploads = []; page.on('request', r => { if (!['GET', 'HEAD'].includes(r.method())) uploads.push(r.url()); });
  await ready(page);
  const rows = [{ id: 'x', title: '<img src=x onerror="window.injected=true">' }];
  await page.locator('#file').setInputFiles({ name: 'own.json', mimeType: 'application/json', buffer: Buffer.from(JSON.stringify(rows)) });
  await expect(page.locator('#records')).toHaveValue(JSON.stringify(rows));
  await page.locator('#expression').fill("id eq 'x'"); await page.locator('#run').click();
  await expect(page.locator('#counts')).toHaveText('1 of 1 records');
  expect(JSON.parse(await downloaded(page, '#download'))).toEqual(rows);
  expect(await page.evaluate(() => Boolean(window.injected))).toBe(false);
  expect(uploads).toEqual([]);
});

test('oversized file refuses without replacing the current draft', async ({ page }) => {
  await ready(page); const original = await page.locator('#records').inputValue();
  await page.locator('#file').setInputFiles({ name: 'large.json', mimeType: 'application/json', buffer: Buffer.alloc(1048577, 32) });
  await expect(page.locator('#error-message')).toContainText('1 MiB');
  await expect(page.locator('#records')).toHaveValue(original);
  await expect(page.locator('#results')).toBeHidden();
  await expect(page.locator('#run')).toBeEnabled();
});

test('Restore example releases a pending file read and ignores its late text', async ({ page }) => {
  await page.addInitScript(() => {
    const original = File.prototype.arrayBuffer;
    File.prototype.arrayBuffer = async function () {
      await new Promise(resolve => { window.finishFileRead = resolve; });
      const value = await original.call(this); window.fileReadFinished = true; return value;
    };
  });
  await ready(page); const original = await page.locator('#records').inputValue();
  await page.locator('#file').setInputFiles({ name: 'late.json', mimeType: 'application/json', buffer: Buffer.from('[{"id":"late"}]') });
  await expect.poll(() => page.evaluate(() => typeof window.finishFileRead)).toBe('function');
  await page.locator('#reset').click();
  await expect(page.locator('#run')).toBeEnabled();
  await page.evaluate(() => window.finishFileRead());
  await expect.poll(() => page.evaluate(() => Boolean(window.fileReadFinished))).toBe(true);
  await expect(page.locator('#records')).toHaveValue(original);
});

test('WASM fetch failure preserves editable drafts and independent source routes', async ({ page }) => {
  await page.route('**/pkg/*.wasm', route => route.abort());
  await page.goto('./'); await expect(page.locator('#retry')).toBeVisible();
  await page.locator('#expression').fill("id eq 'index'");
  const zip = await page.request.get(new URL('source.zip', page.url()).href);
  expect(zip.ok()).toBe(true); expect((await zip.body()).subarray(0, 2).toString()).toBe('PK');
  const wasm = await page.request.get(new URL('pkg/slice_web_bg.wasm', page.url()).href);
  expect((await wasm.body()).subarray(0, 4).toString('hex')).toBe('0061736d');
  const provenance = await (await page.request.get(new URL('provenance.json', page.url()).href)).json();
  for (const file of ['crates/slice-web/src/lib.rs', 'crates/slice-core/src/lib.rs', 'Cargo.lock']) {
    const response = await page.request.get(new URL('source/' + file, page.url()).href);
    const expected = provenance.dirty ? readFileSync(file) : execFileSync('git', ['show', provenance.commit + ':' + file]);
    expect(await response.body()).toEqual(expected);
  }
  for (const file of ['LICENSE', 'BUILD.md', 'THIRD-PARTY-NOTICES.txt', 'rust/COPYRIGHT-library.html']) {
    expect((await page.request.get(new URL(file, page.url()).href)).ok()).toBe(true);
  }
  await page.unroute('**/pkg/*.wasm'); await page.locator('#retry').click();
  await expect(page.locator('#run')).toBeEnabled();
  await expect(page.locator('#expression')).toHaveValue("id eq 'index'");
  await page.locator('#run').click(); await expect(page.locator('#counts')).toHaveText('1 of 4 records');
});

test('startup deadline survives draft edits and Retry retains all three editors', async ({ page }) => {
  await page.clock.install(); let intercepted = false;
  await page.route('**/pkg/*.wasm', () => { intercepted = true; });
  await page.goto('./'); await expect.poll(() => intercepted).toBe(true);
  const request = { records_json: '[{"score":9}]', expression: 'score ge 8', catalog_json: '{"fields":{"score":"number"}}' };
  await page.locator('#records').fill(request.records_json); await page.locator('#expression').fill(request.expression); await page.locator('#catalog').fill(request.catalog_json);
  await page.clock.fastForward(13000);
  await expect(page.locator('#retry')).toBeVisible(); await expect(page.locator('#error-message')).toContainText('deadline');
  await page.unroute('**/pkg/*.wasm'); await page.locator('#retry').click(); await expect(page.locator('#run')).toBeEnabled();
  await expect(page.locator('#records')).toHaveValue(request.records_json); await expect(page.locator('#expression')).toHaveValue(request.expression); await expect(page.locator('#catalog')).toHaveValue(request.catalog_json);
  await page.locator('#run').click(); await expect(page.locator('#counts')).toHaveText('1 of 1 records');
});

test('late ready from a terminated worker cannot undo a startup failure', async ({ page }) => {
  const errors = []; page.on('pageerror', e => errors.push(e.message));
  await page.addInitScript(() => {
    const ActualWorker = window.Worker;
    window.Worker = class extends ActualWorker { constructor(...args) { super(...args); window.testWorker = this; } };
  });
  await page.clock.install(); let intercepted = false;
  await page.route('**/pkg/*.wasm', () => { intercepted = true; });
  await page.goto('./'); await expect.poll(() => intercepted).toBe(true);
  await page.clock.fastForward(13000); await expect(page.locator('#retry')).toBeVisible();
  // Fault injection models an already queued message after worker termination.
  await page.evaluate(() => window.testWorker.onmessage({ data: { type: 'ready' } }));
  await expect(page.locator('#run')).toBeDisabled(); await expect(page.locator('#retry')).toBeVisible();
  await expect(page.locator('#status')).toContainText('unavailable'); expect(errors).toEqual([]);
});

test('compute deadline survives edits and retries an initialized but stalled worker', async ({ page }) => {
  await page.clock.install();
  // Fault injection drops later messages; the initial result still uses real WASM.
  await page.route('**/worker.js', route => route.fulfill({ contentType: 'text/javascript', body: workerSource + '\nconst originalHandler=self.onmessage; let seen=0; self.onmessage=e=>{if(++seen===1)originalHandler(e);};' }));
  await ready(page); await page.locator('#expression').fill("id eq 'guide'"); await page.locator('#run').click();
  await expect(page.locator('#status')).toContainText('Evaluating');
  await page.locator('#expression').fill("id eq 'index'");
  await page.clock.fastForward(11000); await expect(page.locator('#retry')).toBeVisible();
  await expect(page.locator('#error-message')).toContainText('deadline'); await expect(page.locator('#download')).toBeDisabled();
  await page.unroute('**/worker.js'); await page.locator('#retry').click(); await expect(page.locator('#run')).toBeEnabled();
  await expect(page.locator('#expression')).toHaveValue("id eq 'index'");
  await page.locator('#run').click(); await expect(page.locator('#counts')).toHaveText('1 of 4 records');
});

test('late computed results cannot repopulate an edited draft', async ({ page }) => {
  await page.addInitScript(() => {
    const ActualWorker = window.Worker;
    window.Worker = class extends ActualWorker { constructor(...args) { super(...args); window.testWorker = this; } };
  });
  // Fault injection holds delivery until explicitly released; computed bytes stay real.
  await page.route('**/worker.js', route => route.fulfill({ contentType: 'text/javascript', body: workerSource + '\nconst actualPost=self.postMessage.bind(self);let posted=0,pending;self.postMessage=data=>{if(data.type==="result"&&++posted>1)pending=data;else actualPost(data);};const actualHandler=self.onmessage;self.onmessage=e=>{if(e.data.type==="test-release"){actualPost(pending);pending=null;}else actualHandler(e);};' }));
  await ready(page); await page.locator('#expression').fill("id eq 'guide'"); await page.locator('#run').click();
  await expect(page.locator('#status')).toContainText('Evaluating'); await page.locator('#expression').fill("id eq 'index'");
  await page.evaluate(() => window.testWorker.postMessage({ type: 'test-release' }));
  await expect(page.locator('#run')).toBeEnabled(); await expect(page.locator('#results')).toBeHidden(); await expect(page.locator('#download')).toBeDisabled();
  await page.locator('#run').click();
  await page.evaluate(() => window.testWorker.postMessage({ type: 'test-release' }));
  await expect(page.locator('#counts')).toHaveText('1 of 4 records');
  expect(JSON.parse(await downloaded(page, '#download'))[0].id).toBe('index');
});

for (const [name, viewport] of [['desktop', { width: 1365, height: 900 }], ['mobile', { width: 390, height: 844 }]]) {
  test(`keyboard, layout and visual sample: ${name}`, async ({ page, browser }) => {
    await page.setViewportSize(viewport); const start = Date.now(); await ready(page); const readiness = Date.now() - start;
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.locator('#expression').fill("tags has 'public'"); await page.locator('#expression').press('Enter');
    await expect(page.locator('#counts')).toHaveText('2 of 4 records');
    await page.locator('#reset').focus(); await page.keyboard.press('Enter'); await expect(page.locator('#expression')).toBeFocused();
    await page.locator('#run').focus(); await page.keyboard.press('Enter'); await expect(page.locator('#status')).toContainText('Query complete');
    mkdirSync('.cache/review', { recursive: true });
    await page.evaluate(() => scrollTo(0, 0));
    await page.screenshot({ path: `.cache/review/${name}-entry.png` });
    await page.locator('.workbench').screenshot({ path: `.cache/review/${name}-inputs.png` });
    await page.locator('#results').screenshot({ path: `.cache/review/${name}-results.png` });
    writeFileSync(`.cache/review/${name}-observation.json`, JSON.stringify({ browser: browser.version(), viewport, readiness_ms: readiness, fixture_rows: 4, scope: 'one environment-specific startup including worker/network/UI; not Rust CPU benchmark' }, null, 2));
  });
}

test('front-end UTF-8 budget refuses before posting another worker request', async ({ page }) => {
  await page.addInitScript(() => {
    const ActualWorker = window.Worker;
    window.Worker = class extends ActualWorker {
      postMessage(...args) { if (args[0].type === 'evaluate') window.evaluationPosts = (window.evaluationPosts || 0) + 1; return super.postMessage(...args); }
    };
  });
  await ready(page); const before = await page.evaluate(() => window.evaluationPosts);
  await page.locator('#expression').fill('é'.repeat(1025)); await page.locator('#run').click();
  await expect(page.locator('#error-message')).toContainText('exceeds');
  expect(await page.evaluate(() => window.evaluationPosts)).toBe(before);
  await expect(page.locator('#download')).toBeDisabled(); await expect(page.locator('#run')).toBeEnabled();
});

test('invalid UTF-8 file is refused without replacing a valid draft', async ({ page }) => {
  await ready(page); const original = await page.locator('#records').inputValue();
  await page.locator('#file').setInputFiles({ name: 'bad-encoding.json', mimeType: 'application/json', buffer: Buffer.from([91,123,34,105,100,34,58,34,255,34,125,93]) });
  await expect(page.locator('#error-message')).toContainText('valid UTF-8');
  await expect(page.locator('#records')).toHaveValue(original); await expect(page.locator('#run')).toBeEnabled();
});

test.describe('without JavaScript', () => {
  test.use({ javaScriptEnabled: false });
  test('static example, source and build routes remain available', async ({ page }) => {
    await page.goto('./'); await expect(page.getByText('This query playground requires JavaScript and WebAssembly.', { exact: false })).toBeVisible();
    await expect(page.locator('#records')).toContainText('Field guide'); await expect(page.locator('#run')).toBeDisabled();
    for (const file of ['source.zip', 'BUILD.md', 'LICENSE']) expect((await page.request.get(new URL(file, page.url()).href)).ok()).toBe(true);
  });
});
