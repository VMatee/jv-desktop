/** Preserve Windows' case-insensitive Path key when constructing child environments.
 * Node keeps only the first spelling of duplicate keys; adding PATH beside Path
 * can otherwise discard the inherited toolchain search path.
 */
export function prependPath(env, directory, platform = process.platform) {
  const keys = Object.keys(env).filter((key) => platform === 'win32' ? key.toUpperCase() === 'PATH' : key === 'PATH').sort();
  const key = keys[0] ?? 'PATH';
  const previous = env[key] ?? '';
  for (const duplicate of keys.slice(1)) delete env[duplicate];
  env[key] = directory + (platform === 'win32' ? ';' : ':') + previous;
}
