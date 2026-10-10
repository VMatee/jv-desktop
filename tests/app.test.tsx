import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import App from '../src/App';
import { Composer } from '../src/components/Composer';
import { Conversation } from '../src/components/Conversation';
import { MockBackend } from '../src/lib/mock-backend';
import type { JvBackend } from '../src/types/events';

afterEach(cleanup);
function setup() {
  const demo = new MockBackend(async () => '/demo/workspace', 0);
  const native: JvBackend = {
    detectBackend: vi.fn(async () => ({ availability: 'unavailable' as const, canRun: false, detail: 'JV CLI is not installed.' })),
    chooseWorkspace: vi.fn(async () => '/chosen/workspace'), runTurn: vi.fn(), cancelTurn: vi.fn(),
  };
  render(<App backends={{ demo, native }} />); return { demo, native };
}
describe('workspace and conversation', () => {
  it('requires selection, completes an offline conversation, and stays usable', async () => {
    setup(); const send = screen.getByRole('button', { name: /Send message/ }) as HTMLButtonElement;
    fireEvent.change(screen.getByLabelText('Message JV'), { target: { value: 'ช่วยตรวจ project นี้' } });
    expect(send.disabled).toBe(true);
    fireEvent.click(screen.getByRole('button', { name: /Open Folder/ }));
    await waitFor(() => expect(send.disabled).toBe(false)); fireEvent.click(send);
    expect(await screen.findByText(/your offline preview is ready/)).toBeTruthy();
    expect(screen.getByText('ช่วยตรวจ project นี้')).toBeTruthy();
    expect(await screen.findByText('Turn complete')).toBeTruthy();
    expect(screen.getByText('Command completed')).toBeTruthy();
  });
  it('shows backend unavailable and never submits through that backend', async () => {
    const { native } = setup();
    fireEvent.change(screen.getByLabelText('Backend mode'), { target: { value: 'native' } });
    expect(await screen.findByText('JV CLI: unavailable')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: /Open Folder/ }));
    await screen.findByText('/chosen/workspace');
    fireEvent.change(screen.getByLabelText('Message JV'), { target: { value: 'test' } });
    fireEvent.click(screen.getByRole('button', { name: /Send message/ }));
    expect(native.runTurn).not.toHaveBeenCalled();
    expect(native.detectBackend).toHaveBeenCalledTimes(1);
  });
  it('disables networking in read-only mode and preserves explicit write permissions', async () => {
    setup(); const network = screen.getByLabelText('Tool network') as HTMLInputElement;
    expect(network.disabled).toBe(true); expect(network.checked).toBe(false);
    fireEvent.change(screen.getByLabelText('Workspace permission'), { target: { value: 'workspace-write' } });
    fireEvent.click(network); expect(network.checked).toBe(true);
    fireEvent.change(screen.getByLabelText('Workspace permission'), { target: { value: 'read-only' } });
    expect(network.checked).toBe(false); expect(network.disabled).toBe(true);
  });
  it('keeps assistant HTML inert', () => {
    const { container } = render(<Conversation busy={false} demo messages={[{ id: 'one', role: 'assistant', text: '<img src=x onerror=alert(1)><script>bad()</script>' }]} />);
    expect(container.querySelector('img,script')).toBeNull();
    expect(screen.getByText(/<img src=x/)).toBeTruthy();
  });
});

describe('IME-safe composer', () => {
  it('does not send during composition, keyCode 229, or Shift+Enter', () => {
    const send = vi.fn();
    render(<Composer prompt="สวัสดี" permissions={{ access: 'read-only', network: false }} busy={false} canSend
      onPrompt={() => {}} onPermissions={() => {}} onSend={send} onCancel={() => {}} />);
    const input = screen.getByLabelText('Message JV');
    fireEvent.compositionStart(input); fireEvent.keyDown(input, { key: 'Enter' }); expect(send).not.toHaveBeenCalled();
    fireEvent.compositionEnd(input); fireEvent.keyDown(input, { key: 'Enter', isComposing: true });
    fireEvent.keyDown(input, { key: 'Enter', keyCode: 229 });
    fireEvent.keyDown(input, { key: 'Enter', shiftKey: true }); expect(send).not.toHaveBeenCalled();
    fireEvent.keyDown(input, { key: 'Enter' }); expect(send).toHaveBeenCalledTimes(1);
  });
});

