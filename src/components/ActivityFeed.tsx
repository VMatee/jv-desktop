import type { DesktopEvent } from '../types/events';
export interface Activity { id: number; event: DesktopEvent }
function label(event: DesktopEvent): string {
  switch (event.type) {
    case 'session-started': return 'Session started';
    case 'tool-started': return `Tool: ${event.name}`;
    case 'turn-started': return 'Turn started';
    case 'command-started': return 'Command started';
    case 'command-completed': return event.exitCode === 0 ? 'Command completed' : event.exitCode === null ? 'Command finished · status unknown' : `Command failed · exit ${event.exitCode}`;
    case 'files-changed': return `${event.files.length} file${event.files.length === 1 ? '' : 's'} changed`;
    case 'assistant-message': return 'Assistant response';
    case 'turn-completed': return 'Turn completed';
    case 'diagnostic': return event.reason === 'unknown-event' ? 'Unrecognized event ignored' : 'Invalid event skipped';
    case 'error': return 'Turn error';
    case 'run-finished': return event.outcome === 'cancelled' ? 'Stopped by you' : event.outcome === 'completed' ? 'Process finished' : event.outcome === 'timeout' ? 'Process timed out' : 'Process failed';
  }
}
export function ActivityFeed({ activities, busy }: { activities: Activity[]; busy: boolean }) {
  return <aside className="activity-panel" aria-label="Activity feed">
    <div className="panel-title"><h2>Activity</h2><span className="count">{activities.length}</span></div>
    <p className="panel-subtitle">A clear view of each step</p>
    {activities.length === 0 ? <div className="activity-empty"><span aria-hidden="true">⌁</span><p>Nothing running yet</p><small>Commands, file changes, and turn results will appear here.</small></div>
      : <ol className="activity-list">{activities.map(({ id, event }) => <li key={id} className={event.type === 'error' ? 'activity-error' : ''}>
        <span className="activity-dot" aria-hidden="true" /><div><p>{label(event)}</p>
          {(event.type === 'command-started' || event.type === 'command-completed') && <code>{event.command}</code>}
          {event.type === 'command-completed' && event.output && <details><summary>Output</summary><pre>{event.output}</pre></details>}
          {event.type === 'files-changed' && <ul className="file-list">{event.files.map((file, index) => <li key={index}><code>{file.path}</code><small>{file.kind}</small></li>)}</ul>}
          {event.type === 'error' && <small>{event.message}</small>}
        </div></li>)}</ol>}
    <div className="activity-status"><span className={busy ? 'status-dot active' : 'status-dot'} />{busy ? 'Turn in progress' : 'Ready when you are'}</div>
  </aside>;
}
