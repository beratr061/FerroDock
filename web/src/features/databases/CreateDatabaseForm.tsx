import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useForm, useWatch } from "react-hook-form";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert, SelectField, TextField } from "@/shared/ui";

import { enginesQuery, useCreateDatabase } from "./api";
import { engineLabel } from "./presentation";

interface FormValues {
  name: string;
  engine: string;
  version: string;
  databaseName: string;
  username: string;
}

const IDENTIFIER = /^[a-z_][a-z0-9_]{0,62}$/;

function optional(value: string): string | null {
  return value.trim() === "" ? null : value.trim();
}

interface CreateDatabaseFormProps {
  environmentId: string;
  onCancel: () => void;
}

export function CreateDatabaseForm({ environmentId, onCancel }: CreateDatabaseFormProps) {
  const { t } = useTranslation("databases");
  const navigate = useNavigate();
  const engines = useQuery(enginesQuery);
  const create = useCreateDatabase(environmentId);
  const form = useForm<FormValues>({
    defaultValues: { name: "", engine: "postgres", version: "", databaseName: "", username: "" },
  });
  const engineName = useWatch({ control: form.control, name: "engine" });
  const engine = engines.data?.find((candidate) => candidate.engine === engineName);
  const invalid = t("validation.identifier");

  const onSubmit = form.handleSubmit((values) => {
    const body = {
      name: values.name,
      engine: values.engine,
      version: optional(values.version),
      database_name: optional(values.databaseName),
      username: engine?.has_username ? optional(values.username) : null,
    };
    create.mutate(body, {
      onSuccess: (database) =>
        void navigate({ to: "/databases/$databaseId", params: { databaseId: database.id } }),
    });
  });

  return (
    <form onSubmit={onSubmit} noValidate className="enter-settle flex flex-col gap-4 border-b border-border p-4">
      {create.isError ? <InlineAlert tone="danger">{errorMessage(create.error, t)}</InlineAlert> : null}
      <div className="grid gap-4 md:grid-cols-3">
        <TextField
          label={t("fields.name")}
          placeholder={t("fields.namePlaceholder")}
          autoFocus
          error={form.formState.errors.name ? t("validation.required", { ns: "auth" }) : undefined}
          {...form.register("name", { required: true, maxLength: 100 })}
        />
        <SelectField
          label={t("fields.engine")}
          options={(engines.data ?? []).map((item) => ({ value: item.engine, label: engineLabel(item.engine) }))}
          {...form.register("engine", { onChange: () => form.setValue("version", "") })}
        />
        <SelectField
          label={t("fields.version")}
          options={(engine?.versions ?? []).map((version, index) => ({
            value: index === 0 ? "" : version,
            label: index === 0 ? t("fields.latest", { version }) : version,
          }))}
          {...form.register("version")}
        />
      </div>
      <div className="grid gap-4 md:grid-cols-3">
        <TextField
          label={t("fields.databaseName")}
          placeholder={t("fields.fromName")}
          spellCheck={false}
          error={form.formState.errors.databaseName ? invalid : undefined}
          {...form.register("databaseName", { validate: (value) => value === "" || IDENTIFIER.test(value) })}
        />
        {engine?.has_username === false ? null : (
          <TextField
            label={t("fields.username")}
            placeholder="app"
            spellCheck={false}
            error={form.formState.errors.username ? invalid : undefined}
            {...form.register("username", { validate: (value) => value === "" || IDENTIFIER.test(value) })}
          />
        )}
      </div>
      <p className="text-muted">{t("passwordNote")}</p>
      <div className="flex justify-end gap-2">
        <Button variant="ghost" onClick={onCancel}>
          {t("actions.cancel", { ns: "common" })}
        </Button>
        <Button type="submit" busy={create.isPending}>
          {t("create")}
        </Button>
      </div>
    </form>
  );
}
