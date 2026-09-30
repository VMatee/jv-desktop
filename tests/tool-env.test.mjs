import { describe, expect, it } from 'vitest';
import { prependPath } from '../scripts/path-env.mjs';

describe('child toolchain PATH', () => {
  it('preserves Windows runner Path so installed Cargo remains discoverable', () => {
    const env = { Path: 'C:\\runner\\cargo\\bin;C:\\Windows\\System32', OTHER: 'preserved' };
    prependPath(env, 'D:\\checkout\\.cache\\cargo\\bin', 'win32');
    expect(env).toEqual({ Path: 'D:\\checkout\\.cache\\cargo\\bin;C:\\runner\\cargo\\bin;C:\\Windows\\System32', OTHER: 'preserved' });
  });
  it('removes duplicate Windows spellings using Node child-process precedence', () => {
    const env = { PATH: 'selected', Path: 'shadowed' };
    prependPath(env, 'local', 'win32');
    expect(env).toEqual({ PATH: 'local;selected' });
  });
  it('keeps Unix case sensitivity and colon separation', () => {
    const env = { PATH: '/usr/bin', Path: 'unrelated' };
    prependPath(env, '/local/bin', 'linux');
    expect(env).toEqual({ PATH: '/local/bin:/usr/bin', Path: 'unrelated' });
  });
});
