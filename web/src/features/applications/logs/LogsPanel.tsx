import { useState } from "react";
import { ScrollIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";

import { useTranslation } from "@/shared/i18n";
import { LogViewer } from "@/shared/logs";
import { SelectField } from "@/shared/ui";

import { containersQuery } from "../api";
import { logsUrl } from "./protocol";

export function LogsPanel({ applicationId }: { applicationId: string }) {
  const { t } = useTranslation("applications");
  const containers = useQuery(containersQuery(applicationId));
  const [chosen, setChosen] = useState<string>();
  const list = containers.data ?? [];
  const fallback = list.find((container) => container.running) ?? list[0];
  const selected = list.find((container) => container.id === chosen) ?? fallback;

  return (
    <section className="flex flex-col gap-3">
      <header className="flex flex-wrap items-end justify-between gap-3">
        <div className="flex flex-col gap-1">
          <h2 className="flex items-center gap-2 text-lg text-text">
            <ScrollIcon size={18} aria-hidden />
            {t("logs.title")}
          </h2>
          <p className="max-w-2xl text-muted">{t("logs.description")}</p>
        </div>
        {list.length > 1 ? (
          <SelectField
            label={t("logs.container")}
            hideLabel
            className="w-64"
            value={selected?.id}
            onChange={(event) => setChosen(event.target.value)}
            options={list.map((container) => ({ value: container.id, label: container.name }))}
          />
        ) : null}
      </header>
      {selected ? (
        <LogViewer key={selected.id} url={logsUrl(applicationId, selected.id)} />
      ) : (
        <p className="rounded-panel border border-border bg-surface px-4 py-6 text-muted">{t("logs.empty")}</p>
      )}
    </section>
  );
}
