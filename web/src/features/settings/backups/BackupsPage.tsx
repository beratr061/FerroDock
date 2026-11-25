import { useState } from "react";
import { ArchiveIcon, DownloadSimpleIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";

import { ApiError } from "@/shared/api/client";
import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, EmptyState, InlineAlert } from "@/shared/ui";

import { backupDownloadUrl, backupsQuery, useCreateBackup, useDeleteBackup, type Backup } from "./api";
import { MasterKeyPanel } from "./MasterKeyPanel";

const RESTORE_COMMAND = [
  "docker compose -f /etc/ferrodock/docker-compose.yml stop ferrodock",
  "docker compose -f /etc/ferrodock/docker-compose.yml run --rm ferrodock restore \\",
  "  /var/lib/ferrodock/backups/<file>.fdbk --master-key /var/lib/ferrodock/restore.key",
  "docker compose -f /etc/ferrodock/docker-compose.yml start ferrodock",
].join("\n");

function BackupRow({ backup }: { backup: Backup }) {
  const { t, i18n } = useTranslation("settings");
  const remove = useDeleteBackup();
  const [confirming, setConfirming] = useState(false);
  const date = new Intl.DateTimeFormat(i18n.language, { dateStyle: "medium", timeStyle: "short" });
  const size = new Intl.NumberFormat(i18n.language, { style: "unit", unit: "kilobyte", maximumFractionDigits: 0 });

  return (
    <li className="flex flex-wrap items-center justify-between gap-3 border-b border-border px-4 py-3 last:border-b-0">
      <div className="min-w-0">
        <p className="truncate font-mono text-[13px] text-text">{backup.name}</p>
        <p className="text-[12px] text-subtle">
          {date.format(new Date(backup.created_at))} · {size.format(backup.size / 1024)}
        </p>
        {remove.isError ? <p className="text-[13px] text-danger">{errorMessage(remove.error, t)}</p> : null}
      </div>
      <div className="flex gap-2">
        <a
          href={backupDownloadUrl(backup.name)}
          download
          className="press inline-flex h-9 items-center gap-2 rounded-control border border-border-strong bg-surface px-3.5 font-medium text-text hover:bg-surface-hover"
        >
          <DownloadSimpleIcon size={16} aria-hidden />
          {t("backups.download")}
        </a>
        {confirming ? (
          <Button variant="danger" busy={remove.isPending} onClick={() => remove.mutate(backup.name)}>
            {t("backups.confirmDelete")}
          </Button>
        ) : (
          <Button variant="ghost" onClick={() => setConfirming(true)}>
            {t("backups.delete")}
          </Button>
        )}
      </div>
    </li>
  );
}

function useBackups() {
  const create = useCreateBackup();
  const [waitingSince, setWaitingSince] = useState<number | null>(null);
  const backups = useQuery(backupsQuery(waitingSince !== null));
  const newest = backups.data?.[0] ? new Date(backups.data[0].created_at).getTime() : 0;
  if (waitingSince !== null && newest >= waitingSince) {
    setWaitingSince(null);
  }
  const start = (): void => create.mutate(undefined, { onSuccess: () => setWaitingSince(Date.now() - 1_000) });
  return { create, backups, start, waiting: waitingSince !== null };
}

function BackupList({ list }: { list: Backup[] | undefined }) {
  const { t } = useTranslation("settings");
  if (!list) {
    return null;
  }
  if (list.length === 0) {
    return (
      <EmptyState icon={<ArchiveIcon size={22} aria-hidden />} title={t("backups.empty.title")} description={t("backups.empty.description")} />
    );
  }
  return (
    <ul className="rounded-panel border border-border bg-surface">
      {list.map((backup) => (
        <BackupRow key={backup.name} backup={backup} />
      ))}
    </ul>
  );
}

function RestoreHelp() {
  const { t } = useTranslation("settings");
  return (
    <section className="flex flex-col gap-2">
      <h3 className="font-medium text-text">{t("backups.restore.title")}</h3>
      <p className="max-w-2xl text-muted">{t("backups.restore.description")}</p>
      <pre className="overflow-x-auto rounded-control border border-border bg-surface-raised p-3 font-mono text-[12px] text-text">
        {RESTORE_COMMAND}
      </pre>
    </section>
  );
}

export function BackupsPage() {
  const { t } = useTranslation("settings");
  const { create, backups, start, waiting } = useBackups();
  const forbidden = backups.error instanceof ApiError && backups.error.status === 403;
  const failure = create.error ?? (forbidden ? null : backups.error);

  return (
    <section className="flex flex-col gap-5">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex max-w-xl flex-col gap-1">
          <h2 className="text-lg text-text">{t("backups.title")}</h2>
          <p className="text-muted">{t("backups.description")}</p>
        </div>
        {forbidden ? null : (
          <Button icon={<ArchiveIcon size={16} aria-hidden />} busy={create.isPending || waiting} onClick={start}>
            {t("backups.create")}
          </Button>
        )}
      </header>
      {forbidden ? <InlineAlert tone="info">{t("backups.ownerOnly")}</InlineAlert> : null}
      {failure ? <InlineAlert tone="danger">{errorMessage(failure, t)}</InlineAlert> : null}
      {backups.isPending ? <div aria-hidden className="skeleton h-16" /> : null}
      <BackupList list={backups.data} />
      {forbidden ? null : <MasterKeyPanel />}
      {forbidden ? null : <RestoreHelp />}
    </section>
  );
}
