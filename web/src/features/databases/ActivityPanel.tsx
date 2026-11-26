import { useState } from "react";
import { useQuery } from "@tanstack/react-query";

import { useTranslation } from "@/shared/i18n";
import { LogViewer, socketUrl } from "@/shared/logs";
import { SegmentedControl } from "@/shared/ui";

import { deployLogQuery, type Database } from "./api";

type View = "deploy" | "container";

function DeployLog({ database }: { database: Database }) {
  const { t } = useTranslation("databases");
  const log = useQuery(deployLogQuery(database.id, database.status === "deploying"));
  return (
    <pre
      aria-live="off"
      className="h-80 overflow-auto rounded-control border border-border bg-bg p-3 font-mono text-[12px] leading-relaxed whitespace-pre-wrap text-muted"
    >
      {log.data?.text || t("activity.noDeployLog")}
    </pre>
  );
}

export function ActivityPanel({ database }: { database: Database }) {
  const { t } = useTranslation("databases");
  const [view, setView] = useState<View>(database.running ? "container" : "deploy");
  const url = socketUrl(`/api/v1/databases/${encodeURIComponent(database.id)}/logs`);

  return (
    <section className="flex flex-col gap-3">
      <SegmentedControl<View>
        label={t("activity.title")}
        value={view}
        segments={[
          { value: "container", label: t("activity.container") },
          { value: "deploy", label: t("activity.deploy") },
        ]}
        onChange={setView}
      />
      {view === "deploy" ? <DeployLog database={database} /> : null}
      {view === "container" && database.running ? <LogViewer key={database.updated_at} url={url} /> : null}
      {view === "container" && !database.running ? (
        <p className="rounded-panel border border-border bg-surface px-4 py-6 text-muted">{t("activity.notRunning")}</p>
      ) : null}
    </section>
  );
}