describe('memory-only bridge sign-in', () => {
  it.each(['structured', 'legacy'])('explicitly selects %s without exposing attachment inputs', async (transport) => {
    const authenticate = vi.fn(async () => {});
    const native: JvBackend = {
      detectBackend: async () => ({ availability: 'detected', canRun: true, detail: 'Synthetic CLI' }),
      chooseWorkspace: async () => '/synthetic/workspace', authenticate,
      runTurn: vi.fn(), cancelTurn: vi.fn(),
    };
    const { container } = render(<App backends={{ demo: native, native }} />);
    fireEvent.change(screen.getByLabelText('Backend mode'), { target: { value: 'native' } });
    await screen.findByText('JV CLI: detected');
    fireEvent.click(screen.getByRole('button', { name: /Open Folder/ }));
    const mode = await screen.findByLabelText('Session mode') as HTMLSelectElement;
    expect(mode.value).toBe('structured');
    fireEvent.change(mode, { target: { value: transport } });
    fireEvent.change(screen.getByLabelText('Username'), { target: { value: 'synthetic' } });
    fireEvent.change(screen.getByLabelText('Password'), { target: { value: 'FAKE_PASSWORD' } });
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));
    await waitFor(() => expect(authenticate).toHaveBeenCalledWith(expect.objectContaining({ structured: transport === 'structured' })));
    expect(container.querySelector('input[type=file]')).toBeNull();
    expect(await screen.findByText(transport === 'structured' ? 'Structured session · Agent image tools enabled' : 'Legacy session · Text tools only')).toBeTruthy();
  });

  it('requires explicit sign-in, clears password on failure, locks policy after success', async () => {
    const authenticate = vi.fn().mockRejectedValueOnce('authentication-required').mockResolvedValueOnce(undefined);
    const native: JvBackend = {
      detectBackend: async () => ({ availability: 'detected', canRun: true, detail: 'JV CLI 0.4.9' }),
      chooseWorkspace: async () => '/synthetic/workspace', authenticate,
      runTurn: vi.fn(async (_request, event) => { event({type:'run-finished',outcome:'cancelled'}); }), cancelTurn: vi.fn(),
    };
    render(<App backends={{demo:new MockBackend(async()=>'/demo',0),native}} />);
    fireEvent.change(screen.getByLabelText('Backend mode'), {target:{value:'native'}});
    await screen.findByText('JV CLI: detected');
    fireEvent.click(screen.getByRole('button',{name:/Open Folder/}));
    const password = await screen.findByLabelText('Password') as HTMLInputElement;
    fireEvent.change(screen.getByLabelText('Username'), {target:{value:'synthetic'}});
    fireEvent.change(screen.getByLabelText('Service URL'), {target:{value:'http://127.0.0.1:9'}});
    fireEvent.change(password, {target:{value:'TEST_PASSWORD_DO_NOT_LEAK'}});
    expect(authenticate).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button',{name:'Sign in'}));
    await waitFor(()=>expect(password.value).toBe(''));
    expect(sessionStorage.length).toBe(0); expect(localStorage.length).toBe(0);
    fireEvent.change(password, {target:{value:'TEST_PASSWORD_DO_NOT_LEAK'}});
    fireEvent.click(screen.getByRole('button',{name:'Sign in'}));
    await waitFor(()=>expect(screen.queryByLabelText('Password')).toBeNull());
    expect((screen.getByLabelText('Workspace permission') as HTMLSelectElement).disabled).toBe(true);
    fireEvent.change(screen.getByLabelText('Message JV'), {target:{value:'ไทย'}});
    fireEvent.click(screen.getByRole('button',{name:/Send message/}));
    expect(await screen.findByText(/Remote job state is unknown/)).toBeTruthy();
  });
});

describe('public support code presentation', () => {
  it.each(['JV-SVC-103', 'JV-SVC-002', 'JV-TIMEOUT-001', null])('renders %s through the event model', async (code) => {
    const { parseBridgeEvent } = await import('../src/lib/bridge-events');
    const backend: JvBackend = {
      detectBackend: async () => ({ availability: 'detected', canRun: true, detail: 'Synthetic' }),
      chooseWorkspace: async () => '/synthetic', cancelTurn: vi.fn(), authenticate: async () => {},
      runTurn: async (_request, onEvent) => {
        onEvent(parseBridgeEvent({ type: 'turn.failed', code: 'engine_error', source: 'remote', support_code: code,
          public_message: code === 'JV-SVC-103' ? 'The AI service requires administrator authentication.' : '<html>SECRET selector</html>' })!);
      },
    };
    render(<App backends={{ demo: backend, native: backend }} />);
    fireEvent.change(screen.getByLabelText('Backend mode'), { target: { value: 'native' } });
    await screen.findByText('JV CLI: detected');
    fireEvent.click(screen.getByRole('button', { name: /Open Folder/ }));
    await screen.findByText('/synthetic');
    fireEvent.change(await screen.findByLabelText('Username'), { target: { value: 'synthetic' } });
    fireEvent.change(screen.getByLabelText('Password'), { target: { value: 'FAKE_PASSWORD' } });
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));
    await waitFor(() => expect(screen.queryByLabelText('Password')).toBeNull());
    fireEvent.change(screen.getByLabelText('Message JV'), { target: { value: 'Synthetic test' } });
    const send = screen.getByRole('button', { name: /Send message/ }) as HTMLButtonElement;
    await waitFor(() => expect(send.disabled).toBe(false));
    fireEvent.click(send);
    const alert = await screen.findByRole('alert');
    if (code) {
      expect(alert.textContent).toContain('Error code: ' + code);
      expect(alert.querySelector('code')?.textContent).toBe(code);
      if (code === 'JV-SVC-103') expect(alert.textContent).toContain('The AI service requires administrator authentication.');
    } else {
      expect(alert.textContent).toContain('JV CLI could not complete this turn. No detailed cause was provided.');
      expect(alert.textContent).not.toContain('sign in again');
      expect(alert.querySelector('code')).toBeNull();
    }
    expect(screen.queryByLabelText('Password')).toBeNull();
    expect(screen.getByText('Turn did not complete')).toBeTruthy();
    expect(screen.queryByText('Session ready. Send a task when you are ready.')).toBeNull();
    expect(document.body.textContent).not.toContain('SECRET');
    expect(localStorage.length).toBe(0);
    expect(sessionStorage.length).toBe(0);
    fireEvent.click(screen.getByRole('button', { name: 'Dismiss error' }));
    expect(screen.queryByRole('alert')).toBeNull();
  });
});

