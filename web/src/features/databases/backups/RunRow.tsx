import { useState } from "react";
import { ArrowCounterClockwiseIcon, TrashIcon } from "@phosphor-icons/react";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, StatusBadge, type StatusTone } from "@/shared/ui";

import { useDeleteRun, useRestore, type BackupRun } from "./api";

const TONES: Record<string, StatusTone> = {
  queued: "neutral",
  running: "info",
  succeeded: "success",
  failed: "danger",
};

function useFormatters() {
  const { i18n } = useTranslation("databases");
  return {
    date: new Intl.DateTimeFormat(i18n.language, { dateStyle: "medium", timeStyle: "short" }),
    size: new Intl.NumberFormat(i18n.language, { style: "unit", unit: "kilobyte", maximumFractionDigits: 0 }),
  };
}

export function RunRow({ databaseId, run }: { databaseId: string; run: BackupRun }) {
  const { t } = useTranslation("databases");
  const restore = useRestore(databaseId);
  const remove = useDeleteRun(databaseId);
  const [confirming, setConfirming] = useState(false);
  const { date, size } = useFormatters();
  const restorable = run.operation === "backup" && run.status === "succeeded";
  const failure = restore.error ?? remove.error;
  const reason = run.error_code ? t(`errors.${run.error_code}`, { defaultValue: run.error_code }) : null;

  return (
    <li className="flex flex-wrap items-center justify-between gap-3 border-b border-border px-4 py-3 last:border-b-0">
      <div className="min-w-0">
        <div className="flex flex-wrap items-center gap-2">
          <p className="font-medium text-text">{t(`backups.operation.${run.operation}`)}</p>
          <StatusBadge tone={TONES[run.status] ?? "neutral"} label={t(`backups.status.${run.status}`)} />
          <span className="text-[12px] text-subtle">{t(`backups.trigger.${run.trigger}`)}</span>
        </div>
        <p className="text-[12px] text-subtle">
          {date.format(new Date(run.created_at))}
          {run.size_bytes ? ` · ${size.format(run.size_bytes / 1024)}` : ""}
        </p>
        {reason ? <p className="text-[13px] text-danger">{reason}</p> : null}
        {failure ? <p className="text-[13px] text-danger">{errorMessage(failure, t)}</p> : null}
      </div>
      {restorable ? (
        <div className="flex flex-wrap gap-2">
          {confirming ? (
            <>
              <Button variant="ghost" onClick={() => setConfirming(false)}>
                {t("actions.cancel", { ns: "common" })}
              </Button>
              <Button variant="danger" busy={restore.isPending} onClick={() => restore.mutate(run.id, { onSuccess: () => setConfirming(false) })}>
                {t("backups.confirmRestore")}
              </Button>
            </>
          ) : (
            <Button variant="secondary" icon={<ArrowCounterClockwiseIcon size={16} aria-hidden />} onClick={() => setConfirming(true)}>
              {t("backups.restore")}
            </Button>
          )}
          <Button variant="ghost" aria-label={t("backups.delete")} busy={remove.isPending} icon={<TrashIcon size={16} aria-hidden />} onClick={() => remove.mutate(run.id)} />
        </div>
      ) : null}
    </li>
  );
}
