import { useState } from "react";
import { KeyIcon, PlusIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import { errorMessage } from "@/shared/lib/errors";
import { Button, EmptyState, InlineAlert } from "@/shared/ui";

import { apiKeysQuery } from "./api";
import { ApiKeyRow } from "./ApiKeyRow";
import { CreateApiKeyForm } from "./CreateApiKeyForm";
import { CreatedKeyNotice } from "./CreatedKeyNotice";

function ListSkeleton() {
  return (
    <div aria-hidden className="flex flex-col gap-2">
      <div className="skeleton h-14" />
      <div className="skeleton h-14" />
    </div>
  );
}

export function ApiKeysPage() {
  const { t } = useTranslation("settings");
  const keys = useQuery(apiKeysQuery);
  const [creating, setCreating] = useState(false);
  const [secret, setSecret] = useState<string | null>(null);

  const onCreated = (key: string): void => {
    setCreating(false);
    setSecret(key);
  };

  return (
    <section className="flex flex-col gap-5">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex max-w-xl flex-col gap-1">
          <h2 className="text-lg text-text">{t("apiKeys.title")}</h2>
          <p className="text-muted">{t("apiKeys.description")}</p>
        </div>
        {creating ? null : (
          <Button icon={<PlusIcon size={16} weight="bold" aria-hidden />} onClick={() => setCreating(true)}>
            {t("apiKeys.new")}
          </Button>
        )}
      </header>
      {secret ? <CreatedKeyNotice secret={secret} onDismiss={() => setSecret(null)} /> : null}
      {creating ? <CreateApiKeyForm onCreated={onCreated} onCancel={() => setCreating(false)} /> : null}
      {keys.isPending ? <ListSkeleton /> : null}
      {keys.isError ? <InlineAlert tone="danger">{errorMessage(keys.error, t)}</InlineAlert> : null}
      {keys.data?.length === 0 && !creating ? (
        <EmptyState
          icon={<KeyIcon size={22} aria-hidden />}
          title={t("apiKeys.empty.title")}
          description={t("apiKeys.empty.description")}
        />
      ) : null}
      {keys.data && keys.data.length > 0 ? (
        <ul className="rounded-panel border border-border bg-surface">
          {keys.data.map((apiKey) => (
            <ApiKeyRow key={apiKey.id} apiKey={apiKey} />
          ))}
        </ul>
      ) : null}
    </section>
  );
}
