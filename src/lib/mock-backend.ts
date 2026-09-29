import type { DesktopEvent, JvBackend, TurnRequest } from '../types/events';
import { BackendError } from './errors';
import { parseLine } from './jsonl';
import { validateRequest } from './requests';

const demoLines = [
  { type: 'thread.started', thread_id: 'demo-session' },
  { type: 'turn.started' },
  { type: 'item.started', item: { id: 'check', type: 'command_execution', command: 'project checks (simulated)' } },
  { type: 'item.completed', item: { id: 'check', type: 'command_execution', command: 'project checks (simulated)', exit_code: 0, aggregated_output: 'All sample checks passed. No command was executed.' } },
  { type: 'item.completed', item: { id: 'answer', type: 'agent_message', text: 'เรียบร้อยแล้ว — your offline preview is ready.\n\nThis is a simulated response. No project files were read or changed, and no JV service was contacted.' } },
  { type: 'turn.completed' },
].map((event) => JSON.stringify(event));

export class MockBackend implements JvBackend {
  private running = new Map<string, AbortController>();
  constructor(private readonly picker: () => Promise<string | null> = async () => '/demo/workspace', private readonly delay = 300) {}
  async detectBackend() { return { availability: 'demo' as const, canRun: true, detail: 'Offline demo · simulated responses · no file or network access' }; }
  chooseWorkspace() { return this.picker(); }
  async runTurn(request: TurnRequest, onEvent: (event: DesktopEvent) => void) {
    validateRequest(request);
    if (this.running.size) throw new BackendError('internal');
    const controller = new AbortController(); this.running.set(request.runId, controller);
    try {
      for (const line of demoLines) {
        await new Promise<void>((resolve) => {
          const done = () => { clearTimeout(timer); controller.signal.removeEventListener('abort', done); resolve(); };
          const timer = setTimeout(done, this.delay);
          controller.signal.addEventListener('abort', done, { once: true });
          if (controller.signal.aborted) done();
        });
        if (controller.signal.aborted) break;
        const event = parseLine(line); if (event) onEvent(event);
      }
      onEvent({ type: 'run-finished', outcome: controller.signal.aborted ? 'cancelled' : 'completed' });
    } finally { this.running.delete(request.runId); }
  }
  async cancelTurn(runId: string) { this.running.get(runId)?.abort(); }
}
