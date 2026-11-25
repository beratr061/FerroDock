import { useEffect, useState } from "react";
import { ArrowClockwiseIcon, ArrowCounterClockwiseIcon, ArrowSquareOutIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert, StatusBadge, type StatusTone } from "@/shared/ui";

import { updateCheckQuery, updateOverviewQuery, useStartUpdate, waitForRestart, type UpdateCheck, type UpdateRecord } from "./api";

const STATE_TONES: Record<UpdateRecord["state"], StatusTone> = {
  pending: "info",
  applying: "warning",
  succeeded: "success",
  rolled_back: "warning",
  failed: "danger",
};

function LastUpdate({ record }: { record: UpdateRecord }) {
  const { t, i18n } = useTranslation("settings");
  const when = new Intl.DateTimeFormat(i18n.language, { dateStyle: "medium", timeStyle: "short" });
  return (
    <div className="flex flex-col gap-1 rounded-panel border border-border bg-surface p-4">
      <div className="flex flex-wrap items-center gap-2">
        <StatusBadge tone={STATE_TONES[record.state]} label={t(`updates.state.${record.state}`)} />
        <span className="text-[12px] text-subtle">{when.format(new Date(record.started_at))}</span>
      </div>
      <p className="truncate font-mono text-[12px] text-muted">{record.to_image}</p>
      {record.error ? <p className="text-[13px] text-danger">{t(`errors.${record.error}`, { defaultValue: record.error })}</p> : null}
    </div>
  );
}

function Restarting({ from }: { from: string }) {
  const { t } = useTranslation("settings");
  useEffect(() => {
    const controller = new AbortController();
    void waitForRestart(from, controller.signal).then(() => window.location.reload());
    return () => controller.abort();
  }, [from]);
  return <InlineAlert tone="info">{t("updates.restarting")}</InlineAlert>;
}

function CheckResult({ check }: { check: UpdateCheck | undefined }) {
  const { t } = useTranslation("settings");
  if (!check) {
    return null;
  }
  if (check.status !== "available") {
    return <InlineAlert tone="success">{t(`updates.check.${check.status}`, { ...check })}</InlineAlert>;
  }
  return (
    <InlineAlert tone="info">
      <span className="flex flex-wrap items-center gap-2">
        {t("updates.check.available", { latest: check.latest })}
        <a href={check.notes_url} target="_blank" rel="noreferrer noopener" className="inline-flex items-center gap-1 underline">
          {t("updates.notes")}
          <ArrowSquareOutIcon size={14} aria-hidden />
        </a>
      </span>
    </InlineAlert>
  );
}

function Actions({ current, last }: { current: string; last: UpdateRecord | null | undefined }) {
  const { t } = useTranslation("settings");
  const check = useQuery(updateCheckQuery);
  const update = useStartUpdate(false);
  const rollback = useStartUpdate(true);
  const [restarting, setRestarting] = useState(false);
  const available = check.data?.status === "available" ? check.data : null;
  const canRollBack = last?.state === "succeeded" && Boolean(last.from_image);
  const failure = check.error ?? update.error ?? rollback.error;

  if (restarting) {
    return <Restarting from={current} />;
  }
  return (
    <div className="flex flex-col gap-3">
      {failure ? <InlineAlert tone="danger">{errorMessage(failure, t)}</InlineAlert> : null}
      <CheckResult check={check.data} />
      <div className="flex flex-wrap gap-2">
        <Button variant="secondary" icon={<ArrowClockwiseIcon size={16} aria-hidden />} busy={check.isFetching} onClick={() => void check.refetch()}>
          {t("updates.checkNow")}
        </Button>
        {available ? (
          <Button busy={update.isPending} onClick={() => update.mutate(undefined, { onSuccess: () => setRestarting(true) })}>
            {t("updates.install", { latest: available.latest })}
          </Button>
        ) : null}
        {canRollBack ? (
          <Button variant="ghost" icon={<ArrowCounterClockwiseIcon size={16} aria-hidden />} busy={rollback.isPending} onClick={() => rollback.mutate(undefined, { onSuccess: () => setRestarting(true) })}>
            {t("updates.rollback")}
          </Button>
        ) : null}
      </div>
    </div>
  );
}

export function UpdatesPage() {
  const { t } = useTranslation("settings");
  const overview = useQuery(updateOverviewQuery);

  return (
    <section className="flex flex-col gap-5">
      <div className="flex max-w-xl flex-col gap-1">
        <h2 className="text-lg text-text">{t("updates.title")}</h2>
        <p className="text-muted">{t("updates.description")}</p>
      </div>
      {overview.isError ? <InlineAlert tone="danger">{errorMessage(overview.error, t)}</InlineAlert> : null}
      {overview.data ? (
        <>
          <p className="text-text">
            {t("updates.current")} <span className="font-mono">{overview.data.current_version}</span>
          </p>
          {overview.data.last ? <LastUpdate record={overview.data.last} /> : null}
          <Actions current={overview.data.current_version} last={overview.data.last} />
        </>
      ) : null}
    </section>
  );
}
