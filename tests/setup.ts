import { beforeEach, afterEach, vi } from 'vitest';
beforeEach(() => {
  vi.stubGlobal('fetch', () => { throw new Error('Network calls are forbidden in Desktop unit tests'); });
  vi.spyOn(XMLHttpRequest.prototype, 'open').mockImplementation(() => { throw new Error('Network calls are forbidden in Desktop unit tests'); });
});
afterEach(() => { vi.unstubAllGlobals(); vi.restoreAllMocks(); });
