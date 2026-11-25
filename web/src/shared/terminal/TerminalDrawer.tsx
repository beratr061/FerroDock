import { lazy, Suspense, useEffect } from "react";
import { TerminalWindowIcon, XIcon } from "@phosphor-icons/react";
import { useTranslation } from "../i18n";

const TerminalView = lazy(() => import("./TerminalView"));

interface TerminalDrawerProps {
  container: string;
  title: string;
  onClose: () => void;
}

export function TerminalDrawer({ container, title, onClose }: TerminalDrawerProps) {
  const { t } = useTranslation("common");

  useEffect(() => {
    const onKey = (event: KeyboardEvent): void => {
      if (event.key === "Escape" && event.ctrlKey) {
        onClose();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    <section
      aria-label={t("terminal.label", { name: title })}
      className="terminal-drawer fixed inset-x-0 bottom-0 z-40 flex h-[min(60dvh,560px)] flex-col gap-2 border-t border-border-strong bg-surface px-3 pt-2 pb-3"
    >
      <header className="flex items-center justify-between gap-2">
        <p className="flex min-w-0 items-center gap-2 font-mono text-[13px] text-text">
          <TerminalWindowIcon size={16} aria-hidden />
          <span className="truncate">{title}</span>
        </p>
        <button
          type="button"
          onClick={onClose}
          title={t("terminal.close")}
          className="press grid size-8 place-items-center rounded-control text-muted hover:bg-surface-hover hover:text-text"
        >
          <XIcon size={16} aria-hidden />
          <span className="sr-only">{t("terminal.close")}</span>
        </button>
      </header>
      <Suspense fallback={<div aria-hidden className="skeleton flex-1" />}>
        <TerminalView key={container} container={container} />
      </Suspense>
    </section>
  );
}
