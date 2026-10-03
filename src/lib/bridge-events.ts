import type { DesktopEvent } from '../types/events';
import { isRecord } from './jsonl';
import { errorMessages, remoteError, remoteSupport } from './errors';
/** Rust forwards allowlisted bridge fields; validate again at the presentation boundary. */
export function parseBridgeEvent(value: unknown): DesktopEvent | null {
  if (!isRecord(value) || typeof value.type !== 'string') return null;
  const text = (key: string, max = 32768) => typeof value[key] === 'string' && new TextEncoder().encode(value[key]).length <= max ? value[key] as string : null;
  switch (value.type) {
    case 'turn.started': return { type: 'turn-started' };
    case 'assistant.message': { const t = text('text'); return t === null ? null : { type: 'assistant-message', id: crypto.randomUUID(), text: t }; }
    case 'tool.started': { const name = text('name', 256); return name === null ? null : { type: 'tool-started', name }; }
    case 'command.started': { const command = text('command', 8192); return command === null ? null : { type: 'command-started', id: crypto.randomUUID(), command }; }
    case 'command.completed': { const command = text('command', 8192), output = text('output', 16384); return command === null || output === null || !(value.exit_code === null || Number.isInteger(value.exit_code)) ? null : { type: 'command-completed', id: crypto.randomUUID(), command, output, exitCode: value.exit_code as number | null }; }
    case 'files.changed': return Array.isArray(value.changes) && value.changes.length <= 50 && value.changes.every((c) => isRecord(c) && typeof c.path === 'string' && typeof c.kind === 'string') ? { type: 'files-changed', files: value.changes as { path: string; kind: string }[] } : null;
    case 'turn.status': {
      if (value.source !== 'remote' || (value.status !== 'waiting_for_auth' && value.status !== 'running')) return null;
      const message = value.status === 'waiting_for_auth' ? 'Waiting for administrator attention.' : 'Remote job resumed.';
      const support = value.status === 'waiting_for_auth' ? remoteSupport(value.support_code, value.public_message, message) : null;
      return { type: 'remote-status', status: value.status, source: 'remote', message, ...support };
    }
    case 'turn.completed': return { type: 'run-finished', outcome: 'completed' };
    case 'turn.failed': {
      const remote = value.source === 'remote' ? remoteError(value.support_code, value.public_message) : null;
      return remote ? { type: 'error', ...remote } : { type: 'error', category: 'process-failed', source: 'local', message: errorMessages['process-failed'] };
    }
    case 'turn.cancelled': return value.local === true && value.remote_state === 'unknown' ? { type: 'run-finished', outcome: 'cancelled' } : null;
    default: return null;
  }
}
