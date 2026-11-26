import { useState } from "react";
import { CaretRightIcon, DatabaseIcon, PlusIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert, SegmentedControl, StatusBadge } from "@/shared/ui";

import { databasesQuery, type Database } from "./api";
import { CreateDatabaseForm } from "./CreateDatabaseForm";
import { engineSummary, statusKey, statusTone } from "./presentation";

interface Environment {
  id: string;
  name: string;
}

function DatabaseRow({ database }: { database: Database }) {
  const { t } = useTranslation("databases");
  return (
    <li className="border-b border-border last:border-b-0">
      <Link
        to="/databases/$databaseId"
        params={{ databaseId: database.id }}
        className="group flex items-center gap-3 px-4 py-3 hover:bg-surface-hover"
      >
        <DatabaseIcon size={18} aria-hidden className="shrink-0 text-muted" />
        <div className="min-w-0 flex-1">
          <p className="truncate font-medium text-text">{database.name}</p>
          <p className="truncate font-mono text-[12px] text-subtle">{engineSummary(database)}</p>
        </div>
        <StatusBadge tone={statusTone(database)} label={t(statusKey(database))} />
        <CaretRightIcon size={16} aria-hidden className="text-subtle group-hover:text-text" />
      </Link>
    </li>
  );
}

function pickEnvironment(environments: Environment[], selected: string): string {
  return environments.some((env) => env.id === selected) ? selected : (environments[0]?.id ?? "");
}

interface DatabasesSectionProps {
  projectId: string;
  environments: Environment[];
}

export function DatabasesSection({ projectId, environments }: DatabasesSectionProps) {
  const { t } = useTranslation("databases");
  const databases = useQuery(databasesQuery(projectId));
  const [selected, setSelected] = useState(environments[0]?.id ?? "");
  const [creating, setCreating] = useState(false);
  const environmentId = pickEnvironment(environments, selected);
  const visible = databases.data?.filter((database) => database.environment_id === environmentId) ?? [];
  const empty = !databases.isPending && visible.length === 0 && !creating;

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
          <Button variant="secondary" icon={<PlusIcon size={16} weight="bold" aria-hidden />} onClick={() => setCreating(true)}>
            {t("new")}
          </Button>
        )}
      </header>
      {databases.isError ? <InlineAlert tone="danger">{errorMessage(databases.error, t)}</InlineAlert> : null}
      <div className="overflow-hidden rounded-panel border border-border bg-surface">
        {creating ? (
          <CreateDatabaseForm key={environmentId} environmentId={environmentId} onCancel={() => setCreating(false)} />
        ) : null}
        {databases.isPending ? <div aria-hidden className="skeleton m-4 h-10" /> : null}
        {empty ? <p className="px-4 py-6 text-muted">{t("empty")}</p> : null}
        <ul>
          {visible.map((database) => (
            <DatabaseRow key={database.id} database={database} />
          ))}
        </ul>
      </div>
    </section>
  );
}
