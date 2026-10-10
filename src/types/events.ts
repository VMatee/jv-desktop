export type DiagnosticReason = 'malformed-json' | 'invalid-event' | 'unknown-event' | 'oversized-event' | 'invalid-utf8';
export type ErrorCategory = 'unavailable' | 'upgrade-required' | 'protocol-error' | 'invalid-workspace' | 'invalid-prompt' | 'permission-conflict'
  | 'remote-failed' | 'turn-failed' | 'authentication-required' | 'unsupported-platform' | 'process-failed' | 'timeout' | 'stream-limit' | 'internal';
export interface PublicSupport {
  message: string;
  source?: 'local' | 'remote';
  code?: string;
}
export interface DesktopError extends PublicSupport { category: ErrorCategory }
export type DesktopEvent =
  | { type: 'session-started'; sessionId: string }
  | { type: 'turn-started' }
  | { type: 'tool-started'; name: string }
  | { type: 'command-started'; id: string; command: string }
  | { type: 'command-completed'; id: string; command: string; exitCode: number | null; output: string }
  | { type: 'files-changed'; files: { path: string; kind: string }[] }
  | { type: 'assistant-message'; id: string; text: string }
  | { type: 'turn-completed' }
  | ({ type: 'remote-status'; status: 'waiting_for_auth' | 'running' } & PublicSupport)
  | ({ type: 'error' } & DesktopError)
  | { type: 'diagnostic'; reason: DiagnosticReason }
  | { type: 'run-finished'; outcome: 'completed' | 'failed' | 'cancelled' | 'timeout' };

export interface Permissions { access: 'read-only' | 'workspace-write'; network: boolean }
export interface TurnRequest { runId: string; workspace: string; prompt: string; permissions: Permissions }
export interface BackendStatus {
  availability: 'detected' | 'unavailable' | 'unsupported' | 'demo' | 'upgrade-required';
  canRun: boolean;
  detail: string;
  executionMode?: 'sandboxed' | 'trusted-host' | 'unavailable';
  minimumCliVersion?: string;
}
export interface JvBackend {
  authenticate?(request: { username: string; baseUrl: string; password: string; permissions: Permissions; structured?: boolean }): Promise<void>;
  close?(): Promise<void>;
  detectBackend(): Promise<BackendStatus>;
  chooseWorkspace(): Promise<string | null>;
  runTurn(request: TurnRequest, onEvent: (event: DesktopEvent) => void): Promise<void>;
  cancelTurn(runId: string): Promise<void>;
}
