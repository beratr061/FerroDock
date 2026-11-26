import { ArrowClockwiseIcon, ArrowLeftIcon, PlayIcon, StopIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { Link, useParams } from "@tanstack/react-router";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert, StatusBadge } from "@/shared/ui";

import { ActivityPanel } from "./ActivityPanel";
import { databaseQuery, useDatabaseAction, type Database } from "./api";
import { BackupsPanel } from "./backups/BackupsPanel";
import { ConnectionPanel } from "./ConnectionPanel";
import { DeleteDatabase } from "./DeleteDatabase";
import { ExternalPortPanel } from "./ExternalPortPanel";
import { engineSummary, statusKey, statusTone } from "./presentation";

function Actions({ database }: { database: Database }) {
  const { t } = useTranslation("databases");
  const action = useDatabaseAction(database.id);
  const deploying = database.status === "deploying";
  const pending = (name: string): boolean => action.isPending && action.variables === name;

  return (
    <div className="flex flex-col items-end gap-2">
      <div className="flex flex-wrap gap-2">
        {database.running ? (
          <Button variant="secondary" busy={pending("stop")} disabled={deploying} icon={<StopIcon size={16} aria-hidden />} onClick={() => action.mutate("stop")}>
            {t("actions.stop")}
          </Button>
        ) : (
          <Button variant="secondary" busy={pending("start")} disabled={deploying} icon={<PlayIcon size={16} aria-hidden />} onClick={() => action.mutate("start")}>
            {t("actions.start")}
          </Button>
        )}
        <Button busy={pending("deploy") || deploying} icon={<ArrowClockwiseIcon size={16} aria-hidden />} onClick={() => action.mutate("deploy")}>
          {t("actions.redeploy")}
        </Button>
      </div>
      {action.isError ? <p className="text-[13px] text-danger">{errorMessage(action.error, t)}</p> : null}
    </div>
  );
}

function Header({ database }: { database: Database }) {
  const { t } = useTranslation("databases");
  return (
    <header className="flex flex-wrap items-start justify-between gap-3">
      <div className="flex min-w-0 flex-col gap-1">
        <div className="flex flex-wrap items-center gap-3">
          <h1 className="text-xl text-text">{database.name}</h1>
          <StatusBadge tone={statusTone(database)} label={t(statusKey(database))} live={database.running} />
        </div>
        <p className="truncate font-mono text-[12px] text-subtle">{engineSummary(database)}</p>
      </div>
      <Actions database={database} />
    </header>
  );
}

export function DatabasePage() {
  const { t } = useTranslation("databases");
  const { databaseId } = useParams({ from: "/app/databases/$databaseId" });
  const database = useQuery(databaseQuery(databaseId));
  const data = database.data;

  return (
    <div className="flex flex-col gap-8">
      {data ? (
        <Link
          to="/projects/$projectId"
          params={{ projectId: data.project_id }}
          className="press inline-flex w-fit items-center gap-1.5 rounded-control text-muted hover:text-text"
        >
          <ArrowLeftIcon size={16} aria-hidden />
          {t("back")}
        </Link>
      ) : null}
      {database.isPending ? <div aria-hidden className="skeleton h-24" /> : null}
      {database.isError ? <InlineAlert tone="danger">{errorMessage(database.error, t)}</InlineAlert> : null}
      {data ? (
        <>
          <Header database={data} />
          {data.status === "failed" ? (
            <InlineAlert tone="danger">
              {t("failed", { reason: t(`errors.${data.error_code ?? "UNKNOWN"}`, { defaultValue: data.error_code ?? "" }) })}
            </InlineAlert>
          ) : null}
          <ConnectionPanel database={data} />
          <ActivityPanel database={data} />
          <BackupsPanel databaseId={data.id} />
          <ExternalPortPanel key={data.external_port ?? "none"} database={data} />
          <DeleteDatabase database={data} />
        </>
      ) : null}
    </div>
  );
}
