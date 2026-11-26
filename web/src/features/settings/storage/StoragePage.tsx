import { useState } from "react";
import { CloudIcon, HardDriveIcon, PlusIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert } from "@/shared/ui";

import { AddDestinationForm } from "./AddDestinationForm";
import { destinationsQuery, useDeleteDestination, useTestDestination, type Destination } from "./api";

function location(destination: Destination): string {
  const base = destination.kind === "s3" ? `s3://${destination.bucket ?? ""}` : "local";
  return destination.path_prefix ? `${base}/${destination.path_prefix}` : base;
}

function DestinationRow({ destination }: { destination: Destination }) {
  const { t } = useTranslation("settings");
  const test = useTestDestination(destination.id);
  const remove = useDeleteDestination(destination.id);
  const Icon = destination.kind === "s3" ? CloudIcon : HardDriveIcon;
  const failure = test.error ?? remove.error;

  return (
    <li className="flex flex-wrap items-center justify-between gap-3 border-b border-border px-4 py-3 last:border-b-0">
      <div className="flex min-w-0 items-center gap-3">
        <Icon size={18} aria-hidden className="shrink-0 text-muted" />
        <div className="min-w-0">
          <p className="truncate font-medium text-text">{destination.name}</p>
          <p className="truncate font-mono text-[12px] text-subtle">{location(destination)}</p>
          {test.isSuccess ? <p className="text-[13px] text-success">{t("storage.reachable")}</p> : null}
          {failure ? <p className="text-[13px] text-danger">{errorMessage(failure, t)}</p> : null}
        </div>
      </div>
      <div className="flex gap-2">
        <Button variant="secondary" busy={test.isPending} onClick={() => test.mutate()}>
          {t("storage.test")}
        </Button>
        <Button variant="ghost" busy={remove.isPending} onClick={() => remove.mutate()}>
          {t("storage.remove")}
        </Button>
      </div>
    </li>
  );
}

export function StoragePage() {
  const { t } = useTranslation("settings");
  const destinations = useQuery(destinationsQuery);
  const [adding, setAdding] = useState(false);
  const empty = destinations.data?.length === 0 && !adding;

  return (
    <section className="flex flex-col gap-5">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex max-w-xl flex-col gap-1">
          <h2 className="text-lg text-text">{t("storage.title")}</h2>
          <p className="text-muted">{t("storage.description")}</p>
        </div>
        {adding ? null : (
          <Button icon={<PlusIcon size={16} weight="bold" aria-hidden />} onClick={() => setAdding(true)}>
            {t("storage.new")}
          </Button>
        )}
      </header>
      {destinations.isError ? <InlineAlert tone="danger">{errorMessage(destinations.error, t)}</InlineAlert> : null}
      <div className="overflow-hidden rounded-panel border border-border bg-surface">
        {adding ? <AddDestinationForm onDone={() => setAdding(false)} /> : null}
        {destinations.isPending ? <div aria-hidden className="skeleton m-4 h-10" /> : null}
        {empty ? <p className="px-4 py-6 text-muted">{t("storage.empty")}</p> : null}
        <ul>
          {destinations.data?.map((destination) => (
            <DestinationRow key={destination.id} destination={destination} />
          ))}
        </ul>
      </div>
    </section>
  );
}
