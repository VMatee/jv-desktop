import type { JvBackend } from '../types/events';
import { MockBackend } from './mock-backend';
import { desktopAvailable, pickNativeWorkspace, TauriBackend } from './tauri-backend';
export type BackendMode = 'demo' | 'native';
export function createBackends(): Record<BackendMode, JvBackend> {
  return { demo: new MockBackend(desktopAvailable ? pickNativeWorkspace : undefined), native: new TauriBackend() };
}
