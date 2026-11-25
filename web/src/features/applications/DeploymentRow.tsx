import { useTranslation } from "react-i18next";

import { errorMessage } from "@/shared/lib/errors";
import { Button, StatusBadge, type StatusTone } from "@/shared/ui";

import { isActive, useCancelDeployment, useRollback, type Deployment, type DeploymentStatus } from "./deployments";

const TONES: Record<DeploymentStatus, StatusTone> = {
  queued: "info",
  running: "warning",
  succeeded: "success",
  failed: "danger",
  cancelled: "neutral",
};

function revision(deployment: Deployment): string {
  if (deployment.commit_sha) {
    return deployment.commit_sha.slice(0, 7);
  }
  return deployment.image_ref ?? "";
}

interface DeploymentRowProps {
  deployment: Deployment;
  live: boolean;
  selected: boolean;
  canRollback: boolean;
  onSelect: () => void;
}

export function DeploymentRow({ deployment, live, selected, canRollback, onSelect }: DeploymentRowProps) {
  const { t, i18n } = useTranslation("applications");
  const rollback = useRollback(deployment.application_id);
  const cancel = useCancelDeployment(deployment.application_id);
  const created = new Intl.DateTimeFormat(i18n.language, { dateStyle: "medium", timeStyle: "short" });
  const failure = rollback.error ?? cancel.error;

  return (
    <li
      aria-current={selected ? "true" : undefined}
      className="flex flex-wrap items-center gap-x-4 gap-y-2 border-b border-border px-4 py-3 last:border-b-0 aria-[current=true]:bg-surface-raised"
    >
      <button type="button" onClick={onSelect} className="flex min-w-0 flex-1 items-center gap-3 text-left">
        <span className="w-24 shrink-0">
          <StatusBadge
            tone={TONES[deployment.status]}
            label={live ? t("deployments.status.live") : t(`deployments.status.${deployment.status}`)}
            live={live || isActive(deployment.status)}
          />
        </span>
        <span className="min-w-0">
          <span className="block truncate text-text">
            {t(`deployments.kind.${deployment.kind}`)}
            <span className="ml-2 font-mono text-[12px] text-subtle">{revision(deployment)}</span>
          </span>
          <span className="block text-[12px] text-subtle">
            {created.format(new Date(deployment.created_at))}
            {deployment.error_code ? ` · ${t(`errors.${deployment.error_code}`, { defaultValue: deployment.error_code })}` : ""}
          </span>
        </span>
      </button>
      {failure ? <p className="w-full text-[13px] text-danger">{errorMessage(failure, t)}</p> : null}
      <div className="flex gap-2">
        {isActive(deployment.status) ? (
          <Button variant="ghost" busy={cancel.isPending} onClick={() => cancel.mutate(deployment.id)}>
            {t("deployments.cancel")}
          </Button>
        ) : null}
        {canRollback ? (
          <Button variant="secondary" busy={rollback.isPending} onClick={() => rollback.mutate(deployment.id)}>
            {t("deployments.rollback")}
          </Button>
        ) : null}
      </div>
    </li>
  );
}
