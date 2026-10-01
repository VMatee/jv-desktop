import type { ErrorCategory } from '../types/events';
export const errorMessages: Record<ErrorCategory, string> = {
  'upgrade-required': 'Upgrade JV CLI to version 0.4.9 or newer.',
  'protocol-error': 'The local bridge returned invalid protocol data. Reopen the workspace to reconnect.',
  unavailable: 'JV CLI was not found. Install the official Linux CLI and check again.',
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
export class BackendError extends Error {
  constructor(public readonly category: ErrorCategory) { super(errorMessages[category]); }
}
export function safeError(error: unknown): BackendError {
  if (error instanceof BackendError) return error;
  if (typeof error === 'string' && Object.hasOwn(errorMessages, error)) return new BackendError(error as ErrorCategory);
  return new BackendError('internal');
}
