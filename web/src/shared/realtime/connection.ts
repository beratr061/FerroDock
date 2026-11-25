export type Signal =
  | { type: "deployment_log"; deployment_id: string }
  | { type: "deployment_status"; application_id: string; deployment_id: string }
  | { type: "containers_changed" }
  | { type: "resync" };

type Listener = (signal: Signal) => void;

const MIN_BACKOFF_MS = 1_000;
const MAX_BACKOFF_MS = 30_000;
const RENEW_CODE = 4000;

const listeners = new Set<Listener>();
const statusListeners = new Set<() => void>();
let socket: WebSocket | null = null;
let connected = false;
let attempts = 0;
let retryTimer: ReturnType<typeof setTimeout> | undefined;
let wanted = false;

function setConnected(value: boolean): void {
  if (connected !== value) {
    connected = value;
    statusListeners.forEach((notify) => notify());
  }
}

function endpoint(): string {
  const scheme = window.location.protocol === "https:" ? "wss" : "ws";
  return `${scheme}://${window.location.host}/api/v1/ws`;
}

function isSignal(value: unknown): value is Signal {
  return typeof value === "object" && value !== null && typeof (value as { type?: unknown }).type === "string";
}

function scheduleReconnect(immediate: boolean): void {
  if (!wanted) {
    return;
  }
  const backoff = Math.min(MAX_BACKOFF_MS, MIN_BACKOFF_MS * 2 ** attempts);
  const delay = immediate ? 0 : backoff / 2 + Math.random() * (backoff / 2);
  attempts += 1;
  retryTimer = setTimeout(open, delay);
}

function open(): void {
  const current = new WebSocket(endpoint());
  socket = current;
  current.onopen = () => {
    attempts = 0;
    setConnected(true);
    listeners.forEach((listener) => listener({ type: "resync" }));
  };
  current.onmessage = (event: MessageEvent<string>) => {
    const parsed: unknown = JSON.parse(event.data);
    if (isSignal(parsed)) {
      listeners.forEach((listener) => listener(parsed));
    }
  };
  current.onclose = (event) => {
    if (socket === current) {
      socket = null;
      setConnected(false);
      scheduleReconnect(event.code === RENEW_CODE);
    }
  };
}

export function startRealtime(): () => void {
  wanted = true;
  if (!socket && retryTimer === undefined) {
    open();
  }
  return () => {
    wanted = false;
    clearTimeout(retryTimer);
    retryTimer = undefined;
    socket?.close();
    socket = null;
    setConnected(false);
  };
}

export function onSignal(listener: Listener): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function isRealtimeConnected(): boolean {
  return connected;
}

export function onConnectionChange(notify: () => void): () => void {
  statusListeners.add(notify);
  return () => statusListeners.delete(notify);
}

export function waitForSignal(match: (signal: Signal) => boolean, timeoutMs: number, abort: AbortSignal): Promise<void> {
  return new Promise((resolve) => {
    const finish = (): void => {
      clearTimeout(timer);
      unsubscribe();
      resolve();
    };
    const timer = setTimeout(finish, timeoutMs);
    const unsubscribe = onSignal((signal) => {
      if (match(signal) || signal.type === "resync") {
        finish();
      }
    });
    abort.addEventListener("abort", finish, { once: true });
  });
}
