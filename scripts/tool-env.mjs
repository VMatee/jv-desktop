import { existsSync, mkdirSync } from 'node:fs';
import { resolve, delimiter } from 'node:path';
import { fileURLToPath } from 'node:url';

export const root = fileURLToPath(new URL('../', import.meta.url));
export function toolEnv() {
  const cache = resolve(root, '.cache');
  for (const path of ['tmp', 'cargo', 'xdg']) mkdirSync(resolve(cache, path), { recursive: true });
  const env = { ...process.env, CARGO_HOME: resolve(cache, 'cargo'),
    CARGO_TARGET_DIR: resolve(root, 'target'), TMPDIR: resolve(cache, 'tmp'),
    TMP: resolve(cache, 'tmp'), TEMP: resolve(cache, 'tmp'), XDG_CACHE_HOME: resolve(cache, 'xdg') };
  const localRust = resolve(root, '.tools/rustup');
  if (existsSync(localRust)) {
    env.RUSTUP_HOME = localRust;
    env.PATH = resolve(cache, 'cargo/bin') + delimiter + (env.PATH ?? '');
  }
  return env;
}
