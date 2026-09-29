import { test, expect } from '@playwright/test';

test('offline workspace, Thai conversation, stop, native blocker and responsive layout', async ({ page }, testInfo) => {
  const external: string[] = [];
  await page.route('**/*', async (route) => {
    const url = new URL(route.request().url());
    if (url.origin !== 'http://127.0.0.1:1420') { external.push(url.origin); await route.abort(); }
    else await route.continue();
  });
  await page.goto('/');
  await expect(page.getByRole('button', { name: /Send message/ })).toBeDisabled();
  await page.screenshot({ path: testInfo.outputPath('empty.png'), fullPage: true });
  await page.getByRole('button', { name: /Open Folder/ }).click();
  await expect(page.getByText('/demo/workspace')).toBeVisible();
  await page.getByLabel('Message JV').fill('ช่วยตรวจ project นี้');
  await page.getByLabel('Message JV').press('Enter');
  await expect(page.getByText(/your offline preview is ready/)).toBeVisible();
  await expect(page.getByText('Turn complete', { exact: true })).toBeVisible();
  await expect(page.getByLabel('Message JV')).toBeFocused();
  await page.screenshot({ path: testInfo.outputPath('conversation.png'), fullPage: true });
  await page.getByLabel('Message JV').fill('Cancel this simulated turn');
  await page.getByRole('button', { name: /Send message/ }).click();
  await page.getByRole('button', { name: /Stop/ }).click();
  await expect(page.getByText('Turn stopped. No further demo events will arrive.')).toBeVisible();
  await page.setViewportSize({ width: 800, height: 700 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
  await page.screenshot({ path: testInfo.outputPath('compact.png'), fullPage: true });
  await page.getByLabel('Backend mode').selectOption('native');
  await expect(page.getByText('JV CLI: unavailable')).toBeVisible();
  await expect(page.getByRole('button', { name: /Send message/ })).toBeDisabled();
  expect(external).toEqual([]);
});
