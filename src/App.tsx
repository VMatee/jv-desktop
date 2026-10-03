import packageInfo from '../package.json';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { ActivityFeed } from './components/ActivityFeed';
import type { Activity } from './components/ActivityFeed';
import { Composer } from './components/Composer';
import { Conversation } from './components/Conversation';
import type { Message } from './components/Conversation';
import { createBackends } from './lib/backends';
import type { BackendMode } from './lib/backends';
import { safeError } from './lib/errors';
import { validateRequest } from './lib/requests';
import type { BackendStatus, DesktopEvent, JvBackend, Permissions } from './types/events';

export default function App({ backends: supplied }: { backends?: Record<BackendMode, JvBackend> }) {
  const backends = useMemo(() => supplied ?? createBackends(), [supplied]);
  const [mode, setMode] = useState<BackendMode>('demo');
  const backend = backends[mode];
  const [status, setStatus] = useState<BackendStatus | null>(null);
  const [checking, setChecking] = useState(false);
  const [workspace, setWorkspace] = useState<string | null>(null);
  const [choosing, setChoosing] = useState(false);
  const [signedIn, setSignedIn] = useState(false);
  const [signingIn, setSigningIn] = useState(false);
  const [username, setUsername] = useState('');
  const [baseUrl, setBaseUrl] = useState('https://ai.openjvspace.com');
  const [password, setPassword] = useState('');
  const [prompt, setPrompt] = useState('');
  const [permissions, setPermissions] = useState<Permissions>({ access: 'read-only', network: false });
  const trustedHost = mode === 'native' && status?.executionMode === 'trusted-host';
  const activePermissions: Permissions = trustedHost ? { access: 'workspace-write', network: true } : permissions;
  const [messages, setMessages] = useState<Message[]>([]);
  const [activities, setActivities] = useState<Activity[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [outcome, setOutcome] = useState('');
  const nextActivity = useRef(0);
  const active = useRef<{ id: string; backend: JvBackend } | null>(null);
  const mounted = useRef(true);
  const detection = useRef(0);

  const check = useCallback(async () => {
    const generation = ++detection.current; setChecking(true);
    try { const value = await backend.detectBackend(); if (mounted.current && generation === detection.current) setStatus(value); }
    catch { if (mounted.current && generation === detection.current) setStatus({ availability: 'unavailable', canRun: false, detail: 'Backend check failed. Try checking again.' }); }
    finally { if (mounted.current && generation === detection.current) setChecking(false); }
  }, [backend]);
  useEffect(() => { setStatus(null); void check(); return () => { ++detection.current; }; }, [check]);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; const run = active.current; if (run) void run.backend.cancelTurn(run.id).catch(() => {}); }; }, []);

  const canSend = !!workspace && !!status?.canRun && (mode === 'demo' || signedIn) && !!prompt.trim() && !busy && !choosing && !checking;
  async function chooseWorkspace() {
    if (busy || choosing) return;
    setChoosing(true); setError(null); setPassword('');
    try { const selected = await backend.chooseWorkspace(); if (selected && mounted.current) {
      setSignedIn(false); setWorkspace(selected); if (selected !== workspace) { setMessages([]); setActivities([]); setOutcome(''); }
    } }
    catch (failure) { if (mounted.current) { setSignedIn(false); setError(safeError(failure).message); } }
    finally { if (mounted.current) setChoosing(false); }
  }
  function receive(runId: string, event: DesktopEvent) {
    if (!mounted.current || active.current?.id !== runId) return;
    const id = ++nextActivity.current;
    setActivities((previous) => [...previous, { id, event }].slice(-200));
    if (event.type === 'assistant-message') setMessages((previous) => {
      const id = `${runId}:${event.id}`;
      const message: Message = { id, role: 'assistant', text: event.text };
      const index = previous.findIndex((item) => item.id === id);
      return index < 0 ? [...previous, message].slice(-100) : previous.map((item) => item.id === id ? message : item);
    });
    if (event.type === 'error') setError(event.message);
    if (event.type === 'run-finished') setOutcome(event.outcome === 'cancelled' ? (mode === 'demo' ? 'Turn stopped. No further demo events will arrive.' : 'Stopped locally. Remote job state is unknown; a submitted job may continue.') : event.outcome === 'completed' ? 'Turn complete' : 'Turn did not complete');
  }
  async function send() {
    if (!canSend || !workspace || active.current) return;
    const runId = crypto.randomUUID();
    const request = { runId, workspace, prompt, permissions: { ...activePermissions } };
    try { validateRequest(request); } catch (failure) { setError(safeError(failure).message); return; }
    active.current = { id: runId, backend }; setBusy(true); setError(null); setOutcome('');
    setMessages((previous) => [...previous, { id: runId, role: 'user' as const, text: prompt }].slice(-100)); setPrompt('');
    try { await backend.runTurn(request, (event) => receive(runId, event)); }
    catch (failure) { if (mode === 'native') setSignedIn(false); const safe = safeError(failure); receive(runId, { type: 'error', category: safe.category, message: safe.message }); }
    finally { if (active.current?.id === runId) active.current = null; if (mounted.current) setBusy(false); }
  }
  async function cancel() {
    const run = active.current; if (!run) return;
    try { await run.backend.cancelTurn(run.id); }
    catch (failure) { if (mounted.current) setError(safeError(failure).message); }
  }
  async function login() {
    if (!workspace || !backend.authenticate || signingIn) return;
    setSigningIn(true); setError(null);
    try { await backend.authenticate({ username, baseUrl, password, permissions: activePermissions }); setSignedIn(true); }
    catch (failure) { setError(safeError(failure).message); setSignedIn(false); }
    finally { setPassword(''); setSigningIn(false); }
  }
  return <div className="app-shell">
    <header className="app-header"><div className="brand"><span className="brand-mark" aria-hidden="true">jv</span><div>JV Desktop<span className="brand-subtitle">Your workspace, in conversation</span></div></div>
      <div className="header-controls"><label className="sr-only" htmlFor="backend-mode">Backend mode</label>
        <select id="backend-mode" value={mode} disabled={busy || choosing || signingIn} onChange={(e) => { void backends.native.close?.().catch(() => {}); setSignedIn(false); setPassword(''); setMode(e.target.value as BackendMode); setWorkspace(null); setError(null); setMessages([]); setActivities([]); setOutcome(''); }}>
          <option value="demo">Offline demo</option><option value="native">Native JV CLI</option>
        </select><span className="preview-label">v{packageInfo.version}</span></div>
    </header>
    <div className="workspace-bar"><span className="folder-icon" aria-hidden="true">▱</span><div className="workspace-location"><span>WORKSPACE</span><p title={workspace ?? undefined}>{workspace ?? 'Choose a project folder to begin'}</p></div>
      <button className="secondary" onClick={() => void chooseWorkspace()} disabled={busy || choosing || signingIn}>{choosing ? 'Choosing…' : 'Open Folder'}<span aria-hidden="true">↗</span></button>
    </div>
    <div className="backend-banner"><span className="status-dot" /><strong>{checking ? 'Checking backend…' : mode === 'demo' ? 'Offline preview' : `JV CLI: ${status?.availability ?? 'unavailable'}`}</strong><span>{status?.detail}</span>
      {mode === 'native' && !status?.canRun && <span>Requires JV CLI {status?.minimumCliVersion ?? (trustedHost ? '0.5.0' : '0.4.9')}+ for {trustedHost ? 'Windows' : 'Linux'}. Install from github.com/VMatee/jv-cli/releases, then Check again.</span>}
      {mode === 'native' && <button className="text-button" onClick={() => void check()} disabled={checking || busy}>Check again</button>}
    </div>
    {mode === 'native' && workspace && status?.canRun && !signedIn && <form className="auth-form" autoComplete="off" onSubmit={(e) => { e.preventDefault(); void login(); }}>
      <label>Username<input value={username} onChange={(e) => setUsername(e.target.value)} disabled={signingIn} autoComplete="off" /></label>
      <label>Service URL<input type="url" value={baseUrl} onChange={(e) => setBaseUrl(e.target.value)} disabled={signingIn} /></label>
      <label>Password<input type="password" value={password} onChange={(e) => setPassword(e.target.value)} disabled={signingIn} autoComplete="new-password" /></label>
      <button className="primary" disabled={signingIn || !username || !password}>{signingIn ? 'Signing in / starting session…' : 'Sign in'}</button>
      <small>Sign-in contacts the selected service. Password is held in memory only. {trustedHost ? 'Signing in starts a trusted-host session on this dedicated worker.' : 'Choose permissions below first; signing in also starts your workspace session.'}</small>
    </form>}
    <main className="main-grid"><div className="chat-panel"><div className="panel-heading"><h2>Conversation</h2><span>{mode === 'demo' ? 'Simulated session' : signedIn ? 'Session ready' : 'Sign-in required'}</span></div>
      <Conversation messages={messages} busy={busy} demo={mode === 'demo'} />
      {error && <div className="error-banner" role="alert">{error}<button aria-label="Dismiss error" onClick={() => setError(null)}>×</button></div>}
      <div className="turn-outcome" role="status">{outcome || (busy ? 'Turn running…' : signingIn ? 'Signing in and starting your workspace session…' : !workspace ? 'Choose a folder. Select Native JV CLI to connect, or try the offline demo.' : mode === 'native' ? signedIn ? 'Session ready. Send a task when you are ready.' : trustedHost ? 'Sign in to start a trusted-host session.' : 'Choose permissions, then sign in to start a session.' : 'Offline demo ready. No account needed.')}</div>
      <Composer trustedHost={trustedHost} permissionLocked={mode === 'native' && (signedIn || signingIn)} prompt={prompt} permissions={permissions} busy={busy} canSend={canSend} onPrompt={setPrompt} onPermissions={setPermissions} onSend={() => void send()} onCancel={() => void cancel()} />
    </div><ActivityFeed activities={activities} busy={busy} /></main>
    <footer className="app-footer"><span>Local workspace · Explicit permissions</span><span>{mode === 'demo' ? 'No service calls. No file changes.' : 'Bridge protocol 1 · Passwords are not saved.'}</span></footer>
  </div>;
}
