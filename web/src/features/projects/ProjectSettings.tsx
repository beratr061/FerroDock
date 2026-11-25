import { useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";

import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert, TextField } from "@/shared/ui";

import { useDeleteProject, useUpdateProject, type Project } from "./api";

interface FormValues {
  name: string;
  description: string;
}

function DetailsForm({ project }: { project: Project }) {
  const { t } = useTranslation("projects");
  const update = useUpdateProject(project.id);
  const { register, handleSubmit, formState, reset } = useForm<FormValues>({
    values: { name: project.name, description: project.description },
  });
  const onSubmit = handleSubmit((values) => update.mutate(values, { onSuccess: () => reset(values) }));

  return (
    <form onSubmit={onSubmit} noValidate className="flex flex-col gap-4 p-4">
      {update.isError ? <InlineAlert tone="danger">{errorMessage(update.error, t)}</InlineAlert> : null}
      <div className="grid gap-4 md:grid-cols-[1fr_2fr]">
        <TextField
          label={t("fields.name")}
          error={formState.errors.name ? t("validation.required", { ns: "auth" }) : undefined}
          {...register("name", { required: true, maxLength: 100 })}
        />
        <TextField label={t("fields.description")} {...register("description", { maxLength: 500 })} />
      </div>
      <div className="flex justify-end">
        <Button type="submit" busy={update.isPending} disabled={!formState.isDirty}>
          {t("save")}
        </Button>
      </div>
    </form>
  );
}

function DeleteProject({ project }: { project: Project }) {
  const { t } = useTranslation("projects");
  const remove = useDeleteProject(project.id);
  const navigate = useNavigate();
  const [confirming, setConfirming] = useState(false);
  const onDelete = (): void =>
    remove.mutate(undefined, { onSuccess: () => void navigate({ to: "/projects" }) });

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

export function ProjectSettings({ project }: { project: Project }) {
  const { t } = useTranslation("projects");
  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-lg text-text">{t("settings")}</h2>
      <div className="rounded-panel border border-border bg-surface">
        <DetailsForm project={project} />
        <DeleteProject project={project} />
      </div>
    </section>
  );
}
