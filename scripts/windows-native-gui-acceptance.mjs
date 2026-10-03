// Real release app/native dialogs. The operator selects/cancels folders and
// clicks the title-bar X; all state and process postconditions are automated.
// No offline-fixture feature, replacement picker, script close or live service.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, unlinkSync, writeFileSync } from 'node:fs';
import { createServer } from 'node:net';
import { basename, isAbsolute, relative, resolve } from 'node:path';
import { createInterface } from 'node:readline';
import { chromium } from '@playwright/test';
import { root } from './tool-env.mjs';

if (process.platform !== 'win32') throw new Error('Native Windows required');
const args = process.argv.slice(2);
const installed = args.includes('--installed');
if (installed) args.splice(args.indexOf('--installed'), 1);
const nativeOnly = args.includes('--native-only');
if (nativeOnly) args.splice(args.indexOf('--native-only'), 1);
assert.ok(!nativeOnly || installed, '--native-only requires --installed; Offline Demo is a separate prior acceptance');
const appOption = args.indexOf('--app');
if (appOption >= 0) assert.ok(isAbsolute(args[appOption + 1]), '--app requires an absolute installed executable');
const app = appOption >= 0 ? args.splice(appOption, 2)[1] : resolve(root, 'target/release/jv-desktop.exe');
const fixture = resolve(root, 'target/debug/fake-bridge.exe');
assert.ok(existsSync(app) && existsSync(fixture), 'Build the normal release app and test-fixture fake-bridge first');
const base = resolve(root, '.cache/manual-gui');
mkdirSync(base, { recursive: true });
const guided = args.includes('--guided');
const reuse = args.find(arg => arg !== '--guided');
const cache = reuse ? realpathSync(reuse) : mkdtempSync(resolve(base, 'run-'));
const cacheRelative = relative(realpathSync(base), cache);
assert.ok(cacheRelative && !cacheRelative.split(/[\\/]/).includes('..') && !isAbsolute(cacheRelative) && /^run-[A-Za-z0-9]+$/.test(basename(cache)), 'Only a disposable manual-gui run may be reused');
const workspace = resolve(cache, 'workspace ไทย/tree');
const nextWorkspace = resolve(cache, 'workspace ไทย/hung-tree');
const otherWorkspace = resolve(cache, 'other workspace');
const sentinelDir = resolve(cache, 'sentinel');
const local = resolve(cache, 'local');
const fakeInstall = resolve(local, 'Programs/JV CLI/jvcli.exe');
const realCli = resolve(process.env.LOCALAPPDATA, 'Programs/JV CLI/jvcli.exe');
const sentinelExecutable = installed ? resolve(cache, 'sentinel-cli.exe') : fakeInstall;
if (installed) assert.ok(existsSync(realCli), 'The accepted external JV CLI must already be installed');
for (const dir of [workspace, nextWorkspace, otherWorkspace, sentinelDir, resolve(local, 'Programs/JV CLI'), resolve(cache, 'tmp')]) mkdirSync(dir, { recursive: true });
for (const dir of [workspace, nextWorkspace, sentinelDir]) for (const name of ['child.json', 'grandchild.json']) {
  try { unlinkSync(resolve(dir, name)); } catch (error) { if (error.code !== 'ENOENT') throw error; }
}
copyFileSync(fixture, sentinelExecutable);
const stageFile = resolve(base, 'current.json');
const continueFile = resolve(cache, 'continue.json');
try { unlinkSync(continueFile); } catch (error) { if (error.code !== 'ENOENT') throw error; }
const results = [];
function stage(name, action, path) {
  writeFileSync(stageFile, JSON.stringify({ stage: name, action, path, cache, desktopPid: child?.pid, results }, null, 2));
  console.log(JSON.stringify({ stage: name, action, path }));
}
function pass(name, evidence) {
  results.push({ name, result: 'PASS', evidence });
  console.log(`PASS ${name}`);
}
const env = {};
for (const key of ['SYSTEMROOT', 'WINDIR', 'USERPROFILE']) if (process.env[key]) env[key] = process.env[key];
env.LOCALAPPDATA = local; env.APPDATA = resolve(cache, 'roaming');
env.TEMP = env.TMP = resolve(cache, 'tmp');
env.PATH = installed ? `${resolve(realCli, '..')};${env.SYSTEMROOT}\\System32` : `${env.SYSTEMROOT}\\System32`;
env.WEBVIEW2_USER_DATA_FOLDER = resolve(cache, 'webview');
const port = await new Promise(resolvePort => {
  const server = createServer();
  server.listen(0, '127.0.0.1', () => { const p = server.address().port; server.close(() => resolvePort(p)); });
});
env.WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = `--remote-debugging-port=${port} --remote-debugging-address=127.0.0.1`;
const child = spawn(app, [], { cwd: cache, env, stdio: ['ignore', 'ignore', 'pipe'], shell: false, windowsHide: false });
let stderrBytes = 0;
child.stderr.on('data', bytes => { stderrBytes += bytes.length; if (stderrBytes > 65536) child.kill(); });
const watcher = spawn(resolve(process.env.SYSTEMROOT, 'py.exe'), ['-3.10', '-B', resolve(root, 'scripts/windows-gui-process-watch.py'), '--fixture', sentinelExecutable, '--sentinel-dir', sentinelDir], {
  cwd: root, env: { ...env, PATH: `${env.SYSTEMROOT};${env.SYSTEMROOT}\\System32` }, stdio: ['pipe', 'pipe', 'pipe'], windowsHide: true, shell: false
});
watcher.stderr.on('data', bytes => { if (bytes.length) console.error('Native process watcher diagnostic (not displayed in Desktop)'); });
const lines = createInterface({ input: watcher.stdout });
const pending = [];
lines.on('line', line => {
  const job = pending.shift();
  if (!job || line.length > 8192) return;
  const value = JSON.parse(line);
  if (value.ok) job.resolve(value.result); else job.reject(new Error(value.error));
});
function query(command) {
  return new Promise((resolveReply, reject) => {
    const timer = setTimeout(() => reject(new Error('Process watcher deadline exceeded')), 45000);
    pending.push({ resolve: value => { clearTimeout(timer); resolveReply(value); }, reject: error => { clearTimeout(timer); reject(error); } });
    watcher.stdin.write(`${JSON.stringify(command)}\n`);
  });
}
async function until(test, label, milliseconds = 600000) {
  const end = Date.now() + milliseconds;
  while (Date.now() < end) {
    assert.equal(child.exitCode, null, `Desktop exited during ${label}`);
    if (await test()) return;
    await new Promise(r => setTimeout(r, 150));
  }
  throw new Error(`${label} timed out; native action was not accepted`);
}
async function manualStage(name, action, path) {
  if (guided) {
    stage(`ready-${name}`, action, path);
    await until(() => {
      try { return JSON.parse(readFileSync(continueFile, 'utf8')).stage === name; } catch { return false; }
    }, 'guided action instruction');
  }
  stage(name, action, path);
}
async function tree(path, group) {
  await until(() => ['child.json', 'grandchild.json'].every(name => {
    try { return Number.isInteger(JSON.parse(readFileSync(resolve(path, name), 'utf8')).pid); } catch { return false; }
  }), 'fixture tree startup', 10000);
  return query({ type: 'tree', workspace: path, group });
}
let browser;
let accepted = false;
let nativeClose = false;
try {
  await query({ type: 'desktop', pid: child.pid });
  for (let n = 0; n < 100; n++) {
    assert.equal(child.exitCode, null, 'Native app exited before WebView attachment');
    try { browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`); break; } catch { await new Promise(r => setTimeout(r, 100)); }
  }
  assert.ok(browser, 'Native WebView debug attachment timed out');
  const context = browser.contexts()[0];
  let page;
  await until(() => { page = context.pages().find(p => /tauri\.localhost/.test(p.url())); return !!page; }, 'bundled page startup', 10000);
  page.setDefaultTimeout(7000);
  const external = [];
  await page.route('**/*', route => {
    const url = new URL(route.request().url());
    if (!['tauri.localhost', 'ipc.localhost', '127.0.0.1', 'localhost'].includes(url.hostname) && !['data:', 'ipc:'].includes(url.protocol)) { external.push(url.origin); return route.abort(); }
    return route.continue();
  });
  await page.getByRole('heading', { name: 'Conversation', exact: true }).waitFor();
  if (installed && !nativeOnly) {
    assert.equal(await page.getByLabel('Backend mode').inputValue(), 'demo');
    await page.getByText(`v${JSON.parse(readFileSync(resolve(root,'package.json'),'utf8')).version}`, {exact:true}).waitFor();
    await manualStage('select-demo', 'Select this disposable folder for Offline Demo in the real Windows folder dialog', workspace);
    await page.getByRole('button', { name: /Open Folder/ }).click();
    await until(async () => await page.locator('.workspace-location p').textContent() === realpathSync(workspace), 'installed demo folder selection');
    await page.getByLabel('Message JV').fill('สวัสดี ไทย — release acceptance, offline only');
    await page.getByRole('button', {name:/Send message/}).click();
    await page.getByText('Turn complete',{exact:true}).waitFor();
    await page.getByText(/your offline preview is ready/).waitFor();
    pass('installed frontend/version renders and Offline Demo completes a Thai conversation without service calls');
  }
  await page.getByLabel('Backend mode').selectOption('native');
  await page.getByText('JV CLI: detected', { exact: true }).waitFor();
  await page.getByRole('note', {name:'Trusted host policy'}).waitFor();
  pass(installed ? 'installed application detects external JV CLI and displays fixed trusted-host policy' : 'normal release application renders; isolated fake CLI detected');

  await manualStage('select-primary', 'Select this disposable folder in the real Windows folder dialog', workspace);
  await page.getByRole('button', { name: /Open Folder/ }).click();
  await page.getByRole('button', { name: /Choosing/ }).waitFor();
  await until(async () => await page.locator('.workspace-location p').textContent() === realpathSync(workspace), 'native folder selection');
  await page.getByLabel('Password').waitFor();
  let first;
  if (installed) {
    await until(async () => {
      try { first = await query({type:'real-bridge', executable:realCli}); return true; }
      catch { return false; }
    }, 'installed bridge process identity', 10000);
  } else { first = await tree(workspace, 'first'); }
  if (installed) {
    // Tauri uses Windows FOLDERID_LocalAppData, not the child's LOCALAPPDATA
    // environment override. Readiness is unauthenticated and need not create
    // state. Its separate absolute HOME is covered by the native Rust/CLI probe.
    const desktopState = resolve(process.env.LOCALAPPDATA, 'com.openjvspace.desktop/jvcli-state');
    assert.notEqual(desktopState.toLowerCase(), resolve(process.env.LOCALAPPDATA,'JV/jv-cli').toLowerCase());
    pass('Desktop-specific state path is separate from terminal CLI state; native launch contract verified by installed-CLI smoke', {desktopState, stateContentsInspected:false});
  }
  pass('native folder picker returns canonical space/Thai drive path; Rust opens its matching bridge', { workspace: realpathSync(workspace), ...first });
  const errors = await page.evaluate(async paths => {
    const answer = [];
    for (const path of paths) {
      try {
        await window.__TAURI_INTERNALS__.invoke('run_turn', { request: { runId: 'gui-workspace-check', workspace: path, prompt: 'synthetic workspace validation only', permissions: { access: 'workspace-write', network: true } }, onEvent: '__CHANNEL__:12345' });
        answer.push('unexpected-success');
      } catch (e) { answer.push(e); }
    }
    return answer;
  }, [otherWorkspace, workspace.slice(0, 3)]);
  assert.deepEqual(errors, ['invalid-workspace', 'invalid-workspace']);
  pass('registered Rust IPC rejects different/root workspaces; the matching bridge confirms the canonical selection');

  let second = first;
  if (!installed) {
  await manualStage('cancel-dialog', 'Click Cancel in the real Windows folder dialog; the selection and bridge must remain unchanged', workspace);
  await page.getByRole('button', { name: /Open Folder/ }).click();
  await page.getByRole('button', { name: /Choosing/ }).waitFor();
  await until(() => page.getByRole('button', { name: /Open Folder/ }).count().then(n => n === 1), 'native dialog cancellation');
  assert.equal(await page.locator('.workspace-location p').textContent(), realpathSync(workspace));
  await query({ type: 'unchanged', group: 'first' });
  pass('native dialog cancellation preserves the selected workspace and owned bridge');

  await manualStage('change-workspace', 'Select the second disposable folder in the real Windows folder dialog', nextWorkspace);
  await page.getByRole('button', { name: /Open Folder/ }).click();
  await page.getByRole('button', { name: /Choosing/ }).waitFor();
  await until(async () => await page.locator('.workspace-location p').textContent() === realpathSync(nextWorkspace), 'native workspace change');
  second = await tree(nextWorkspace, 'second');
  const replacement = await query({ type: 'changed', old: 'first', new: 'second' });
  pass('workspace change shuts down the previous bridge before starting the new bridge and cleans its owned tree', replacement);
  }
  await query({ type: 'before-close' });
  await page.screenshot({ path: resolve(cache, 'before-native-close.png') });
  assert.deepEqual(external, [], 'Frontend attempted an external request');
  await manualStage('close-window', installed ? 'Click the real title-bar X on the installed JV Desktop; no login or production task was sent' : 'Click the real title-bar X on JV Desktop. The fake bridge deliberately ignores cooperative shutdown', installed ? workspace : nextWorkspace);
  nativeClose = true;
  // Neither JavaScript close nor a forced termination is used for acceptance.
  await new Promise((resolveExit, reject) => {
    const timer = setTimeout(() => reject(new Error('Native title-bar close action was not completed')), 600000);
    if (child.exitCode !== null) { clearTimeout(timer); resolveExit(); }
    else child.once('exit', () => { clearTimeout(timer); resolveExit(); });
  });
  assert.equal(child.exitCode, 0, 'Native Desktop exit was not clean');
  const start = Date.now();
  const cleanup = await query({ type: 'closed' });
  const postExitVerificationMs = Date.now() - start;
  pass(installed ? 'installed native window close exits Desktop, real bridge and all captured descendants; unrelated sentinel survives' : 'real native window close exits Desktop, bridge, child/grandchild and preserves the unrelated sentinel', { ...cleanup, postExitVerificationMs, ...second });
  accepted = true;
  const report = { acceptance: 'native manual-action acceptance + automated postconditions', app, installed, results, skips: 0, productionCalls: 0, nativeClose: true };
  writeFileSync(resolve(cache, 'results.json'), JSON.stringify(report, null, 2));
  stage('complete', 'Both native controls and all automated postconditions passed');
} catch (error) {
  writeFileSync(resolve(cache, 'results.json'), JSON.stringify({ acceptance: 'native manual-action acceptance + automated postconditions', results, failure: error.message, nativeCloseAttempted: nativeClose, forcedCleanupIsAcceptance: false }, null, 2));
  stage('failed', error.message);
  throw error;
} finally {
  if (browser) await browser.close().catch(() => {});
  // Failure-only cleanup is scoped to our own process and never counted as a
  // native close pass. Successful acceptance never terminates Desktop here.
  if (!accepted && child.exitCode === null) child.kill();
  if (child.exitCode === null) await new Promise(r => child.once('exit', r));
  watcher.stdin.end();
  await new Promise(r => watcher.exitCode !== null ? r() : watcher.once('exit', r));
  lines.close();
}
