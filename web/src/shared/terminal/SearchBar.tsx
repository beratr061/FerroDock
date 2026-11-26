import { useState } from "react";
import { CaretDownIcon, CaretUpIcon, XIcon } from "@phosphor-icons/react";

import { useTranslation } from "../i18n";

interface SearchBarProps {
  onNext: (query: string) => boolean;
  onPrevious: (query: string) => boolean;
  onClose: () => void;
}

const ICON_BUTTON = "press grid size-7 place-items-center rounded-control text-muted hover:bg-surface-hover hover:text-text";

export function SearchBar({ onNext, onPrevious, onClose }: SearchBarProps) {
  const { t } = useTranslation("common");
  const [query, setQuery] = useState("");
  const [missed, setMissed] = useState(false);
  const run = (backwards: boolean): void => {
    if (query) {
      setMissed(!(backwards ? onPrevious(query) : onNext(query)));
    }
  };

  return (
    <div className="flex items-center gap-1">
      <input
        type="search"
        autoFocus
        value={query}
        aria-label={t("terminal.search")}
        placeholder={t("terminal.search")}
        onChange={(event) => setQuery(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            run(event.shiftKey);
          } else if (event.key === "Escape") {
            onClose();
          }
        }}
        className="h-7 w-40 rounded-control border border-border-strong bg-bg px-2 font-mono text-[12px] text-text"
      />
      {missed ? <span className="text-[12px] text-subtle">{t("terminal.noMatch")}</span> : null}
      <button type="button" className={ICON_BUTTON} title={t("terminal.previous")} onClick={() => run(true)}>
        <CaretUpIcon size={14} aria-hidden />
      </button>
      <button type="button" className={ICON_BUTTON} title={t("terminal.next")} onClick={() => run(false)}>
        <CaretDownIcon size={14} aria-hidden />
      </button>
      <button type="button" className={ICON_BUTTON} title={t("terminal.closeSearch")} onClick={onClose}>
        <XIcon size={14} aria-hidden />
      </button>
    </div>
  );
}
