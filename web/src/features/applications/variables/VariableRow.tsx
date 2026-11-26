import { useState } from "react";
import { EyeIcon, LockKeyIcon, TrashIcon } from "@phosphor-icons/react";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";

import { revealVariable } from "./api";
import type { DraftRow } from "./useVariableDraft";

interface VariableRowProps {
  applicationId: string;
  row: DraftRow;
  onChange: (change: Partial<DraftRow>) => void;
  onRemove: () => void;
}

const INPUT =
  "h-9 w-full min-w-0 rounded-control border border-border-strong bg-surface px-2.5 font-mono text-[13px] text-text caret-primary placeholder:text-subtle hover:border-subtle focus:border-focus";
const ICON_BUTTON =
  "press grid size-9 shrink-0 place-items-center rounded-control text-muted hover:bg-surface-hover hover:text-text";

export function VariableRow({ applicationId, row, onChange, onRemove }: VariableRowProps) {
  const { t } = useTranslation("applications");
  const [error, setError] = useState<string | null>(null);

  const reveal = async (): Promise<void> => {
    try {
      onChange({ value: await revealVariable(applicationId, row.key) });
      setError(null);
    } catch (failure) {
      setError(errorMessage(failure, t));
    }
  };

  return (
    <li className="grid gap-2 border-b border-border px-3 py-2.5 last:border-b-0 md:grid-cols-[minmax(0,2fr)_minmax(0,3fr)_auto] md:items-center">
      <input
        aria-label={t("variables.key")}
        value={row.key}
        placeholder="DATABASE_URL"
        spellCheck={false}
        onChange={(event) => onChange({ key: event.target.value })}
        className={INPUT}
      />
      {row.value === null ? (
        <div className="flex items-center gap-2">
          <span className="truncate font-mono text-[13px] text-subtle">{t("variables.hidden")}</span>
          <button type="button" className="press text-[13px] text-muted underline hover:text-text" onClick={() => void reveal()}>
            <EyeIcon size={14} aria-hidden className="mr-1 inline" />
            {t("variables.reveal")}
          </button>
          <button type="button" className="press text-[13px] text-muted underline hover:text-text" onClick={() => onChange({ value: "" })}>
            {t("variables.replace")}
          </button>
        </div>
      ) : (
        <input
          aria-label={t("variables.value")}
          type={row.secret ? "password" : "text"}
          value={row.value}
          spellCheck={false}
          autoComplete="off"
          onChange={(event) => onChange({ value: event.target.value })}
          className={INPUT}
        />
      )}
      <div className="flex gap-1">
        <button
          type="button"
          aria-pressed={row.secret}
          title={t("variables.secret")}
          onClick={() => onChange({ secret: !row.secret })}
          className={`${ICON_BUTTON} aria-pressed:bg-surface-raised aria-pressed:text-text`}
        >
          <LockKeyIcon size={16} aria-hidden weight={row.secret ? "fill" : "regular"} />
          <span className="sr-only">{t("variables.secret")}</span>
        </button>
        <button type="button" title={t("variables.remove")} onClick={onRemove} className={ICON_BUTTON}>
          <TrashIcon size={16} aria-hidden />
          <span className="sr-only">{t("variables.remove")}</span>
        </button>
      </div>
      {error ? <p className="text-[13px] text-danger md:col-span-3">{error}</p> : null}
    </li>
  );
}
