import { useState } from "react";
import { HardDrivesIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import { ApiError } from "@/shared/api/client";
import { errorMessage } from "@/shared/lib/errors";
import { EmptyState, InlineAlert } from "@/shared/ui";

import { containersQuery, engineInfoQuery } from "./api";
import { ContainerRow } from "./ContainerRow";

function ListSkeleton() {
  return (
    <div aria-hidden className="flex flex-col gap-2">
      <div className="skeleton h-16" />
      <div className="skeleton h-16" />
      <div className="skeleton h-16" />
    </div>
  );
}

export function ServersPage() {
  const { t } = useTranslation("servers");
  const [includeStopped, setIncludeStopped] = useState(false);
  const engine = useQuery(engineInfoQuery);
  const containers = useQuery(containersQuery(includeStopped));
  const forbidden = containers.error instanceof ApiError && containers.error.status === 403;

  return (
    <div className="flex flex-col gap-6">
      <header className="flex flex-col gap-1">
        <h1 className="text-xl text-text">{t("title")}</h1>
        <p className="text-muted">{t("subtitle")}</p>
      </header>
      <section className="flex flex-col gap-4">
        <div className="flex flex-wrap items-end justify-between gap-3">
          <div className="flex flex-col gap-0.5">
            <h2 className="text-lg text-text">{t("local.title")}</h2>
            <p className="font-mono text-[12px] text-subtle">
              {engine.data
                ? t("local.engine", { version: engine.data.version, os: engine.data.os, arch: engine.data.arch })
                : " "}
            </p>
          </div>
          <label className="flex cursor-pointer items-center gap-2 text-muted select-none">
            <input
              type="checkbox"
              checked={includeStopped}
              onChange={(event) => setIncludeStopped(event.target.checked)}
              className="size-4 accent-current"
            />
            {t("showStopped")}
          </label>
        </div>
        {containers.isPending ? <ListSkeleton /> : null}
        {forbidden ? <InlineAlert tone="info">{t("adminOnly")}</InlineAlert> : null}
        {containers.isError && !forbidden ? (
          <InlineAlert tone="danger">{errorMessage(containers.error, t)}</InlineAlert>
        ) : null}
        {containers.data?.length === 0 ? (
          <EmptyState
            icon={<HardDrivesIcon size={22} aria-hidden />}
            title={t("empty.title")}
            description={t("empty.description")}
          />
        ) : null}
        {containers.data && containers.data.length > 0 ? (
          <ul className="rounded-panel border border-border bg-surface">
            {containers.data.map((container) => (
              <ContainerRow key={container.id} container={container} />
            ))}
          </ul>
        ) : null}
      </section>
    </div>
  );
}
