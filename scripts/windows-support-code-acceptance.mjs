// Extension of the real Tauri WebView harness. Only the compiled fake bridge
// understands these synthetic prompts; this module is called in --fixture mode.
import assert from 'node:assert/strict';
import { readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { expect } from '@playwright/test';

export async function supportCodeAcceptance(page, app, pass) {
  const workspace = resolve(dirname(app), '../workspace/success');
  const running = resolve(workspace, 'support-running');
  const complete = resolve(workspace, 'support-complete');
  const submissions = resolve(workspace, 'support-submissions.json');
  for (const file of [running, complete, submissions]) rmSync(file, { force: true });
  const notice = page.getByRole('status', { name: 'Remote service status' });
  const alert = page.getByRole('alert');
  const input = page.getByLabel('Message JV');
  const stop = page.getByRole('button', { name: /Stop/ });
  let count = 0;
  async function submit(scenario) {
    await expect(input).toBeEnabled();
    await input.fill(`FIXTURE_SUPPORT_${scenario}`);
    await page.getByRole('button', { name: /Send message/ }).click();
    count++;
    await expect.poll(() => JSON.parse(readFileSync(submissions, 'utf8'))).toBe(count);
  }
  async function selectable(container, code) {
    const element = container.locator('code.support-code');
    await expect(element).toHaveText(code);
    assert.equal(await element.evaluate(el => {
      if (getComputedStyle(el).userSelect === 'none') return false;
      const range = document.createRange(); range.selectNodeContents(el);
      const selection = window.getSelection(); selection.removeAllRanges(); selection.addRange(range);
      const selected = selection.toString(); selection.removeAllRanges();
      return selected === el.textContent;
    }), true, 'Support code must be selectable for copying');
  }
  async function active() {
    await expect(input).toBeDisabled();
    await expect(stop).toBeEnabled();
    await expect(page.getByRole('button', { name: /Send message/ })).toHaveCount(0);
    await expect(page.getByText('Session ready', { exact: true })).toBeVisible();
    await expect(alert).toHaveCount(0);
    assert.equal(JSON.parse(readFileSync(submissions, 'utf8')), count);
  }
  await pass('support A: authentication attention is nonterminal and selectable', async () => {
    await submit('RECOVER');
    await expect(notice).toContainText('Waiting for administrator attention');
    await expect(notice).toContainText('The AI service requires administrator authentication.');
    await selectable(notice, 'JV-SVC-103');
    await active();
  });
  await pass('support A: running clears attention; same submission completes', async () => {
    writeFileSync(running, 'synthetic');
    await expect(notice).toHaveCount(0);
    await active();
    writeFileSync(complete, 'synthetic');
    await expect(page.getByText('SYNTHETIC_RECOVERY_COMPLETE', { exact: true })).toBeVisible();
    await expect(input).toBeEnabled();
    await expect(page.getByText('Turn complete', { exact: true })).toBeVisible();
    assert.equal(JSON.parse(readFileSync(submissions, 'utf8')), count);
  });
  await pass('support F: subsequent local failure cannot reuse remote code', async () => {
    await submit('LOCAL');
    await expect(alert).toContainText('JV CLI could not complete this turn.');
    await expect(alert).not.toContainText('bridge exited');
    await expect(alert.locator('code')).toHaveCount(0);
    await expect(notice).toHaveCount(0);
    await expect(input).toBeEnabled();
  });
  for (const [scenario, code] of [
    ['PROVIDER', 'JV-SVC-002'], ['TIMEOUT', 'JV-TIMEOUT-001'], ['UNSUPPORTED', 'JV-AGENT-UNSUPPORTED-001'], ['LEGACY', null],
    ['MALFORMED', null], ['LONG', null], ['NO_SOURCE', null],
    ['UNREVIEWED', 'JV-SVC-002'], ['OVERSIZED', 'JV-SVC-002'],
  ]) {
    await pass(`support terminal ${scenario}: bounded allowlisted UI`, async () => {
      await submit(scenario);
      await expect(alert).toBeVisible();
      await expect(input).toBeEnabled();
      await expect(stop).toHaveCount(0);
      await expect(notice).toHaveCount(0);
      if (code) {
        await selectable(alert, code);
        await expect(alert).toContainText('JV could not complete this request.');
      } else {
        await expect(alert.locator('code')).toHaveCount(0);
        await expect(alert).toContainText('JV CLI could not complete this turn.');
      }
      await expect(page.getByText('Session ready', { exact: true })).toBeVisible();
      await expect(page.getByText('Structured session · Agent image tools enabled', { exact: true })).toBeVisible();
      assert.equal(JSON.parse(readFileSync(resolve(workspace, 'session-transport.json'), 'utf8')), 'structured');
      assert.equal(JSON.parse(readFileSync(submissions, 'utf8')), count);
      assert.doesNotMatch(await page.locator('body').innerText(), /SYNTHETIC_(?:PROVIDER_DIAGNOSTIC|UNREVIEWED_PROSE|OVERSIZED_PROSE)/);
    });
  }
  await pass('support G: Stop during authentication attention is local; no duplicate', async () => {
    await submit('CANCEL');
    await selectable(notice, 'JV-SVC-103');
    await active();
    await stop.click();
    await expect(page.getByText(/Stopped locally\. Remote job state is unknown/)).toBeVisible();
    await expect(notice).toHaveCount(0);
    await expect(input).toBeEnabled();
    assert.equal(JSON.parse(readFileSync(submissions, 'utf8')), count);
    assert.equal(await page.evaluate(() => localStorage.length + sessionStorage.length), 0);
  });
}
