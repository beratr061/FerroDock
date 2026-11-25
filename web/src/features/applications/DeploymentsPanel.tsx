import { useState } from "react";
import { RocketLaunchIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert } from "@/shared/ui";

import type { Application } from "./api";
import { DeploymentLog } from "./DeploymentLog";
import { DeploymentRow } from "./DeploymentRow";
import { deploymentsQuery, useDeploy, type Deployment } from "./deployments";

function rollbackTargets(deployments: Deployment[], application: Application): Set<string> {
  if (application.source.kind === "git" && application.source.build_kind === "compose") {
    return new Set();
  }
  const succeeded = deployments.filter((deployment) => deployment.status === "succeeded");
  return new Set(succeeded.slice(1).map((deployment) => deployment.id));
}

interface HistoryProps {
  list: Deployment[];
  selected: string | null;
  targets: Set<string>;
  onSelect: (id: string) => void;
}

function DeploymentHistory({ list, selected, targets, onSelect }: HistoryProps) {
  const liveId = list.find((deployment) => deployment.status === "succeeded")?.id;
  return (
    <div className="grid gap-3 lg:grid-cols-[minmax(0,2fr)_minmax(0,3fr)]">
      <ul className="max-h-80 overflow-auto rounded-panel border border-border bg-surface">
        {list.map((deployment) => (
          <DeploymentRow
            key={deployment.id}
            deployment={deployment}
            live={deployment.id === liveId}
            selected={deployment.id === selected}
            canRollback={targets.has(deployment.id)}
            onSelect={() => onSelect(deployment.id)}
          />
        ))}
      </ul>
      {selected ? <DeploymentLog key={selected} deploymentId={selected} /> : null}
    </div>
  );
}

export function DeploymentsPanel({ application }: { application: Application }) {
  const { t } = useTranslation("applications");
  const deployments = useQuery(deploymentsQuery(application.id));
  const deploy = useDeploy(application.id);
  const [chosen, setChosen] = useState<string | null>(null);
  const list = deployments.data ?? [];
  const selected = chosen ?? list[0]?.id ?? null;
  const targets = rollbackTargets(list, application);

  const onDeploy = (): void => deploy.mutate(undefined, { onSuccess: (created) => setChosen(created.id) });

  return (
    <section className="flex flex-col gap-3">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-col gap-1">
          <h2 className="text-lg text-text">{t("deployments.title")}</h2>
          <p className="text-muted">{t("deployments.description")}</p>
        </div>
        <Button icon={<RocketLaunchIcon size={16} aria-hidden />} busy={deploy.isPending} onClick={onDeploy}>
          {t("deployments.deploy")}
        </Button>
      </header>
      {deploy.isError ? <InlineAlert tone="danger">{errorMessage(deploy.error, t)}</InlineAlert> : null}
      {deployments.isError ? <InlineAlert tone="danger">{errorMessage(deployments.error, t)}</InlineAlert> : null}
      {deployments.isPending ? <div aria-hidden className="skeleton h-16" /> : null}
      {deployments.data && list.length === 0 ? (
        <p className="rounded-panel border border-border bg-surface px-4 py-6 text-muted">{t("deployments.empty")}</p>
      ) : null}
      {list.length > 0 ? (
        <DeploymentHistory list={list} selected={selected} targets={targets} onSelect={setChosen} />
      ) : null}
    </section>
  );
}