describe('administrator authentication wait lifecycle', () => {
  it.each(['success', 'failure', 'cancel', 'timeout'])('keeps the turn active until %s', async (outcome) => {
    const { act } = await import('@testing-library/react');
    const { parseBridgeEvent } = await import('../src/lib/bridge-events');
    let emit: (wire: unknown) => void = () => { throw new Error('No active turn'); };
    let finish: () => void = () => {};
    let fail: (e: unknown) => void = () => {};
    const backend: JvBackend = {
      detectBackend: async () => ({ availability:'detected', canRun:true, detail:'Synthetic' }),
      chooseWorkspace: async () => '/synthetic', authenticate: async () => {},
      runTurn: vi.fn(async (_request, onEvent) => new Promise<void>((resolve, reject) => {
        finish = resolve; fail = reject;
        emit = (wire) => onEvent(parseBridgeEvent(wire)!);
        emit({ type:'turn.status', source:'remote', status:'waiting_for_auth', support_code:'JV-SVC-103',
          public_message:'The AI service requires administrator authentication.' });
      })),
      cancelTurn: vi.fn(async () => { emit({type:'turn.cancelled',local:true,remote_state:'unknown'}); finish(); }),
    };
    render(<App backends={{ demo:backend, native:backend }} />);
    fireEvent.change(screen.getByLabelText('Backend mode'), {target:{value:'native'}});
    await screen.findByText('JV CLI: detected');
    fireEvent.click(screen.getByRole('button', {name:/Open Folder/}));
    fireEvent.change(await screen.findByLabelText('Username'), {target:{value:'synthetic'}});
    fireEvent.change(screen.getByLabelText('Password'), {target:{value:'FAKE_PASSWORD'}});
    fireEvent.click(screen.getByRole('button', {name:'Sign in'}));
    await waitFor(() => expect(screen.queryByLabelText('Password')).toBeNull());
    fireEvent.change(screen.getByLabelText('Message JV'), {target:{value:'synthetic task'}});
    fireEvent.click(screen.getByRole('button', {name:/Send message/}));
    const waiting = await screen.findByRole('status', {name:'Remote service status'});
    expect(waiting.textContent).toContain('Support code: JV-SVC-103');
    expect(waiting.textContent).toContain('This turn remains active');
    expect(screen.queryByRole('alert')).toBeNull();
    expect(screen.queryByLabelText('Password')).toBeNull();
    expect((screen.getByLabelText('Message JV') as HTMLTextAreaElement).disabled).toBe(true);
    expect(screen.queryByRole('button', {name:/Send message/})).toBeNull();
    expect((screen.getByRole('button', {name:/Stop/}) as HTMLButtonElement).disabled).toBe(false);
    if (outcome === 'success') {
      await act(async () => emit({type:'turn.status',source:'remote',status:'running'}));
      expect(screen.queryByRole('status', {name:'Remote service status'})).toBeNull();
      expect(screen.getByRole('button', {name:/Stop/})).toBeTruthy();
      await act(async () => { emit({type:'assistant.message',text:'Completed after refresh'}); emit({type:'turn.completed'}); finish(); });
      expect(await screen.findByText('Turn complete')).toBeTruthy();
      expect(screen.getByText('Completed after refresh')).toBeTruthy();
    } else if (outcome === 'failure') {
      await act(async () => { emit({type:'turn.failed',source:'remote',support_code:'JV-TIMEOUT-001'}); finish(); });
      expect((await screen.findByRole('alert')).textContent).toContain('JV-TIMEOUT-001');
      expect(screen.getByRole('alert').textContent).not.toContain('JV-SVC-103');
    } else if (outcome === 'cancel') {
      fireEvent.click(screen.getByRole('button', {name:/Stop/}));
      expect(await screen.findByText(/Stopped locally. Remote job state is unknown/)).toBeTruthy();
      expect(backend.cancelTurn).toHaveBeenCalledTimes(1);
    } else {
      await act(async () => fail('timeout'));
      expect((await screen.findByRole('alert')).textContent).toContain('local process timed out');
      expect(screen.getByRole('alert').textContent).not.toContain('JV-SVC-103');
    }
    await waitFor(() => expect(screen.queryByRole('button', {name:/Stop/})).toBeNull());
    expect(screen.queryByRole('status', {name:'Remote service status'})).toBeNull();
    expect(backend.runTurn).toHaveBeenCalledTimes(1);
    expect(localStorage.length).toBe(0);
  });
});
