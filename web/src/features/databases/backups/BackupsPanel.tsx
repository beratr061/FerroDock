import { ArchiveIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert } from "@/shared/ui";

import { destinationsQuery, policyQuery, runsQuery, useBackUpNow, type BackupRun } from "./api";
import { PolicyForm } from "./PolicyForm";
import { RunRow } from "./RunRow";

function NoDestinations() {
  const { t } = useTranslation("databases");
  return (
    <div className="flex flex-wrap items-center justify-between gap-3 rounded-panel border border-border bg-surface px-4 py-4">
      <p className="text-muted">{t("backups.noDestinations")}</p>
      <Link to="/settings/storage" className="press font-medium text-text underline underline-offset-4">
        {t("backups.addDestination")}
      </Link>
    </div>
  );
}

function RunList({ databaseId, runs }: { databaseId: string; runs: BackupRun[] | undefined }) {
  if (!runs || runs.length === 0) {
    return null;
  }
  return (
    <ul className="overflow-hidden rounded-panel border border-border bg-surface">
      {runs.map((run) => (
        <RunRow key={run.id} databaseId={databaseId} run={run} />
      ))}
    </ul>
  );
}

export function BackupsPanel({ databaseId }: { databaseId: string }) {
  const { t } = useTranslation("databases");
  const destinations = useQuery(destinationsQuery);
  const policy = useQuery(policyQuery(databaseId));
  const runs = useQuery(runsQuery(databaseId));
  const backUp = useBackUpNow(databaseId);
  const options = (destinations.data ?? []).map((item) => ({ value: item.id, label: item.name }));
  const target = policy.data?.destination_id ?? options[0]?.value;

  return (
    <section className="flex flex-col gap-3">
      <header className="flex flex-wrap items-end justify-between gap-3">
        <div className="flex flex-col gap-1">
          <h2 className="flex items-center gap-2 text-lg text-text">
            <ArchiveIcon size={18} aria-hidden />
            {t("backups.title")}
          </h2>
          <p className="max-w-2xl text-muted">{t("backups.description")}</p>
        </div>
        {target ? (
          <Button variant="secondary" busy={backUp.isPending} onClick={() => backUp.mutate(target)}>
            {t("backups.now")}
          </Button>
        ) : null}
      </header>
      {backUp.isError ? <InlineAlert tone="danger">{errorMessage(backUp.error, t)}</InlineAlert> : null}
      {destinations.isSuccess && options.length === 0 ? <NoDestinations /> : null}
      {options.length > 0 && policy.isSuccess ? (
        <div className="rounded-panel border border-border bg-surface">
          <PolicyForm key={policy.dataUpdatedAt} databaseId={databaseId} policy={policy.data ?? null} destinations={options} />
        </div>
      ) : null}
      <RunList databaseId={databaseId} runs={runs.data} />
    </section>
  );
}
