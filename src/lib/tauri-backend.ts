import { Channel, invoke, isTauri } from '@tauri-apps/api/core';
import type { BackendStatus, DesktopEvent, JvBackend, TurnRequest } from '../types/events';
import { BackendError, safeError } from './errors';
import { isRecord } from './jsonl';
import { parseBridgeEvent } from './bridge-events';
import { validateRequest } from './requests';

export const desktopAvailable = isTauri();
export async function pickNativeWorkspace(): Promise<string | null> {
  try { return await invoke<string | null>('choose_workspace'); } catch (error) { throw safeError(error); }
}
/** IPC details and untrusted wire data stay outside React components. */
export class TauriBackend implements JvBackend {
  async detectBackend(): Promise<BackendStatus> {
    if (!desktopAvailable) return { availability: 'unavailable', canRun: false, detail: 'Native detection is available in the desktop app. Use the offline demo in this browser.' };
    try { return await invoke<BackendStatus>('detect_backend'); }
    catch { return { availability: 'unavailable', canRun: false, detail: 'The local backend check could not complete.' }; }
  }
  async chooseWorkspace() {
    const workspace = await pickNativeWorkspace();
    if (workspace) { try { await invoke('open_bridge'); } catch (e) { throw safeError(e); } }
    return workspace;
  }
  async authenticate(request: Parameters<NonNullable<JvBackend['authenticate']>>[0]) {
    try { await invoke('authenticate', { request }); } catch (e) { throw safeError(e); }
    finally { request.password = ''; }
  }
  async close() { if (desktopAvailable) await invoke('close_bridge'); }
  async runTurn(request: TurnRequest, onEvent: (event: DesktopEvent) => void) {
    validateRequest(request);
    if (!desktopAvailable) throw new BackendError('unavailable');
    const channel = new Channel<unknown>();
    channel.onmessage = (wire) => {
      if (!isRecord(wire) || wire.runId !== request.runId) return;
      const event = parseBridgeEvent(wire.event); if (event) onEvent(event);
    };
    try { await invoke('run_turn', { request, onEvent: channel }); }
    catch (error) { throw safeError(error); }
  }
  async cancelTurn(runId: string) {
    try { await invoke('cancel_turn', { runId }); } catch (error) { throw safeError(error); }
  }
}
