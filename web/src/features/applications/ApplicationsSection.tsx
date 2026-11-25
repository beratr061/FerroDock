import { useState } from "react";
import { CaretRightIcon, GitBranchIcon, PackageIcon, PlusIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";

import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert, SegmentedControl } from "@/shared/ui";

import { applicationsQuery, type Application, type Environment } from "./api";
import { CreateApplicationForm } from "./CreateApplicationForm";
import { sourceSummary } from "./sourceForm";

function ApplicationRow({ application }: { application: Application }) {
  const Icon = application.source.kind === "image" ? PackageIcon : GitBranchIcon;
  return (
    <li className="border-b border-border last:border-b-0">
      <Link
        to="/applications/$applicationId"
        params={{ applicationId: application.id }}
        className="group flex items-center gap-3 px-4 py-3 hover:bg-surface-hover"
      >
        <Icon size={18} aria-hidden className="shrink-0 text-muted" />
        <div className="min-w-0 flex-1">
          <p className="truncate font-medium text-text">{application.name}</p>
          <p className="truncate font-mono text-[12px] text-subtle">{sourceSummary(application)}</p>
        </div>
        <CaretRightIcon size={16} aria-hidden className="text-subtle group-hover:text-text" />
      </Link>
    </li>
  );
}

function ApplicationList({ query, visible, creating }: {
  query: ReturnType<typeof useApplications>;
  visible: Application[];
  creating: boolean;
}) {
  const { t } = useTranslation("applications");
  if (query.isPending) {
    return <div aria-hidden className="skeleton m-4 h-10" />;
  }
  if (visible.length === 0) {
    return creating ? null : <p className="px-4 py-6 text-muted">{t("empty")}</p>;
  }
  return (
    <ul>
      {visible.map((application) => (
        <ApplicationRow key={application.id} application={application} />
      ))}
    </ul>
  );
}

function useApplications(projectId: string) {
  return useQuery(applicationsQuery(projectId));
}

function pickEnvironment(environments: Environment[], selected: string): string {
  return environments.some((env) => env.id === selected) ? selected : (environments[0]?.id ?? "");
}

interface ApplicationsSectionProps {
  projectId: string;
  environments: Environment[];
}

export function ApplicationsSection({ projectId, environments }: ApplicationsSectionProps) {
  const { t } = useTranslation("applications");
  const applications = useApplications(projectId);
  const [selected, setSelected] = useState(environments[0]?.id ?? "");
  const [creating, setCreating] = useState(false);
  const environmentId = pickEnvironment(environments, selected);
  const visible = applications.data?.filter((app) => app.environment_id === environmentId) ?? [];

  return (
    <section className="flex flex-col gap-3">
      <header className="flex flex-wrap items-end justify-between gap-3">
        <SegmentedControl
          label={t("title")}
          value={environmentId}
          segments={environments.map((env) => ({ value: env.id, label: env.name }))}
          onChange={setSelected}
        />
        {creating ? null : (
          <Button icon={<PlusIcon size={16} weight="bold" aria-hidden />} onClick={() => setCreating(true)}>
            {t("new")}
          </Button>
        )}
      </header>
      {applications.isError ? <InlineAlert tone="danger">{errorMessage(applications.error, t)}</InlineAlert> : null}
      <div className="overflow-hidden rounded-panel border border-border bg-surface">
        {creating ? (
          <CreateApplicationForm key={environmentId} environmentId={environmentId} onCancel={() => setCreating(false)} />
        ) : null}
        <ApplicationList query={applications} visible={visible} creating={creating} />
      </div>
    </section>
  );
}
