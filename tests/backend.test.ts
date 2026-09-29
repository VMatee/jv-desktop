import { afterEach, describe, expect, it, vi } from 'vitest';
import { MockBackend } from '../src/lib/mock-backend';
import { TauriBackend } from '../src/lib/tauri-backend';
import { BackendError, safeError } from '../src/lib/errors';
import { permissionArgs, validateRequest } from '../src/lib/requests';
import type { DesktopEvent, TurnRequest } from '../src/types/events';

const request: TurnRequest = { runId: 'one', workspace: '/demo/work space;$(literal)', prompt: 'ช่วยตรวจ project นี้\n--not-an-option', permissions: { access: 'read-only', network: false } };
afterEach(() => vi.useRealTimers());
describe('requests and safe errors', () => {
  it('matches the existing permission flags and rejects conflicts', () => {
    expect(permissionArgs({ access: 'read-only', network: false })).toEqual(['--read-only']);
    expect(permissionArgs({ access: 'workspace-write', network: false })).toEqual(['--no-network']);
    expect(permissionArgs({ access: 'workspace-write', network: true })).toEqual(['--allow-network']);
    expect(() => permissionArgs({ access: 'read-only', network: true })).toThrow(BackendError);
  });
  it('validates prompt size and explicit workspace without shell parsing', () => {
    expect(() => validateRequest(request)).not.toThrow();
    for (const workspace of ['', '/', '.', '/bad\0path', 'C:\\']) expect(() => validateRequest({ ...request, workspace })).toThrow();
    for (const workspace of ['C:\\Projects\\ไทย', '\\\\?\\C:\\Projects\\demo', '\\\\server\\share\\project']) expect(() => validateRequest({ ...request, workspace })).not.toThrow();
    for (const prompt of ['', ' ', '\0', 'ก'.repeat(12000)]) expect(() => validateRequest({ ...request, prompt })).toThrow();
  });
  it('never presents arbitrary exception data', () => {
    expect(safeError(new Error('do not display this')).category).toBe('internal');
    expect(safeError('authentication-required').category).toBe('authentication-required');
  });
  it('native backend is safely unavailable in a browser', async () => {
    const native = new TauriBackend();
    expect(await native.detectBackend()).toMatchObject({ availability: 'unavailable', canRun: false });
    await expect(native.runTurn(request, () => {})).rejects.toMatchObject({ category: 'unavailable' });
  });
});
describe('offline backend', () => {
  it('emits normalized command, Thai response and completion events', async () => {
    vi.useFakeTimers(); const mock = new MockBackend(); const events: DesktopEvent[] = [];
    const run = mock.runTurn(request, (event) => events.push(event));
    await vi.runAllTimersAsync(); await run;
    expect(events.map((e) => e.type)).toEqual(['session-started', 'turn-started', 'command-started', 'command-completed', 'assistant-message', 'turn-completed', 'run-finished']);
    expect(events.find((e) => e.type === 'assistant-message')).toMatchObject({ text: expect.stringContaining('เรียบร้อยแล้ว') });
  });
  it('cancels immediately, emits no later events and permits another turn', async () => {
    vi.useFakeTimers(); const mock = new MockBackend(); const events: DesktopEvent[] = [];
    const run = mock.runTurn(request, (event) => events.push(event));
    await mock.cancelTurn(request.runId); await run;
    expect(events).toEqual([{ type: 'run-finished', outcome: 'cancelled' }]);
    await vi.runAllTimersAsync(); expect(events).toHaveLength(1);
    const again = mock.runTurn({ ...request, runId: 'two' }, (event) => events.push(event));
    await vi.runAllTimersAsync(); await again;
    expect(events.at(-1)).toEqual({ type: 'run-finished', outcome: 'completed' });
  });
});
