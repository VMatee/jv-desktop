import type { DesktopError, ErrorCategory, PublicSupport } from '../types/events';
export const errorMessages: Record<ErrorCategory, string> = {
  'remote-failed': 'JV could not complete this request.',
  'upgrade-required': 'Upgrade JV CLI to the version required for your platform, then check again.',
  'protocol-error': 'The local bridge returned invalid protocol data. Reopen the workspace to reconnect.',
  unavailable: 'JV CLI was not found. Install the official CLI for your platform and check again.',
  'invalid-workspace': 'Choose an existing project folder, outside your home root or application folder.',
  'invalid-prompt': 'Enter a message no larger than 32 KiB.',
  'permission-conflict': 'Read only cannot be combined with tool networking.',
  'authentication-required': 'Sign-in is required or was unsuccessful. Reopen the workspace and sign in again.',
  'unsupported-platform': 'A native JV CLI backend is not supported on this platform yet. The offline demo is available.',
  'process-failed': 'JV could not complete the operation or the bridge exited. Reopen the folder and sign in again.',
  timeout: 'The local process timed out and was stopped.',
  'stream-limit': 'The process output exceeded the safety limit and was stopped.',
  internal: 'The operation could not be completed. Please try again.',
};
// Reviewed public phrases only. This is deliberately not a code-to-message map.
const publicMessages = new Set([
  'The AI service requires administrator authentication.',
  'The service restarted after submission. The request was not sent again to prevent a duplicate.',
  'Provider job failed.',
  'Provider is temporarily unavailable.',
]);
export function remoteSupport(code: unknown, message: unknown, fallback: string): PublicSupport | null {
  if (typeof code !== 'string' || code.length > 32 || !/^JV-(?:[A-Z]{1,16}-){1,3}[0-9]{3}$/.test(code) || /[^A-Z0-9-]/.test(code)) return null;
  return { source: 'remote', code,
    message: typeof message === 'string' && new TextEncoder().encode(message).length <= 256 && publicMessages.has(message)
      ? message : fallback };
}
export function remoteError(code: unknown, message: unknown): DesktopError | null {
  const support = remoteSupport(code, message, errorMessages['remote-failed']);
  return support ? { ...support, category: 'remote-failed' } : null;
}
export class BackendError extends Error implements DesktopError {
  readonly source = 'local' as const;
  constructor(public readonly category: ErrorCategory) { super(errorMessages[category]); }
}
export function safeError(error: unknown): BackendError {
  if (error instanceof BackendError) return error;
  if (typeof error === 'string' && Object.hasOwn(errorMessages, error)) return new BackendError(error as ErrorCategory);
  return new BackendError('internal');
}
