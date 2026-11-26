import { ArrowClockwiseIcon, PlayIcon, StopIcon, TerminalWindowIcon } from "@phosphor-icons/react";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { StatusBadge, type StatusTone } from "@/shared/ui";

import { useContainerAction, type Container, type ContainerAction } from "./api";

const TONE_BY_STATE: Record<string, StatusTone> = {
  running: "success",
  restarting: "warning",
  paused: "warning",
  created: "info",
  exited: "neutral",
  dead: "danger",
  removing: "warning",
};

function ActionButton({ label, onClick, disabled, children }: {
  label: string;
  onClick: () => void;
  disabled: boolean;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      title={label}
      onClick={onClick}
      disabled={disabled}
      className="press grid size-9 place-items-center rounded-control text-muted hover:bg-surface-hover hover:text-text disabled:text-subtle disabled:hover:bg-transparent"
    >
      {children}
      <span className="sr-only">{label}</span>
    </button>
  );
}

export function ContainerRow({ container, onTerminal }: { container: Container; onTerminal: () => void }) {
  const { t } = useTranslation("servers");
  const action = useContainerAction();
  const running = container.state === "running";
  const run = (kind: ContainerAction): void => action.mutate({ reference: container.id, action: kind });
  const busy = action.isPending;

  return (
    <li className="grid gap-x-6 gap-y-2 border-b border-border px-4 py-3.5 last:border-b-0 md:grid-cols-[minmax(0,2fr)_minmax(0,2fr)_auto_auto] md:items-center">
      <div className="min-w-0">
        <p className="truncate font-mono text-[13px] text-text">{container.name}</p>
        <p className="truncate text-[12px] text-subtle">
          {container.compose_project ?? t("standalone")}
          {container.managed ? ` · ${t("managed")}` : ""}
        </p>
      </div>
      <p className="truncate font-mono text-[12px] text-muted">{container.image}</p>
      <div className="flex flex-col items-start gap-1">
        <StatusBadge
          tone={TONE_BY_STATE[container.state] ?? "neutral"}
          label={t(`state.${container.state}`, { defaultValue: container.state })}
        />
        {action.isError ? <p className="text-[12px] text-danger">{errorMessage(action.error, t)}</p> : null}
      </div>
      <div className="flex gap-1 md:justify-end">
        {container.protected ? (
          <span title={t("protectedHint")} className="px-2 text-[12px] text-subtle">
            {t("protected")}
          </span>
        ) : running ? (
          <ActionButton label={t("actions.stop")} onClick={() => run("stop")} disabled={busy}>
            <StopIcon aria-hidden size={18} />
          </ActionButton>
        ) : (
          <ActionButton label={t("actions.start")} onClick={() => run("start")} disabled={busy}>
            <PlayIcon aria-hidden size={18} />
          </ActionButton>
        )}
        {container.protected || !running ? null : (
          <ActionButton label={t("actions.terminal")} onClick={onTerminal} disabled={false}>
            <TerminalWindowIcon aria-hidden size={18} />
          </ActionButton>
        )}
        {container.protected ? null : (
          <ActionButton label={t("actions.restart")} onClick={() => run("restart")} disabled={busy || !running}>
            <ArrowClockwiseIcon aria-hidden size={18} className={busy ? "animate-spin" : undefined} />
          </ActionButton>
        )}
      </div>
    </li>
  );
}
