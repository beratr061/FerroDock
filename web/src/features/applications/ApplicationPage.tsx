import { ArrowLeftIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { Link, useParams, useRouterState } from "@tanstack/react-router";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { InlineAlert } from "@/shared/ui";

import { applicationQuery, type Application } from "./api";
import { ApplicationSettings } from "./ApplicationSettings";
import { ApplicationTabs } from "./ApplicationTabs";
import { ComposeTrustPanel } from "./ComposeTrustPanel";
import { WebhookPanel } from "./WebhookPanel";
import { ContainerTerminal } from "./ContainerTerminal";
import { DeploymentsPanel } from "./deployments/DeploymentsPanel";
import { DeployKeyPanel } from "./DeployKeyPanel";
import { DomainsPanel } from "./domains/DomainsPanel";
import { LogsPanel } from "./logs/LogsPanel";
import { RuntimePanel } from "./runtime/RuntimePanel";
import { VolumeBackups } from "./runtime/VolumeBackups";
import { isCompose, sourceSummary } from "./sourceForm";
import { applicationTab, type ApplicationTab } from "./tabs";
import { VariablesPanel } from "./variables/VariablesPanel";

function TabContent({ tab, application }: { tab: ApplicationTab; application: Application }) {
  switch (tab) {
    case "deployments":
      return <DeploymentsPanel application={application} />;
    case "logs":
      return <LogsPanel applicationId={application.id} />;
    case "environment":
      return <VariablesPanel applicationId={application.id} />;
    case "domains":
      return <DomainsPanel applicationId={application.id} compose={isCompose(application)} />;
    case "storage":
      return (
        <>
          <RuntimePanel application={application} />
          <VolumeBackups application={application} />
        </>
      );
    case "settings":
      return (
        <>
          <ApplicationSettings application={application} />
          {application.source.kind === "git" ? <WebhookPanel application={application} /> : null}
          {isCompose(application) ? <ComposeTrustPanel application={application} /> : null}
          <DeployKeyPanel application={application} />
        </>
      );
  }
}

function Header({ application }: { application: Application }) {
  return (
    <header className="flex flex-col gap-1">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h1 className="text-xl text-text">{application.name}</h1>
        <ContainerTerminal applicationId={application.id} />
      </div>
      <p className="truncate font-mono text-[12px] text-subtle">{sourceSummary(application)}</p>
      {application.description ? <p className="max-w-2xl text-muted">{application.description}</p> : null}
    </header>
  );
}

export function ApplicationPage() {
  const { t } = useTranslation("applications");
  const { applicationId } = useParams({ from: "/app/applications/$applicationId" });
  const tabParam = useRouterState({ select: (state) => state.location.search.tab });
  const application = useQuery(applicationQuery(applicationId));
  const data = application.data;
  const tab = applicationTab(tabParam);

  return (
    <div className="flex flex-col gap-6">
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
          <Header application={data} />
          <ApplicationTabs applicationId={data.id} active={tab} />
          <div role="tabpanel" aria-label={t(`tabs.${tab}`)} className="flex flex-col gap-8">
            <TabContent tab={tab} application={data} />
          </div>
        </>
      ) : null}
    </div>
  );
}
