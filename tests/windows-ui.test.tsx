import { afterEach, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import App from '../src/App';
import { MockBackend } from '../src/lib/mock-backend';
import type { DesktopEvent, JvBackend } from '../src/types/events';
afterEach(cleanup);
function native(status: 'detected' | 'upgrade-required' | 'unavailable' = 'detected'): JvBackend {
  return {
    detectBackend: async () => ({ availability: status, canRun: status === 'detected', executionMode: 'trusted-host', minimumCliVersion: '0.5.0', detail: 'Local fixture' }),
    chooseWorkspace: async () => 'C:\\Projects\\ไทย', authenticate: vi.fn(async () => {}),
    runTurn: vi.fn(async (_request, emit) => { emit({ type: 'run-finished', outcome: 'completed' }); }), cancelTurn: vi.fn(), close: vi.fn(async () => {}),
  };
}
async function open(backend: JvBackend) {
  render(<App backends={{ demo: new MockBackend(async () => '/demo', 0), native: backend }} />);
  fireEvent.change(screen.getByLabelText('Backend mode'), { target: { value: 'native' } });
  await screen.findByText(/JV CLI:/);
  fireEvent.click(screen.getByRole('button', { name: /Open Folder/ }));
  await screen.findByText('C:\\Projects\\ไทย');
}
async function login() {
  fireEvent.change(await screen.findByLabelText('Username'), { target: { value: 'synthetic' } });
  fireEvent.change(screen.getByLabelText('Service URL'), { target: { value: 'http://127.0.0.1:9' } });
  fireEvent.change(screen.getByLabelText('Password'), { target: { value: 'TEST_PASSWORD_DO_NOT_LEAK' } });
  fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));
  await waitFor(() => expect(screen.queryByLabelText('Password')).toBeNull());
}
it('shows Windows trusted-host policy and removes conflicting permission controls', async () => {
  const backend = native(); await open(backend);
  expect(await screen.findByRole('note', { name: 'Trusted host policy' })).toBeTruthy();
  expect(screen.getByText(/Full filesystem access.*Network enabled.*No tool approval prompts/)).toBeTruthy();
  expect(screen.getByText(/For dedicated worker PCs/)).toBeTruthy();
  expect(screen.queryByLabelText('Workspace permission')).toBeNull(); expect(screen.queryByLabelText('Tool network')).toBeNull();
  await login();
  expect(backend.authenticate).toHaveBeenCalledWith(expect.objectContaining({ permissions: { access: 'workspace-write', network: true } }));
  fireEvent.change(screen.getByLabelText('Message JV'), { target: { value: 'ไทย' } });
  fireEvent.click(screen.getByRole('button', { name: /Send message/ }));
  await screen.findByText('Turn complete');
  expect(backend.runTurn).toHaveBeenCalledWith(expect.objectContaining({ permissions: { access: 'workspace-write', network: true } }), expect.any(Function));
  expect(localStorage.length + sessionStorage.length).toBe(0);
});
it('keeps Offline Demo permission controls independent of Windows native policy', async () => {
  await open(native());
  fireEvent.change(screen.getByLabelText('Backend mode'), { target: { value: 'demo' } });
  expect(screen.queryByRole('note', { name: 'Trusted host policy' })).toBeNull();
  expect((screen.getByLabelText('Workspace permission') as HTMLSelectElement).value).toBe('read-only');
  expect((screen.getByLabelText('Tool network') as HTMLInputElement).checked).toBe(false);
});
it('does not submit when Windows requires a CLI upgrade', async () => {
  const backend = native('upgrade-required'); await open(backend);
  expect(await screen.findByText('JV CLI: upgrade-required')).toBeTruthy();
  expect(screen.getByText(/Requires JV CLI 0.5.0\+ for Windows/)).toBeTruthy();
  expect(screen.queryByLabelText('Password')).toBeNull();
  fireEvent.change(screen.getByLabelText('Message JV'), { target: { value: 'test' } });
  expect((screen.getByRole('button', { name: /Send message/ }) as HTMLButtonElement).disabled).toBe(true);
  expect(backend.runTurn).not.toHaveBeenCalled();
});
it('directs Windows users to the official CLI release when it is missing', async () => {
  const backend = native('unavailable'); await open(backend);
  expect(await screen.findByText('JV CLI: unavailable')).toBeTruthy();
  expect(screen.getByText(/Requires JV CLI 0.5.0\+ for Windows.*github.com\/VMatee\/jv-cli\/releases/)).toBeTruthy();
  expect(screen.queryByLabelText('Password')).toBeNull();
  expect((screen.getByRole('button', { name: /Send message/ }) as HTMLButtonElement).disabled).toBe(true);
  expect(backend.runTurn).not.toHaveBeenCalled();
});
it('keeps Linux on its released 0.4.9 floor and editable sandbox permissions', async () => {
  const backend = native('unavailable');
  backend.detectBackend = async () => ({ availability: 'unavailable', canRun: false, executionMode: 'sandboxed', minimumCliVersion: '0.4.9', detail: 'Linux fixture' });
  await open(backend);
  expect(screen.getByText(/Requires JV CLI 0.4.9\+ for Linux/)).toBeTruthy();
  expect(screen.queryByText(/Requires JV CLI 0.5.0/)).toBeNull();
  expect(screen.getByLabelText('Workspace permission')).toBeTruthy();
  expect(screen.getByLabelText('Tool network')).toBeTruthy();
  expect(screen.queryByRole('note', { name: 'Trusted host policy' })).toBeNull();
});
it('sanitizes bridge failures and clears password references', async () => {
  const backend = native(); backend.authenticate = vi.fn(async (request) => { request.password = ''; throw new Error('PRIVATE_RAW_STDERR'); }); await open(backend);
  fireEvent.change(await screen.findByLabelText('Username'), { target: { value: 'synthetic' } });
  const password = screen.getByLabelText('Password') as HTMLInputElement;
  fireEvent.change(password, { target: { value: 'TEST_PASSWORD_DO_NOT_LEAK' } }); fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));
  await waitFor(() => expect(password.value).toBe(''));
  expect(screen.queryByText(/PRIVATE_RAW_STDERR/)).toBeNull(); expect(localStorage.length + sessionStorage.length).toBe(0);
});
it('Stop and stale event suppression preserve conservative cancellation', async () => {
  const backend = native(); let emit: ((e: DesktopEvent) => void) | undefined; let finish: (() => void) | undefined;
  backend.runTurn = vi.fn((_request, callback) => { emit = callback; return new Promise<void>((resolve) => { finish = resolve; }); });
  backend.cancelTurn = vi.fn(async () => { emit?.({ type: 'run-finished', outcome: 'cancelled' }); finish?.(); });
  await open(backend); await login(); fireEvent.change(screen.getByLabelText('Message JV'), { target: { value: 'synthetic' } }); fireEvent.click(screen.getByRole('button', { name: /Send message/ }));
  fireEvent.click(await screen.findByRole('button', { name: /Stop/ }));
  await screen.findByText(/Remote job state is unknown/);
  await waitFor(() => expect(screen.queryByRole('button', { name: /Stop/ })).toBeNull());
  emit?.({ type: 'assistant-message', id: 'stale', text: 'LATE_STALE_EVENT' });
  expect(screen.queryByText('LATE_STALE_EVENT')).toBeNull();
});
