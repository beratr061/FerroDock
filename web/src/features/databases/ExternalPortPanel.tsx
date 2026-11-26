import { GlobeIcon } from "@phosphor-icons/react";
import { useForm } from "react-hook-form";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert, TextField } from "@/shared/ui";

import { useSetExternalPort, type Database } from "./api";

const MIN_PORT = 1024;
const MAX_PORT = 65535;

function validPort(value: string): boolean {
  const port = Number(value);
  return Number.isInteger(port) && port >= MIN_PORT && port <= MAX_PORT;
}

export function ExternalPortPanel({ database }: { database: Database }) {
  const { t } = useTranslation("databases");
  const save = useSetExternalPort(database.id);
  const published = database.external_port ?? null;
  const form = useForm<{ port: string }>({ defaultValues: { port: published === null ? "" : String(published) } });

  const onSubmit = form.handleSubmit((values) => save.mutate(Number(values.port)));

  return (
    <section className="flex flex-col gap-3">
      <header className="flex flex-col gap-1">
        <h2 className="flex items-center gap-2 text-lg text-text">
          <GlobeIcon size={18} aria-hidden />
          {t("external.title")}
        </h2>
        <p className="max-w-2xl text-muted">{t("external.description")}</p>
      </header>
      <form onSubmit={onSubmit} noValidate className="flex flex-col gap-3 rounded-panel border border-border bg-surface p-4">
        {save.isError ? <InlineAlert tone="danger">{errorMessage(save.error, t)}</InlineAlert> : null}
        {published === null ? null : <InlineAlert tone="warning">{t("external.exposed", { port: published })}</InlineAlert>}
        <div className="flex flex-wrap items-end gap-2">
          <TextField
            label={t("external.port")}
            inputMode="numeric"
            placeholder={String(database.internal_port + 10000)}
            className="w-40"
            error={form.formState.errors.port ? t("external.invalid", { min: MIN_PORT }) : undefined}
            {...form.register("port", { validate: validPort })}
          />
          <Button type="submit" busy={save.isPending && save.variables !== null} disabled={!form.formState.isDirty}>
            {t("external.publish")}
          </Button>
          {published === null ? null : (
            <Button variant="ghost" busy={save.isPending && save.variables === null} onClick={() => save.mutate(null)}>
              {t("external.remove")}
            </Button>
          )}
        </div>
      </form>
    </section>
  );
}
