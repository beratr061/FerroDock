import { useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";

import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert, TextField } from "@/shared/ui";

import { useDeleteApplication, useUpdateApplication, type Application } from "./api";
import { sourceValues, toSourceRequest, type SourceFormValues } from "./sourceForm";
import { SourceFields } from "./SourceFields";

type FormValues = SourceFormValues & { name: string; description: string };

function EditForm({ application }: { application: Application }) {
  const { t } = useTranslation("applications");
  const update = useUpdateApplication(application.id);
  const form = useForm<FormValues>({
    values: { ...sourceValues(application), name: application.name, description: application.description },
  });
  const onSubmit = form.handleSubmit((values) => {
    const body = {
      name: values.name,
      description: values.description,
      source: toSourceRequest(values),
      strategy: values.strategy,
    };
    update.mutate(body, { onSuccess: () => form.reset(values) });
  });

  return (
    <form onSubmit={onSubmit} noValidate className="flex flex-col gap-4 p-4">
      {update.isError ? <InlineAlert tone="danger">{errorMessage(update.error, t)}</InlineAlert> : null}
      <div className="grid gap-4 md:grid-cols-[1fr_2fr]">
        <TextField
          label={t("fields.name")}
          error={form.formState.errors.name ? t("validation.required", { ns: "auth" }) : undefined}
          {...form.register("name", { required: true, maxLength: 100 })}
        />
        <TextField label={t("fields.description")} {...form.register("description", { maxLength: 500 })} />
      </div>
      <SourceFields form={form} />
      <div className="flex justify-end">
        <Button type="submit" busy={update.isPending} disabled={!form.formState.isDirty}>
          {t("save")}
        </Button>
      </div>
    </form>
  );
}

function DeleteApplication({ application }: { application: Application }) {
  const { t } = useTranslation("applications");
  const remove = useDeleteApplication(application.id);
  const navigate = useNavigate();
  const [confirming, setConfirming] = useState(false);
  const onDelete = (): void =>
    remove.mutate(undefined, {
      onSuccess: () => void navigate({ to: "/projects/$projectId", params: { projectId: application.project_id } }),
    });

  return (
    <div className="flex flex-wrap items-center justify-between gap-3 border-t border-border p-4">
      <div className="flex max-w-xl flex-col gap-1">
        <p className="font-medium text-text">{t("delete.title")}</p>
        <p className="text-muted">{t("delete.description")}</p>
        {remove.isError ? <p className="text-[13px] text-danger">{errorMessage(remove.error, t)}</p> : null}
      </div>
      {confirming ? (
        <div className="flex gap-2">
          <Button variant="ghost" onClick={() => setConfirming(false)}>
            {t("actions.cancel", { ns: "common" })}
          </Button>
          <Button variant="danger" busy={remove.isPending} onClick={onDelete}>
            {t("delete.confirm")}
          </Button>
        </div>
      ) : (
        <Button variant="danger" onClick={() => setConfirming(true)}>
          {t("delete.action")}
        </Button>
      )}
    </div>
  );
}

export function ApplicationSettings({ application }: { application: Application }) {
  const { t } = useTranslation("applications");
  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-lg text-text">{t("settings")}</h2>
      <div className="rounded-panel border border-border bg-surface">
        <EditForm application={application} />
        <DeleteApplication application={application} />
      </div>
    </section>
  );
}
