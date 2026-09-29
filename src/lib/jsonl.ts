import type { DesktopEvent, DiagnosticReason } from '../types/events';

export const MAX_LINE_BYTES = 64 * 1024;
export const MAX_TEXT_LENGTH = 16 * 1024;
const encoder = new TextEncoder();
const diagnostic = (reason: DiagnosticReason): DesktopEvent => ({ type: 'diagnostic', reason });
export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
const text = (value: unknown, max = MAX_TEXT_LENGTH): string | null =>
  typeof value === 'string' ? Array.from(value).slice(0, max).join('') : null;

/** This is the only engine-JSON-to-presentation boundary. No raw object escapes. */
export function parseLine(line: string): DesktopEvent | null {
  if (encoder.encode(line).byteLength > MAX_LINE_BYTES) return diagnostic('oversized-event');
  if (!line.trim()) return null;
  let value: unknown;
  try { value = JSON.parse(line); } catch { return diagnostic('malformed-json'); }
  if (!isRecord(value) || typeof value.type !== 'string') return diagnostic('invalid-event');
  switch (value.type) {
    case 'thread.started': {
      const id = text(value.thread_id, 256);
      return id ? { type: 'session-started', sessionId: id } : diagnostic('invalid-event');
    }
    case 'turn.started': return { type: 'turn-started' };
    case 'turn.completed': return { type: 'turn-completed' };
    case 'turn.failed':
    case 'error': return { type: 'error', category: 'process-failed', message: 'JV reported a turn error. Review the activity and try again.' };
    case 'item.started':
    case 'item.completed': {
      const item = value.item;
      if (!isRecord(item)) return diagnostic('invalid-event');
      const id = text(item.id, 256);
      if (item.type === 'command_execution') {
        const command = text(item.command, 4096);
        if (!id || command === null) return diagnostic('invalid-event');
        return value.type === 'item.started' ? { type: 'command-started', id, command }
          : { type: 'command-completed', id, command,
            exitCode: Number.isSafeInteger(item.exit_code) ? item.exit_code as number : null,
            output: text(item.aggregated_output) ?? '' };
      }
      if (item.type === 'agent_message') {
        if (value.type === 'item.started') return null;
        const message = text(item.text);
        return id && message !== null ? { type: 'assistant-message', id, text: message } : diagnostic('invalid-event');
      }
      if (item.type === 'file_change' || item.type === 'file_changes') {
        if (value.type === 'item.started') return null;
        if (!Array.isArray(item.changes)) return diagnostic('invalid-event');
        const files = item.changes.slice(0, 100).flatMap((change: unknown) => {
          if (!isRecord(change)) return [];
          const path = text(change.path, 2048);
          return path ? [{ path, kind: text(change.kind, 64) ?? 'changed' }] : [];
        });
        return { type: 'files-changed', files };
      }
      return diagnostic('unknown-event');
    }
    default: return diagnostic('unknown-event');
  }
}

/** Bounded chunk framing for fixtures/future bridge transports, including CRLF. */
export class JsonlDecoder {
  private pending = '';
  private dropping = false;
  push(chunk: string): DesktopEvent[] {
    const events: DesktopEvent[] = [];
    let start = 0;
    for (let i = 0; i <= chunk.length; i++) {
      if (i !== chunk.length && chunk[i] !== '\n') continue;
      const piece = chunk.slice(start, i);
      if (!this.dropping) {
        // Check lengths before concatenating an untrusted large chunk.
        if (piece.length + this.pending.length > MAX_LINE_BYTES || encoder.encode(this.pending + piece).length > MAX_LINE_BYTES) {
          this.pending = ''; this.dropping = true; events.push(diagnostic('oversized-event'));
        } else this.pending += piece;
      }
      if (i < chunk.length) {
        if (!this.dropping) { const event = parseLine(this.pending); if (event) events.push(event); }
        this.pending = ''; this.dropping = false;
      }
      start = i + 1;
    }
    return events;
  }
  finish(): DesktopEvent[] {
    const event = this.dropping ? null : parseLine(this.pending);
    this.pending = ''; this.dropping = false;
    return event ? [event] : [];
  }
}
