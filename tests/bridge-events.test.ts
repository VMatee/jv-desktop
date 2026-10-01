import { describe, it, expect } from 'vitest';
import { parseBridgeEvent } from '../src/lib/bridge-events';
describe('bridge presentation boundary', () => {
  it('maps Thai, commands, paths and tools without raw fields', () => {
    expect(parseBridgeEvent({ type: 'assistant.message', text: 'เรียบร้อยแล้ว', token: 'never' })).toMatchObject({ type: 'assistant-message', text: 'เรียบร้อยแล้ว' });
    expect(JSON.stringify(parseBridgeEvent({ type: 'assistant.message', text: 'ไทย', secret: 'never' }))).not.toContain('never');
    expect(parseBridgeEvent({ type: 'command.completed', command: 'check', exit_code: 0, output: 'ok' })).toMatchObject({ type: 'command-completed', exitCode: 0 });
    expect(parseBridgeEvent({ type: 'tool.started', name: 'search' })).toEqual({ type: 'tool-started', name: 'search' });
    expect(parseBridgeEvent({ type: 'files.changed', changes: [{path:'../ไฟล์.txt',kind:'update'}] })).toMatchObject({ type: 'files-changed' });
  });
  it('ignores unknowns and invalid fields and preserves cancellation meaning', () => {
    expect(parseBridgeEvent({ type: 'future.event', raw: 'private' })).toBeNull();
    expect(parseBridgeEvent({ type: 'assistant.message', text: 'x'.repeat(32769) })).toBeNull();
    expect(parseBridgeEvent({ type: 'turn.cancelled', local: true, remote_state: 'cancelled' })).toBeNull();
    expect(parseBridgeEvent({ type: 'turn.cancelled', local: true, remote_state: 'unknown' })).toEqual({ type: 'run-finished', outcome: 'cancelled' });
    expect(parseBridgeEvent({ type: 'turn.failed', message: 'SECRET' })).not.toHaveProperty('message', 'SECRET');
  });
});
