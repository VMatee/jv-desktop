export type DiagnosticReason = 'malformed-json' | 'invalid-event' | 'unknown-event' | 'oversized-event' | 'invalid-utf8';
export type ErrorCategory = 'unavailable' | 'invalid-workspace' | 'invalid-prompt' | 'permission-conflict'
  | 'authentication-required' | 'unsupported-platform' | 'process-failed' | 'timeout' | 'stream-limit' | 'internal';
export type DesktopEvent =
  | { type: 'session-started'; sessionId: string }
  | { type: 'turn-started' }
  | { type: 'command-started'; id: string; command: string }
  | { type: 'command-completed'; id: string; command: string; exitCode: number | null; output: string }
  | { type: 'files-changed'; files: { path: string; kind: string }[] }
  | { type: 'assistant-message'; id: string; text: string }
  | { type: 'turn-completed' }
  | { type: 'error'; category: ErrorCategory; message: string }
  | { type: 'diagnostic'; reason: DiagnosticReason }
  | { type: 'run-finished'; outcome: 'completed' | 'failed' | 'cancelled' | 'timeout' };

export interface Permissions { access: 'read-only' | 'workspace-write'; network: boolean }
export interface TurnRequest { runId: string; workspace: string; prompt: string; permissions: Permissions }
export interface BackendStatus {
  availability: 'detected' | 'unavailable' | 'unsupported' | 'demo';
  canRun: boolean;
  detail: string;
}
export interface JvBackend {
  detectBackend(): Promise<BackendStatus>;
  chooseWorkspace(): Promise<string | null>;
  runTurn(request: TurnRequest, onEvent: (event: DesktopEvent) => void): Promise<void>;
  cancelTurn(runId: string): Promise<void>;
}
