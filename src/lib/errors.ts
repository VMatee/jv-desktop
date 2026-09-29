import type { ErrorCategory } from '../types/events';
export const errorMessages: Record<ErrorCategory, string> = {
  unavailable: 'JV CLI was not found. Install the official Linux CLI and check again.',
  'invalid-workspace': 'Choose an existing project folder, outside your home root or application folder.',
  'invalid-prompt': 'Enter a message no larger than 32 KiB.',
  'permission-conflict': 'Read only cannot be combined with tool networking.',
  'authentication-required': 'Native runs are paused until secure sign-in and prompt transport are ready. Use the offline demo for now.',
  'unsupported-platform': 'A native JV CLI backend is not supported on this platform yet. The offline demo is available.',
  'process-failed': 'JV exited unsuccessfully. No raw process diagnostics have been displayed.',
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
