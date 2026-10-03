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

describe('public remote failures', () => {
  it.each(['JV-SVC-103', 'JV-SVC-002', 'JV-TIMEOUT-001', 'JV-FUTURE-999', 'JV-AGENT-PROVIDER-CLEANUP-001'])('preserves %s without provider mapping', (code) => {
    expect(parseBridgeEvent({ type: 'turn.failed', source: 'remote', support_code: code,
      public_message: 'The AI service requires administrator authentication.', message: 'SECRET' }))
      .toEqual({ type: 'error', category: 'remote-failed', source: 'remote', code,
        message: 'The AI service requires administrator authentication.' });
  });
  it.each(['JV-SVC-103\u2028', 'JV-SVC-103\u2029', 'JV-SVC-103\n', 'JV-SVC-103\u0000', 'JV-secret-001', 'https://secret', 'JV-' + 'A'.repeat(30) + '-001', null])('rejects malformed code %s', (code) => {
    const event = parseBridgeEvent({ type: 'turn.failed', source: 'remote', support_code: code, public_message: 'SECRET' });
    expect(event).toMatchObject({ category: 'process-failed', source: 'local' });
    expect(event).not.toHaveProperty('code');
    expect(JSON.stringify(event)).not.toContain('SECRET');
  });
  it.each(['<html>selector SECRET</html>', 'x'.repeat(257), 'token=SECRET', 'The AI service requires administrator authentication.\n', undefined])('drops untrusted prose', (message) => {
    expect(parseBridgeEvent({ type: 'turn.failed', source: 'remote', support_code: 'JV-SVC-002', public_message: message }))
      .toMatchObject({ category: 'remote-failed', code: 'JV-SVC-002', message: 'JV could not complete this request.' });
  });
  it('keeps legacy and local failures generic', () => {
    for (const fields of [{ code: 'engine_error', message: 'SECRET' }, { source: 'local', support_code: 'JV-SVC-103' }]) {
      expect(parseBridgeEvent({ type: 'turn.failed', ...fields })).toMatchObject({ category: 'process-failed', source: 'local' });
    }
  });
});

describe('nonterminal remote status', () => {
  it('keeps authentication waiting separate from a failure and clears details on running', () => {
    expect(parseBridgeEvent({ type:'turn.status', source:'remote', status:'waiting_for_auth', support_code:'JV-SVC-103',
      public_message:'The AI service requires administrator authentication.' })).toEqual({
      type:'remote-status', source:'remote', status:'waiting_for_auth', code:'JV-SVC-103',
      message:'The AI service requires administrator authentication.' });
    expect(parseBridgeEvent({ type:'turn.status', source:'remote', status:'running', support_code:'JV-SVC-103', public_message:'SECRET' }))
      .toEqual({ type:'remote-status', source:'remote', status:'running', message:'Remote job resumed.' });
  });
  it('bounds public fields without inventing terminal failure', () => {
    const event = parseBridgeEvent({ type:'turn.status', source:'remote', status:'waiting_for_auth',
      support_code:'JV-SVC-103\n', public_message:'x'.repeat(257) });
    expect(event).toEqual({ type:'remote-status', source:'remote', status:'waiting_for_auth', message:'Waiting for administrator attention.' });
    expect(parseBridgeEvent({ type:'turn.status', source:'local', status:'waiting_for_auth' })).toBeNull();
    expect(parseBridgeEvent({ type:'turn.status', source:'remote', status:'failed' })).toBeNull();
  });
});
