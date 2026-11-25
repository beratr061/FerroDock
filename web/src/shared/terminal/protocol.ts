const DATA = 0;
const CONTROL = 1;
const encoder = new TextEncoder();

export type ServerFrame = { kind: "data"; bytes: Uint8Array } | { kind: "exit"; code: number | null };

export function dataFrame(text: string): Uint8Array<ArrayBuffer> {
  const payload = encoder.encode(text);
  const frame = new Uint8Array(payload.length + 1);
  frame[0] = DATA;
  frame.set(payload, 1);
  return frame;
}

export function resizeFrame(cols: number, rows: number): Uint8Array<ArrayBuffer> {
  const payload = encoder.encode(JSON.stringify({ type: "resize", cols, rows }));
  const frame = new Uint8Array(payload.length + 1);
  frame[0] = CONTROL;
  frame.set(payload, 1);
  return frame;
}

export function decodeFrame(buffer: ArrayBuffer): ServerFrame | null {
  const frame = new Uint8Array(buffer);
  if (frame[0] === DATA) {
    return { kind: "data", bytes: frame.subarray(1) };
  }
  if (frame[0] === CONTROL) {
    const message: unknown = JSON.parse(new TextDecoder().decode(frame.subarray(1)));
    if (typeof message === "object" && message !== null && (message as { type?: unknown }).type === "exit") {
      const code = (message as { code?: unknown }).code;
      return { kind: "exit", code: typeof code === "number" ? code : null };
    }
  }
  return null;
}

export function terminalUrl(container: string, cols: number, rows: number): string {
  const scheme = window.location.protocol === "https:" ? "wss" : "ws";
  const query = new URLSearchParams({ container, cols: String(cols), rows: String(rows) });
  return `${scheme}://${window.location.host}/api/v1/terminal?${query.toString()}`;
}
