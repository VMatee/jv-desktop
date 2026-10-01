import { useEffect, useRef } from 'react';
export interface Message { id: string; role: 'user' | 'assistant'; text: string }
export function Conversation({ messages, busy, demo }: { messages: Message[]; busy: boolean; demo: boolean }) {
  const bottom = useRef<HTMLDivElement>(null);
  useEffect(() => { if (messages.length || busy) bottom.current?.scrollIntoView?.({ block: 'nearest' }); }, [messages, busy]);
  return <section className="conversation" aria-label="Conversation">
    {messages.length === 0 ? <div className="empty-state">
      <h1>Your project.<br />A thoughtful copilot.</h1>
      <p>Choose a folder, set your permissions, and start a conversation.</p>
      <div className="example-prompt"><span aria-hidden="true">↗</span> ช่วยตรวจ project นี้</div>
      <p className="empty-footnote">{demo ? 'This offline demo simulates a conversation. Your files stay untouched.' : 'Choose permissions, then sign in to start a native conversation. Sign-in contacts the selected service.'}</p>
    </div> : <div className="message-list" role="log" aria-live="polite" aria-relevant="additions text">
      {messages.map((message) => <article className={`message ${message.role}`} key={message.id}>
        <div className="avatar" aria-hidden="true">{message.role === 'user' ? 'Y' : 'jv'}</div>
        <div className="message-content"><div className="message-author">{message.role === 'user' ? 'You' : 'JV'}{message.role === 'assistant' && demo && <span>Simulated</span>}</div>
          <p className="message-text" dir="auto">{message.text}</p></div>
      </article>)}
      {busy && <div className="working" role="status"><span className="pulse" />JV is working…</div>}
    </div>}
    <div ref={bottom} />
  </section>;
}
