import { useId, useState, type KeyboardEvent } from "react";
import type { Icon } from "@phosphor-icons/react";
import { DatabaseIcon, FolderIcon, MagnifyingGlassIcon, PackageIcon, SquaresFourIcon } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";

import { useTranslation } from "@/shared/i18n";
import { cx } from "@/shared/lib/cx";

import { matches, usePaletteItems, type PaletteItem, type PaletteKind } from "./items";
import type { PaletteSources } from "./PaletteHost";

const ICONS: Record<PaletteKind, Icon> = {
  page: SquaresFourIcon,
  project: FolderIcon,
  application: PackageIcon,
  database: DatabaseIcon,
};
const MAX_RESULTS = 40;

function Result({ item, active, id, onPick }: { item: PaletteItem; active: boolean; id: string; onPick: () => void }) {
  const { t } = useTranslation("common");
  const ItemIcon = ICONS[item.kind];
  return (
    <li
      id={id}
      role="option"
      aria-selected={active}
      onPointerDown={(event) => event.preventDefault()}
      onClick={onPick}
      className={cx("flex cursor-pointer items-center gap-3 rounded-control px-3 py-2", active ? "bg-surface-hover text-text" : "text-muted")}
    >
      <ItemIcon size={16} aria-hidden className="shrink-0" />
      <span className="min-w-0 flex-1 truncate text-text">{item.label}</span>
      <span className="truncate font-mono text-[12px] text-subtle">{item.detail}</span>
      <span className="sr-only">{t(`palette.kinds.${item.kind}`)}</span>
    </li>
  );
}

export default function CommandPalette({ sources, onClose }: { sources: PaletteSources; onClose: () => void }) {
  const { t } = useTranslation("common");
  const navigate = useNavigate();
  const listId = useId();
  const items = usePaletteItems(sources, t);
  const [query, setQuery] = useState("");
  const [cursor, setCursor] = useState(0);
  const results = items.filter((item) => matches(item, query)).slice(0, MAX_RESULTS);
  const active = Math.min(cursor, Math.max(0, results.length - 1));

  const pick = (item: PaletteItem | undefined): void => {
    if (item) {
      onClose();
      void navigate({ to: item.to });
    }
  };

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>): void => {
    const moves: Record<string, number> = { ArrowDown: 1, ArrowUp: -1 };
    const move = moves[event.key];
    if (move !== undefined && results.length > 0) {
      event.preventDefault();
      setCursor((active + move + results.length) % results.length);
    } else if (event.key === "Enter") {
      event.preventDefault();
      pick(results[active]);
    } else if (event.key === "Escape") {
      onClose();
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center px-4 pt-[12dvh]" onPointerDown={onClose}>
      <div
        role="dialog"
        aria-modal="true"
        aria-label={t("palette.label")}
        onPointerDown={(event) => event.stopPropagation()}
        className="palette-enter flex max-h-[70dvh] w-full max-w-xl flex-col overflow-hidden rounded-panel border border-border-strong bg-surface shadow-overlay"
      >
        <div className="flex items-center gap-2 border-b border-border px-3">
          <MagnifyingGlassIcon size={16} aria-hidden className="text-subtle" />
          <input
            autoFocus
            role="combobox"
            aria-expanded="true"
            aria-controls={listId}
            aria-activedescendant={results[active] ? `${listId}-${active}` : undefined}
            value={query}
            placeholder={t("palette.placeholder")}
            onChange={(event) => {
              setQuery(event.target.value);
              setCursor(0);
            }}
            onKeyDown={onKeyDown}
            className="h-12 min-w-0 flex-1 bg-transparent text-text outline-none placeholder:text-subtle"
          />
          <kbd className="rounded-control border border-border px-1.5 font-mono text-[11px] text-subtle">Esc</kbd>
        </div>
        <ul id={listId} role="listbox" aria-label={t("palette.results")} className="overflow-y-auto p-2">
          {results.map((item, index) => (
            <Result key={`${item.kind}-${item.id}`} id={`${listId}-${index}`} item={item} active={index === active} onPick={() => pick(item)} />
          ))}
          {results.length === 0 ? <li className="px-3 py-6 text-center text-muted">{t("palette.empty")}</li> : null}
        </ul>
      </div>
    </div>
  );
}
