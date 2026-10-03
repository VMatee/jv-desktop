// Opt-in native Tauri WebView acceptance. No browser UI authentication or real
// account: the fixture build uses only the Rust-selected sibling fake bridge.
import { spawn } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { createServer } from 'node:net';
import assert from 'node:assert/strict';
import { chromium } from '@playwright/test';
import { root } from './tool-env.mjs';
if (process.platform !== 'win32') throw new Error('Native Windows required');
const app = process.argv[2];
const fixture = process.argv.includes('--fixture');
const packageSmoke = process.argv.includes('--package-smoke');
if (!app || !resolve(app).toLowerCase().endsWith('.exe')) throw new Error('usage: node scripts/windows-webview-acceptance.mjs ABSOLUTE_APP.exe [--fixture]');
const cache = resolve(root, '.cache/gui', fixture ? 'fixture-run' : 'real-run');
mkdirSync(cache, { recursive: true });
const port = await new Promise((resolvePort) => {
  const server = createServer(); server.listen(0, '127.0.0.1', () => { const p = server.address().port; server.close(() => resolvePort(p)); });
});
const env = {};
// Preserve native runtime/profile locations; a LOCALAPPDATA override does not
// relocate Tauri's known folder and can hide a per-user WebView2 installation.
// Browser data still goes only to the disposable acceptance directory.
for (const k of ['SYSTEMROOT', 'WINDIR', 'SYSTEMDRIVE', 'PROGRAMFILES', 'PROGRAMFILES(X86)', 'PROGRAMW6432', 'USERPROFILE', 'LOCALAPPDATA', 'APPDATA', 'TEMP', 'TMP']) if (process.env[k]) env[k] = process.env[k];
env.PATH = packageSmoke ? `${process.env.SYSTEMROOT}\\System32` : `${process.env.LOCALAPPDATA}\\Programs\\JV CLI;${process.env.SYSTEMROOT}\\System32`;
env.WEBVIEW2_USER_DATA_FOLDER = resolve(cache, 'webview');
env.WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = `--remote-debugging-port=${port} --remote-debugging-address=127.0.0.1`;
const child = spawn(resolve(app), [], { cwd: cache, env, windowsHide: true, stdio: ['ignore', 'ignore', 'pipe'], shell: false });
let stderrBytes = 0;
child.stderr.on('data', (bytes) => { stderrBytes += bytes.length; if (stderrBytes > 65536) child.kill(); });
const exited = new Promise((resolveExit) => child.once('exit', resolveExit));
const results = [];
let browser;
let phase = 'WebView attachment';
async function pass(name, action) { phase = name; await action(); results.push({ name, result: 'PASS' }); console.log(`PASS ${name}`); }
try {
  // A cold hosted Windows runner can take longer than a warm developer session.
  // Bound the whole attachment phase and every individual connection attempt.
  const attachmentDeadline = Date.now() + 45000;
  while (Date.now() < attachmentDeadline) {
    if (child.exitCode !== null) throw new Error('Native application exited before WebView attachment');
    try { browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`, { timeout: Math.min(1000, Math.max(1, attachmentDeadline - Date.now())) }); break; } catch { await new Promise((r) => setTimeout(r, 100)); }
  }
  if (!browser) throw new Error('Native WebView debug attachment timed out');
  const context = browser.contexts()[0];
  phase = 'Tauri page startup';
  let page;
  for (let n = 0; n < 50; n++) { page = context.pages().find(p => /tauri\.localhost/.test(p.url())); if (page) break; await new Promise(r => setTimeout(r,100)); }
  if (!page) throw new Error('Tauri page unavailable');
  page.setDefaultTimeout(7000);
  const external = [];
  await page.route('**/*', route => {
    const url = new URL(route.request().url());
    if (!['tauri.localhost', 'ipc.localhost', '127.0.0.1', 'localhost'].includes(url.hostname) && !['data:', 'ipc:'].includes(url.protocol)) { external.push(url.origin); return route.abort(); }
    return route.continue();
  });
  await pass('native window renders bundled UI', async () => { await page.getByRole('heading', {name: 'Conversation', exact: true}).waitFor(); assert.equal(await page.title(), 'JV Desktop'); });
  if (packageSmoke) {
    await pass('installed default Offline Demo and Native JV CLI choices render', async () => {
      assert.equal(await page.getByLabel('Backend mode').inputValue(), 'demo');
      await page.getByLabel('Backend mode').selectOption('native');
      await page.getByRole('note', {name:'Trusted host policy'}).waitFor();
      assert.equal(await page.getByLabel('Workspace permission').count(),0);
      assert.equal(await page.getByLabel('Tool network').count(),0);
    });
    await page.evaluate(()=>window.__TAURI_INTERNALS__.invoke('close_bridge'));
    assert.deepEqual(external, []);
    writeFileSync(resolve(cache, 'package-smoke.json'), JSON.stringify({results, skips:0, productionCalls:0}, null,2));
  } else {
  if (fixture) {
    await pass('offline demo folder and Thai conversation', async () => {
      await page.getByRole('button', { name: /Open Folder/ }).click();
      await page.getByLabel('Message JV').fill('สวัสดี ไทย\n$(literal)');
      await page.getByRole('button', { name: /Send message/ }).click(); await page.getByText('Turn complete', { exact: true }).waitFor();
      await page.getByText(/your offline preview is ready/).waitFor();
    });
    await pass('offline demo Stop', async () => {
      await page.getByLabel('Message JV').fill('Stop this offline fixture'); await page.getByRole('button',{ name:/Send message/ }).click();
      await page.getByRole('button',{ name:/Stop/ }).click(); await page.getByText(/Turn stopped/).waitFor();
    });
  }
  await pass('native backend detects and exposes trusted-host capabilities', async () => {
    await page.getByLabel('Backend mode').selectOption('native'); await page.getByText('JV CLI: detected', { exact: true }).waitFor();
    await page.getByRole('note', {name:'Trusted host policy'}).waitFor();
    assert.equal(await page.getByLabel('Workspace permission').count(),0); assert.equal(await page.getByLabel('Tool network').count(),0);
    await page.getByText(/Full filesystem access.*Network enabled.*No tool approval prompts/).waitFor();
  });
  if (fixture) {
    await pass('Rust-selected fixture folder opens protocol bridge', async () => {
      await page.getByRole('button', {name:/Open Folder/}).click(); await page.getByLabel('Password').waitFor();
    });
    await pass('synthetic stdin sign-in and full-access session', async () => {
      await page.getByLabel('Username', {exact:true}).fill('synthetic'); await page.getByLabel('Service URL').fill('http://127.0.0.1:9'); await page.getByLabel('Password').fill('TEST_PASSWORD_DO_NOT_LEAK');
      await page.getByRole('button',{name:'Sign in',exact:true}).click(); await page.getByText('Session ready',{exact:true}).waitFor();
      assert.equal(await page.getByLabel('Password').count(),0); assert.equal(await page.evaluate(()=>localStorage.length+sessionStorage.length),0);
    });
    await pass('Thai turn command file and assistant events', async () => {
      await page.getByLabel('Message JV').fill('ช่วยตรวจไฟล์ ไทย\n$(not a shell command)'); await page.getByRole('button',{name:/Send message/}).click();
      await page.getByText('เรียบร้อยแล้ว',{exact:true}).waitFor(); await page.getByText('Turn complete',{exact:true}).waitFor(); await page.getByText('Command completed',{exact:true}).waitFor();
    });
    await pass('second turn cancellation is local and conservative', async () => {
      await page.getByLabel('Message JV').fill('FIXTURE_SLOW_TURN'); await page.getByRole('button',{name:/Send message/}).click();
      await page.getByRole('button',{name:/Stop/}).click(); await page.getByText(/Remote job state is unknown/).waitFor();
    });
    await pass('reopening folder creates fresh unauthenticated bridge', async () => {
      await page.getByRole('button',{name:/Open Folder/}).click(); await page.getByLabel('Password').waitFor();
    });
  }
  await page.screenshot({path:resolve(cache,'window.png')});
  await pass('cooperative IPC bridge shutdown', async () => { await page.evaluate(()=>window.__TAURI_INTERNALS__.invoke('close_bridge')); });
  assert.deepEqual(external,[],'Frontend attempted external network');
  writeFileSync(resolve(cache,'results.json'), JSON.stringify({ fixture, results, nativeDialog:'Not driven: native computer-use helper unavailable', windowClose:'Not accepted: script close did not exit Desktop; process terminated after cooperative bridge close', productionCalls:0 },null,2));
  }
} catch (error) {
  // Static test phases/exit codes only: never publish child stderr, UI values,
  // environment values or an authentication transcript as CI annotations.
  if (process.env.GITHUB_ACTIONS === 'true') {
    console.error(`::error title=Installed WebView acceptance::Phase: ${phase}; error: ${error.name}; application exit: ${child.exitCode}; stderr bytes: ${stderrBytes}`);
  }
  throw error;
} finally {
  if (browser) await browser.close();
  if (child.exitCode === null && child.signalCode === null) child.kill();
  await exited;
}
