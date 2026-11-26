import { ClipboardTextIcon } from "@phosphor-icons/react";

import { useTranslation } from "../i18n";

import { sendToFocused } from "./store";

const KEYS: { label: string; sequence: string }[] = [
  { label: "Esc", sequence: "\u001b" },
  { label: "Tab", sequence: "\t" },
  { label: "Ctrl+C", sequence: "\u0003" },
  { label: "Ctrl+D", sequence: "\u0004" },
  { label: "Ctrl+L", sequence: "\u000c" },
  { label: "↑", sequence: "\u001b[A" },
  { label: "↓", sequence: "\u001b[B" },
  { label: "←", sequence: "\u001b[D" },
  { label: "→", sequence: "\u001b[C" },
];

const KEY = "press h-8 shrink-0 rounded-control border border-border bg-surface-raised px-2.5 font-mono text-[12px] text-muted hover:text-text";

async function paste(): Promise<void> {
  try {
    const text = await navigator.clipboard.readText();
    if (text) {
      sendToFocused(text);
    }
  } catch {
    return;
  }
}

export function KeyBar() {
  const { t } = useTranslation("common");
  return (
    <div role="toolbar" aria-label={t("terminal.keys")} className="flex gap-1 overflow-x-auto pb-1">
      {KEYS.map((key) => (
        <button
          key={key.label}
          type="button"
          className={KEY}
          onPointerDown={(event) => event.preventDefault()}
          onClick={() => sendToFocused(key.sequence)}
        >
          {key.label}
        </button>
      ))}
      <button type="button" className={`${KEY} inline-flex items-center gap-1.5`} onClick={() => void paste()}>
        <ClipboardTextIcon size={14} aria-hidden />
        {t("terminal.paste")}
      </button>
    </div>
  );
}
