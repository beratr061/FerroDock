import { ArrowLeftIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { Link, useParams } from "@tanstack/react-router";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { InlineAlert } from "@/shared/ui";

import { applicationQuery } from "./api";
import { ApplicationSettings } from "./ApplicationSettings";
import { ContainerTerminal } from "./ContainerTerminal";
import { DeploymentsPanel } from "./deployments/DeploymentsPanel";
import { DeployKeyPanel } from "./DeployKeyPanel";
import { DomainsPanel } from "./domains/DomainsPanel";
import { VariablesPanel } from "./variables/VariablesPanel";
import { sourceSummary } from "./sourceForm";

export function ApplicationPage() {
  const { t } = useTranslation("applications");
  const { applicationId } = useParams({ from: "/app/applications/$applicationId" });
  const application = useQuery(applicationQuery(applicationId));
  const data = application.data;

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
      {application.isPending ? <div aria-hidden className="skeleton h-24" /> : null}
      {application.isError ? <InlineAlert tone="danger">{errorMessage(application.error, t)}</InlineAlert> : null}
      {data ? (
        <>
          <header className="flex flex-col gap-1">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <h1 className="text-xl text-text">{data.name}</h1>
              <ContainerTerminal applicationId={data.id} />
            </div>
            <p className="truncate font-mono text-[12px] text-subtle">{sourceSummary(data)}</p>
            {data.description ? <p className="max-w-2xl text-muted">{data.description}</p> : null}
          </header>
          <DeploymentsPanel application={data} />
          <VariablesPanel applicationId={data.id} />
          <DomainsPanel applicationId={data.id} />
          <DeployKeyPanel application={data} />
          <ApplicationSettings application={data} />
        </>
      ) : null}
    </div>
  );
}
