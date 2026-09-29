import type { Permissions, TurnRequest } from '../types/events';
import { BackendError } from './errors';

export function permissionArgs(permissions: Permissions): string[] {
  if (permissions.access === 'read-only') {
    if (permissions.network) throw new BackendError('permission-conflict');
    return ['--read-only'];
  }
  if (permissions.access !== 'workspace-write') throw new BackendError('permission-conflict');
  return [permissions.network ? '--allow-network' : '--no-network'];
}
export function validateRequest(request: TurnRequest): void {
  const path = request.workspace.replaceAll('\\', '/').replace(/^\/\/\?\//, '');
  if (!path || path.includes('\0') || /^(\/+|[A-Za-z]:\/*)$/.test(path) ||
      !(/^(\/|[A-Za-z]:\/)/.test(path))) throw new BackendError('invalid-workspace');
  if (!request.prompt.trim() || request.prompt.includes('\0') || new TextEncoder().encode(request.prompt).length > 32 * 1024)
    throw new BackendError('invalid-prompt');
  permissionArgs(request.permissions);
}
