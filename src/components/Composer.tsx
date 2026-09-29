import { useEffect, useRef } from 'react';
import type { Permissions } from '../types/events';

interface Props {
  prompt: string; permissions: Permissions; busy: boolean; canSend: boolean;
  onPrompt: (text: string) => void; onPermissions: (value: Permissions) => void;
  onSend: () => void; onCancel: () => void;
}
export function Composer({ prompt, permissions, busy, canSend, onPrompt, onPermissions, onSend, onCancel }: Props) {
  const composing = useRef(false);
  const input = useRef<HTMLTextAreaElement>(null);
  const wasBusy = useRef(false);
  useEffect(() => { if (wasBusy.current && !busy) input.current?.focus(); wasBusy.current = busy; }, [busy]);
  return <section className="composer" aria-label="Message composer">
    <div className="permission-row">
      <label className="access-label">Access
        <select aria-label="Workspace permission" value={permissions.access} disabled={busy}
          onChange={(e) => onPermissions({ access: e.target.value as Permissions['access'], network: e.target.value === 'read-only' ? false : permissions.network })}>
          <option value="read-only">Read only</option><option value="workspace-write">Workspace write</option>
        </select>
      </label>
      <label className="network-control"><input type="checkbox" checked={permissions.network}
        disabled={busy || permissions.access === 'read-only'}
        onChange={(e) => onPermissions({ ...permissions, network: e.target.checked })} />Tool network</label>
      <span className="permission-note">Permissions apply to each new turn</span>
    </div>
    <form onSubmit={(e) => { e.preventDefault(); if (canSend && !composing.current) onSend(); }}>
      <label className="sr-only" htmlFor="message">Message JV</label>
      <textarea ref={input} id="message" rows={3} value={prompt} disabled={busy}
        placeholder="Ask JV about your project…" onChange={(e) => onPrompt(e.target.value)}
        onCompositionStart={() => { composing.current = true; }}
        onCompositionEnd={() => { composing.current = false; }}
        onKeyDown={(e) => {
          if (e.key === 'Enter' && !e.shiftKey && !e.nativeEvent.isComposing && !composing.current && e.keyCode !== 229) {
            e.preventDefault(); if (canSend) onSend();
          }
        }} aria-describedby="input-hint" />
      <div className="composer-footer"><span id="input-hint">Enter to send · Shift + Enter for a new line</span>
        {busy ? <button type="button" className="stop-button" onClick={onCancel}>■ Stop</button>
          : <button className="primary" type="submit" disabled={!canSend}>Send message <span aria-hidden="true">↑</span></button>}
      </div>
    </form>
  </section>;
}
