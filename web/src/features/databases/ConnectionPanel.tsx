import { KeyIcon, PlugsConnectedIcon } from "@phosphor-icons/react";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, CopyButton } from "@/shared/ui";

import { useRevealCredentials, type Database } from "./api";
import { internalAddress } from "./presentation";

function Field({ label, value, secret = false }: { label: string; value: string; secret?: boolean }) {
  return (
    <div className="grid gap-2 border-b border-border px-4 py-3 last:border-b-0 md:grid-cols-[12rem_1fr_auto] md:items-center">
      <dt className="text-muted">{label}</dt>
      <dd className="min-w-0 truncate font-mono text-[12px] text-text" translate="no">
        {value}
      </dd>
      {secret ? <CopyButton value={value} /> : <span aria-hidden />}
    </div>
  );
}

export function ConnectionPanel({ database }: { database: Database }) {
  const { t } = useTranslation("databases");
  const reveal = useRevealCredentials(database.id);
  const secrets = reveal.data;

  return (
    <section className="flex flex-col gap-3">
      <header className="flex flex-col gap-1">
        <h2 className="flex items-center gap-2 text-lg text-text">
          <PlugsConnectedIcon size={18} aria-hidden />
          {t("connection.title")}
        </h2>
        <p className="max-w-2xl text-muted">{t("connection.description")}</p>
      </header>
      <dl className="overflow-hidden rounded-panel border border-border bg-surface">
        <Field label={t("connection.internalHost")} value={internalAddress(database)} />
        <Field label={t("connection.database")} value={database.database_name} />
        <Field label={t("connection.username")} value={database.username} />
        {secrets ? (
          <>
            <Field label={t("connection.password")} value={secrets.password} secret />
            {secrets.root_password ? (
              <Field label={t("connection.rootPassword")} value={secrets.root_password} secret />
            ) : null}
            <Field label={t("connection.internalUrl")} value={secrets.internal_url} secret />
            {secrets.external_url ? (
              <Field label={t("connection.externalUrl")} value={secrets.external_url} secret />
            ) : null}
          </>
        ) : (
          <div className="flex flex-wrap items-center justify-between gap-3 px-4 py-3">
            <p className="text-muted">{t("connection.hidden")}</p>
            <Button
              variant="secondary"
              busy={reveal.isPending}
              icon={<KeyIcon size={16} aria-hidden />}
              onClick={() => reveal.mutate()}
            >
              {t("connection.reveal")}
            </Button>
          </div>
        )}
      </dl>
      {reveal.isError ? <p className="text-[13px] text-danger">{errorMessage(reveal.error, t)}</p> : null}
    </section>
  );
}
