import { createRequire } from 'node:module';
import { spawnSync } from 'node:child_process';
import { root, toolEnv } from './tool-env.mjs';
const require = createRequire(import.meta.url);
const result = spawnSync(process.execPath, [require.resolve('@tauri-apps/cli/tauri.js'), ...process.argv.slice(2)],
  { cwd: root, env: toolEnv(), stdio: 'inherit', shell: false });
if (result.error) console.error('Tauri could not start; see docs/DEVELOPMENT.md.');
process.exit(result.status ?? 1);
