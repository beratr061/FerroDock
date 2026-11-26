export type LogSource = "stdout" | "stderr";

export interface LogEntry {
  id: number;
  source: LogSource;
  text: string;
}

export const MAX_LOG_LINES = 2000;

export function socketUrl(path: string, query: Record<string, string> = {}): string {
  const scheme = window.location.protocol === "https:" ? "wss" : "ws";
  const search = new URLSearchParams(query).toString();
  return `${scheme}://${window.location.host}${path}${search ? `?${search}` : ""}`;
}

export function parseLogLine(raw: unknown): Omit<LogEntry, "id"> | null {
  if (typeof raw !== "string") {
    return null;
  }
  try {
    const value: unknown = JSON.parse(raw);
    if (typeof value !== "object" || value === null) {
      return null;
    }
    const { source, text } = value as Record<string, unknown>;
    if ((source === "stdout" || source === "stderr") && typeof text === "string") {
      return { source, text };
    }
    return null;
  } catch {
    return null;
  }
}

export function appendCapped(current: LogEntry[], batch: LogEntry[]): LogEntry[] {
  const combined = current.concat(batch);
  return combined.length > MAX_LOG_LINES ? combined.slice(-MAX_LOG_LINES) : combined;
}
