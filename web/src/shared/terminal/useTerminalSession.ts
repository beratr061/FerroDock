import { useCallback, useEffect, useRef, useState, type RefObject } from "react";

import { ShellConnection, type Phase } from "./connection";
import { fitVisible, mountTerminal, type MountedTerminal } from "./xterm";

export type { Phase } from "./connection";

interface SessionOptions {
  container: string;
  sessionId: string | null;
  onSession: (id: string | null) => void;
}

function wire(mounted: MountedTerminal, connection: ShellConnection, element: HTMLElement): () => void {
  const { term } = mounted;
  const input = term.onData((text) => connection.send(text));
  const resize = term.onResize(({ cols, rows }) => connection.resize(cols, rows));
  const observer = new ResizeObserver(() => fitVisible(mounted, element));
  observer.observe(element);
  return () => {
    observer.disconnect();
    input.dispose();
    resize.dispose();
  };
}

export function useTerminalSession(options: SessionOptions, host: RefObject<HTMLDivElement | null>) {
  const [phase, setPhase] = useState<Phase>("connecting");
  const [exitCode, setExitCode] = useState<number | null>(null);
  const connection = useRef<ShellConnection | null>(null);
  const terminal = useRef<MountedTerminal | null>(null);
  const initial = useRef(options);

  useEffect(() => {
    let disposed = false;
    let unwire = (): void => undefined;
    const { container, sessionId, onSession } = initial.current;
    void mountTerminal(host.current ?? document.createElement("div"))
      .then((mounted) => {
        const element = host.current;
        if (disposed || !element) {
          mounted.term.dispose();
          return;
        }
        terminal.current = mounted;
        fitVisible(mounted, element);
        const shell = new ShellConnection(container, sessionId, {
          size: () => ({ cols: mounted.term.cols, rows: mounted.term.rows }),
          output: (bytes) => mounted.term.write(bytes),
          resumed: () => mounted.term.reset(),
          session: onSession,
          phase: (next, code) => {
            setPhase(next);
            setExitCode(code ?? null);
          },
        });
        connection.current = shell;
        unwire = wire(mounted, shell, element);
        shell.start();
        mounted.term.focus();
      })
      .catch(() => setPhase("failed"));
    return () => {
      disposed = true;
      unwire();
      connection.current?.dispose();
      terminal.current?.term.dispose();
      connection.current = null;
      terminal.current = null;
    };
  }, [host]);

  const send = useCallback((text: string) => connection.current?.send(text), []);

  return {
    phase,
    exitCode,
    send,
    restart: () => {
      terminal.current?.term.reset();
      setPhase("connecting");
      connection.current?.restart();
    },
    focus: () => terminal.current?.term.focus(),
    findNext: (query: string) => terminal.current?.search.findNext(query) ?? false,
    findPrevious: (query: string) => terminal.current?.search.findPrevious(query) ?? false,
    clearSearch: () => terminal.current?.search.clearDecorations(),
  };
}
