import { describe, expect, it } from 'vitest';
import { JsonlDecoder, MAX_LINE_BYTES, parseLine } from '../src/lib/jsonl';

const parse = (value: unknown) => parseLine(JSON.stringify(value));
describe('engine event normalization', () => {
  it('maps sessions, turns, command state and completed responses', () => {
    expect(parse({ type: 'thread.started', thread_id: 't1', arbitrary: 'not exposed' })).toEqual({ type: 'session-started', sessionId: 't1' });
    expect(parse({ type: 'turn.started' })).toEqual({ type: 'turn-started' });
    expect(parse({ type: 'turn.completed' })).toEqual({ type: 'turn-completed' });
    const item = { id: 'one', type: 'command_execution', command: 'pytest', exit_code: 7, aggregated_output: 'test failed' };
    expect(parse({ type: 'item.started', item })).toEqual({ type: 'command-started', id: 'one', command: 'pytest' });
    expect(parse({ type: 'item.completed', item })).toEqual({ type: 'command-completed', id: 'one', command: 'pytest', exitCode: 7, output: 'test failed' });
    expect(parse({ type: 'item.completed', item: { ...item, exit_code: '0' } })).toMatchObject({ exitCode: null });
  });
  it.each(['file_change', 'file_changes'])('maps %s, copying only bounded known fields', (type) => {
    expect(parse({ type: 'item.completed', item: { type, changes: [{ path: 'ไทย.txt', kind: 'update', arbitrary: true }, null] } })).toEqual({ type: 'files-changed', files: [{ path: 'ไทย.txt', kind: 'update' }] });
  });
  it('preserves Thai, Unicode and HTML as inert text', () => {
    const text = 'เรียบร้อยแล้ว 🟢\n<script>alert(1)</script>';
    expect(parse({ type: 'item.completed', item: { id: 'a', type: 'agent_message', text } })).toEqual({ type: 'assistant-message', id: 'a', text });
    expect(parse({ type: 'item.started', item: { id: 'a', type: 'agent_message', text } })).toBeNull();
  });
  it.each(['turn.failed', 'error'])('does not expose raw error content for %s', (type) => {
    const event = parse({ type, message: 'untrusted diagnostic', error: { message: 'untrusted diagnostic' } });
    expect(event).toMatchObject({ type: 'error', category: 'process-failed' });
    expect(JSON.stringify(event)).not.toContain('untrusted diagnostic');
  });
  it('rejects invalid data without throwing or evaluating it', () => {
    for (const line of ['{', '<script>bad()</script>', '{"type":']) expect(parseLine(line)).toEqual({ type: 'diagnostic', reason: 'malformed-json' });
    for (const value of [null, [], 1, {}, { type: 'thread.started' }, { type: 'item.completed', item: [] }, { type: 'item.completed', item: { type: 'agent_message', id: 'a', text: 42 } }]) expect(parse(value)).toEqual({ type: 'diagnostic', reason: 'invalid-event' });
    expect(parse({ type: 'new.event', payload: '<anything>' })).toEqual({ type: 'diagnostic', reason: 'unknown-event' });
    expect(parse({ type: 'item.completed', item: { type: 'future_item' } })).toEqual({ type: 'diagnostic', reason: 'unknown-event' });
    expect(parseLine(' \r')).toBeNull();
  });
  it('limits UTF-8 bytes and field lengths, rather than splitting Unicode', () => {
    expect(parseLine('ก'.repeat(MAX_LINE_BYTES / 2))).toEqual({ type: 'diagnostic', reason: 'oversized-event' });
    const event = parse({ type: 'item.completed', item: { id: 'x', type: 'agent_message', text: 'a'.repeat(18000) + '🟢' } });
    expect(event?.type === 'assistant-message' && event.text.length).toBe(16384);
  });
});

describe('bounded JSONL framing', () => {
  it('handles split records, CRLF, blank lines and final records without newline', () => {
    const decoder = new JsonlDecoder();
    expect(decoder.push('{"type":"turn.')).toEqual([]);
    expect(decoder.push('started"}\r\n\n{"type":"turn.completed"}')).toEqual([{ type: 'turn-started' }]);
    expect(decoder.finish()).toEqual([{ type: 'turn-completed' }]);
    expect(decoder.finish()).toEqual([]);
  });
  it('discards an oversized fragmented line and recovers on the next newline', () => {
    const decoder = new JsonlDecoder();
    expect(decoder.push('x'.repeat(MAX_LINE_BYTES))).toEqual([]);
    expect(decoder.push('x')).toEqual([{ type: 'diagnostic', reason: 'oversized-event' }]);
    expect(decoder.push('discard\n{"type":"turn.started"}\n')).toEqual([{ type: 'turn-started' }]);
    expect(decoder.finish()).toEqual([]);
  });
});
