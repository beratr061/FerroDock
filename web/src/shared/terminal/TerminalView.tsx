import { useRef } from "react";
import { ArrowClockwiseIcon } from "@phosphor-icons/react";
import { useTranslation } from "react-i18next";

import { Button } from "../ui";

import { useTerminalSession, type Phase } from "./useTerminalSession";

const KEYS: { label: string; sequence: string }[] = [
  { label: "Esc", sequence: "\u001b" },
  { label: "Tab", sequence: "\t" },
  { label: "Ctrl+C", sequence: "\u0003" },
  { label: "Ctrl+D", sequence: "\u0004" },
  { label: "↑", sequence: "\u001b[A" },
  { label: "↓", sequence: "\u001b[B" },
];

function StatusLine({ phase, exitCode, onReconnect }: { phase: Phase; exitCode: number | null; onReconnect: () => void }) {
  const { t } = useTranslation("common");
  if (phase === "open" || phase === "connecting") {
    return <span className="text-[12px] text-subtle">{t(`terminal.${phase}`)}</span>;
  }
  const message = phase === "ended" ? t("terminal.ended", { code: exitCode ?? "?" }) : t("terminal.failed");
  return (
    <span className="flex items-center gap-2 text-[12px] text-muted">
      {message}
      <Button variant="ghost" className="h-7 px-2" icon={<ArrowClockwiseIcon size={14} aria-hidden />} onClick={onReconnect}>
        {t("terminal.reconnect")}
      </Button>
    </span>
  );
}

export default function TerminalView({ container }: { container: string }) {
  const host = useRef<HTMLDivElement>(null);
  const session = useTerminalSession(container, host);

  return (
    <div className="flex h-full min-h-0 flex-col gap-2">
      <div ref={host} className="terminal-host min-h-0 flex-1 overflow-hidden rounded-control border border-border bg-bg p-2" />
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex flex-wrap gap-1">
          {KEYS.map((key) => (
            <button
              key={key.label}
              type="button"
              onClick={() => session.send(key.sequence)}
              className="press h-7 rounded-control border border-border bg-surface-raised px-2 font-mono text-[12px] text-muted hover:text-text"
            >
              {key.label}
            </button>
          ))}
        </div>
        <StatusLine phase={session.phase} exitCode={session.exitCode} onReconnect={session.reconnect} />
      </div>
    </div>
  );
}
