const DATA = 0;
const CONTROL = 1;
const encoder = new TextEncoder();

export type ServerFrame =
  | { kind: "data"; bytes: Uint8Array }
  | { kind: "exit"; code: number | null }
  | { kind: "session"; id: string };

export type TerminalTarget = { container: string } | { session: string };

function controlMessage(payload: Uint8Array): ServerFrame | null {
  const message: unknown = JSON.parse(new TextDecoder().decode(payload));
  if (typeof message !== "object" || message === null) {
    return null;
  }
  const { type, code, id } = message as Record<string, unknown>;
  if (type === "exit") {
    return { kind: "exit", code: typeof code === "number" ? code : null };
  }
  if (type === "session" && typeof id === "string") {
    return { kind: "session", id };
  }
  return null;
}

function frameOf(kind: number, payload: Uint8Array): Uint8Array<ArrayBuffer> {
  const frame = new Uint8Array(payload.length + 1);
  frame[0] = kind;
  frame.set(payload, 1);
  return frame;
}

export function dataFrame(text: string): Uint8Array<ArrayBuffer> {
  return frameOf(DATA, encoder.encode(text));
}

export function resizeFrame(cols: number, rows: number): Uint8Array<ArrayBuffer> {
  return frameOf(CONTROL, encoder.encode(JSON.stringify({ type: "resize", cols, rows })));
}

export function decodeFrame(buffer: ArrayBuffer): ServerFrame | null {
  const frame = new Uint8Array(buffer);
  if (frame[0] === DATA) {
    return { kind: "data", bytes: frame.subarray(1) };
  }
  if (frame[0] === CONTROL) {
    try {
      return controlMessage(frame.subarray(1));
    } catch {
      return null;
    }
  }
  return null;
}

export function terminalUrl(target: TerminalTarget, cols: number, rows: number): string {
  const scheme = window.location.protocol === "https:" ? "wss" : "ws";
  const query = new URLSearchParams({ ...target, cols: String(cols), rows: String(rows) });
  return `${scheme}://${window.location.host}/api/v1/terminal?${query.toString()}`;
}

const BASE_DELAY_MS = 1_000;
const MAX_DELAY_MS = 10_000;

export function reconnectDelay(attempt: number): number {
  return Math.min(MAX_DELAY_MS, BASE_DELAY_MS * 2 ** Math.max(0, attempt));
}
