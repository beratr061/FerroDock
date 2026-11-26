import { useEffect, useRef, useState } from "react";
import { ArrowClockwiseIcon, MagnifyingGlassIcon, XIcon } from "@phosphor-icons/react";

import { useTranslation } from "../i18n";
import { cx } from "../lib/cx";

import { SearchBar } from "./SearchBar";
import { focusPane, registerSender, rememberSession, type Pane } from "./store";
import { useTerminalSession, type Phase } from "./useTerminalSession";

const ICON_BUTTON = "press grid size-7 place-items-center rounded-control text-muted hover:bg-surface-hover hover:text-text";

function statusText(t: (key: string, options?: Record<string, unknown>) => string, phase: Phase, code: number | null): string {
  return phase === "ended" ? t("terminal.ended", { code: code ?? "?" }) : t(`terminal.${phase}`);
}

interface TerminalPaneProps {
  pane: Pane;
  container: string;
  focused: boolean;
  onClose: () => void;
}

export function TerminalPane({ pane, container, focused, onClose }: TerminalPaneProps) {
  const { t } = useTranslation("common");
  const host = useRef<HTMLDivElement>(null);
  const [searching, setSearching] = useState(false);
  const session = useTerminalSession(
    { container, sessionId: pane.sessionId, onSession: (id) => rememberSession(pane.id, id) },
    host,
  );
  const { send } = session;
  const stopped = session.phase === "ended" || session.phase === "expired" || session.phase === "failed";

  useEffect(() => registerSender(pane.id, send), [pane.id, send]);

  return (
    <div
      onFocusCapture={() => focusPane(pane.id)}
      onPointerDown={() => focusPane(pane.id)}
      className={cx(
        "flex min-h-0 min-w-0 flex-1 flex-col rounded-control border bg-bg",
        focused ? "border-border-strong" : "border-border",
      )}
    >
      <div className="flex min-h-9 flex-wrap items-center justify-between gap-2 border-b border-border px-2 py-1">
        <span className={cx("text-[12px]", stopped ? "text-muted" : "text-subtle")}>
          {statusText(t, session.phase, session.exitCode)}
        </span>
        <div className="flex items-center gap-1">
          {stopped ? (
            <button type="button" className={ICON_BUTTON} title={t("terminal.newShell")} onClick={session.restart}>
              <ArrowClockwiseIcon size={14} aria-hidden />
            </button>
          ) : null}
          {searching ? (
            <SearchBar
              onNext={session.findNext}
              onPrevious={session.findPrevious}
              onClose={() => {
                session.clearSearch();
                setSearching(false);
                session.focus();
              }}
            />
          ) : (
            <button type="button" className={ICON_BUTTON} title={t("terminal.search")} onClick={() => setSearching(true)}>
              <MagnifyingGlassIcon size={14} aria-hidden />
            </button>
          )}
          <button type="button" className={ICON_BUTTON} title={t("terminal.closePane")} onClick={onClose}>
            <XIcon size={14} aria-hidden />
          </button>
        </div>
      </div>
      <div ref={host} className="terminal-host min-h-0 flex-1 overflow-hidden p-2" />
    </div>
  );
}
