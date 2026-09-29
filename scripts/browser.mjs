import { createRequire } from 'node:module';
import { spawnSync } from 'node:child_process';
import { resolve } from 'node:path';
import { root, toolEnv } from './tool-env.mjs';
const require = createRequire(import.meta.url);
const result = spawnSync(process.execPath, [require.resolve('@playwright/test/cli'), ...process.argv.slice(2)], {
  cwd: root, env: { ...toolEnv(), PLAYWRIGHT_BROWSERS_PATH: resolve(root, '.cache/ms-playwright') },
  stdio: 'inherit', shell: false,
});
if (result.error) console.error('Browser checks could not start. See docs/DEVELOPMENT.md.');
process.exit(result.status ?? 1);
