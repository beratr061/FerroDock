import { useEffect, useRef, useState, type RefObject } from "react";

import { dataFrame, decodeFrame, resizeFrame, terminalUrl } from "./protocol";
import { withStyleNonce } from "./styleNonce";

export type Phase = "connecting" | "open" | "ended" | "failed";

function token(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

function theme() {
  return {
    background: token("--bg"),
    foreground: token("--text"),
    cursor: token("--text"),
    cursorAccent: token("--bg"),
    selectionBackground: token("--surface-hover"),
    selectionForeground: token("--text"),
  };
}

async function loadXterm() {
  const [xterm, fit, links] = await Promise.all([
    import("@xterm/xterm"),
    import("@xterm/addon-fit"),
    import("@xterm/addon-web-links"),
    import("@xterm/xterm/css/xterm.css"),
  ]);
  return { Terminal: xterm.Terminal, FitAddon: fit.FitAddon, WebLinksAddon: links.WebLinksAddon };
}

export function useTerminalSession(container: string, host: RefObject<HTMLDivElement | null>) {
  const [phase, setPhase] = useState<Phase>("connecting");
  const [exitCode, setExitCode] = useState<number | null>(null);
  const [attempt, setAttempt] = useState(0);
  const socket = useRef<WebSocket | null>(null);

  useEffect(() => {
    let disposed = false;
    const cleanups: (() => void)[] = [];
    void loadXterm().then(({ Terminal, FitAddon, WebLinksAddon }) => {
      const element = host.current;
      if (disposed || !element) {
        return;
      }
      const term = new Terminal({
        fontFamily: '"JetBrains Mono", ui-monospace, monospace',
        fontSize: 13,
        cursorBlink: true,
        scrollback: 5000,
        theme: theme(),
      });
      const fit = new FitAddon();
      term.loadAddon(fit);
      term.loadAddon(new WebLinksAddon());
      withStyleNonce(() => term.open(element));
      fit.fit();
      const ws = new WebSocket(terminalUrl(container, term.cols, term.rows));
      ws.binaryType = "arraybuffer";
      socket.current = ws;
      let exited = false;
      ws.onopen = () => {
        setPhase("open");
        term.focus();
      };
      ws.onmessage = (event: MessageEvent<ArrayBuffer>) => {
        const frame = decodeFrame(event.data);
        if (frame?.kind === "data") {
          term.write(frame.bytes);
        } else if (frame?.kind === "exit") {
          exited = true;
          setExitCode(frame.code);
        }
      };
      ws.onclose = () => setPhase(exited ? "ended" : "failed");
      const input = term.onData((text) => ws.readyState === WebSocket.OPEN && ws.send(dataFrame(text)));
      const resize = term.onResize(({ cols, rows }) => ws.readyState === WebSocket.OPEN && ws.send(resizeFrame(cols, rows)));
      const observer = new ResizeObserver(() => fit.fit());
      observer.observe(element);
      cleanups.push(() => {
        observer.disconnect();
        input.dispose();
        resize.dispose();
        ws.close();
        term.dispose();
      });
    }).catch(() => setPhase("failed"));
    return () => {
      disposed = true;
      socket.current = null;
      cleanups.forEach((cleanup) => cleanup());
    };
  }, [container, host, attempt]);

  const send = (text: string): void => {
    const ws = socket.current;
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(dataFrame(text));
    }
  };

  const reconnect = (): void => {
    setPhase("connecting");
    setExitCode(null);
    setAttempt((value) => value + 1);
  };

  return { phase, exitCode, send, reconnect };
}
