import { spawnSync } from 'node:child_process';
import { root, toolEnv } from './tool-env.mjs';
const result = spawnSync('cargo', process.argv.slice(2), { cwd: root, env: toolEnv(), stdio: 'inherit', shell: false });
if (result.error) console.error('Cargo is unavailable; see docs/DEVELOPMENT.md.');
process.exit(result.status ?? 1);
