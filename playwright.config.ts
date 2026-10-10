import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: './tests/e2e', fullyParallel: false, workers: 1,
  use: { baseURL: 'http://127.0.0.1:1420', viewport: { width: 1180, height: 820 }, trace: 'off' },
  // Exercise the same production assets bundled into Desktop. Cold dev-server
  // transforms on Windows can consume the entire scenario deadline before the
  // first control renders; HMR and per-module transforms are not shipped.
  webServer: { command: 'npm run build && npm run preview', url: 'http://127.0.0.1:1420', reuseExistingServer: false, timeout: 30_000 },
});
