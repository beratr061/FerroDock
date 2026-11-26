import { useEffect, useState } from "react";

import { appendCapped, parseLogLine, type LogEntry } from "./protocol";

export type LogConnection = "connecting" | "live" | "closed";

export function useContainerLogs(url: string) {
  const [lines, setLines] = useState<LogEntry[]>([]);
  const [connection, setConnection] = useState<LogConnection>("connecting");

  useEffect(() => {
    let pending: LogEntry[] = [];
    let frame = 0;
    let counter = 0;
    const flush = (): void => {
      frame = 0;
      const batch = pending;
      pending = [];
      setLines((current) => appendCapped(current, batch));
    };
    const socket = new WebSocket(url);
    socket.onopen = () => setConnection("live");
    socket.onclose = () => setConnection("closed");
    socket.onmessage = (event: MessageEvent) => {
      const line = parseLogLine(event.data);
      if (line) {
        pending.push({ id: counter++, ...line });
        frame ||= requestAnimationFrame(flush);
      }
    };
    return () => {
      cancelAnimationFrame(frame);
      socket.close();
    };
  }, [url]);

  return { lines, connection };
}
