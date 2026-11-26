import { dataFrame, decodeFrame, reconnectDelay, resizeFrame, terminalUrl, type TerminalTarget } from "./protocol";

export type Phase = "connecting" | "open" | "reconnecting" | "ended" | "expired" | "failed";

const MAX_ATTEMPTS = 8;
const RESUME_ATTEMPTS = 3;

export interface ConnectionEvents {
  size: () => { cols: number; rows: number };
  output: (bytes: Uint8Array) => void;
  resumed: () => void;
  session: (id: string | null) => void;
  phase: (phase: Phase, exitCode?: number | null) => void;
}

export class ShellConnection {
  private socket: WebSocket | null = null;
  private timer = 0;
  private attempt = 0;
  private closed = false;
  private connected = false;

  constructor(
    private readonly container: string,
    private sessionId: string | null,
    private readonly events: ConnectionEvents,
  ) {}

  start(): void {
    this.closed = false;
    this.connect();
  }

  send(text: string): void {
    if (this.socket?.readyState === WebSocket.OPEN) {
      this.socket.send(dataFrame(text));
    }
  }

  resize(cols: number, rows: number): void {
    if (this.socket?.readyState === WebSocket.OPEN) {
      this.socket.send(resizeFrame(cols, rows));
    }
  }

  restart(): void {
    this.dispose();
    this.sessionId = null;
    this.connected = false;
    this.events.session(null);
    this.attempt = 0;
    this.start();
  }

  dispose(): void {
    this.closed = true;
    window.clearTimeout(this.timer);
    this.socket?.close();
    this.socket = null;
  }

  private target(): TerminalTarget {
    return this.sessionId ? { session: this.sessionId } : { container: this.container };
  }

  private connect(): void {
    const { cols, rows } = this.events.size();
    const resuming = this.sessionId !== null;
    const socket = new WebSocket(terminalUrl(this.target(), cols, rows));
    socket.binaryType = "arraybuffer";
    this.socket = socket;
    const seen = { session: false, exit: false };
    socket.onmessage = (event: MessageEvent<ArrayBuffer>) => this.receive(event.data, seen, resuming);
    socket.onclose = () => this.closedWith(seen, resuming);
  }

  private receive(buffer: ArrayBuffer, seen: { session: boolean; exit: boolean }, resuming: boolean): void {
    const frame = decodeFrame(buffer);
    if (frame?.kind === "session") {
      seen.session = true;
      this.connected = true;
      this.attempt = 0;
      this.sessionId = frame.id;
      this.events.session(frame.id);
      if (resuming) {
        this.events.resumed();
      }
      this.events.phase("open");
    } else if (frame?.kind === "data") {
      this.events.output(frame.bytes);
    } else if (frame?.kind === "exit") {
      seen.exit = true;
      this.sessionId = null;
      this.events.session(null);
      this.events.phase("ended", frame.code);
    }
  }

  private closedWith(seen: { session: boolean; exit: boolean }, resuming: boolean): void {
    if (this.closed || seen.exit) {
      return;
    }
    const limit = this.connected ? MAX_ATTEMPTS : RESUME_ATTEMPTS;
    if (!resuming && !seen.session) {
      this.events.phase("failed");
    } else if (this.attempt >= limit) {
      this.giveUp();
    } else {
      this.events.phase("reconnecting");
      this.timer = window.setTimeout(() => this.connect(), reconnectDelay(this.attempt));
      this.attempt += 1;
    }
  }

  private giveUp(): void {
    this.sessionId = null;
    this.events.session(null);
    this.events.phase("expired");
  }
}
