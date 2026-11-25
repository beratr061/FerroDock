import { useForm } from "react-hook-form";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert, TextField } from "@/shared/ui";

import { useCreateProject, type Project } from "./api";

interface FormValues {
  name: string;
  description: string;
}

interface CreateProjectFormProps {
  onCreated: (project: Project) => void;
  onCancel: () => void;
}

export function CreateProjectForm({ onCreated, onCancel }: CreateProjectFormProps) {
  const { t } = useTranslation("projects");
  const create = useCreateProject();
  const { register, handleSubmit, formState } = useForm<FormValues>();

  const onSubmit = handleSubmit((values) => {
    create.mutate({ name: values.name, description: values.description }, { onSuccess: onCreated });
  });

  return (
    <form
      onSubmit={onSubmit}
      noValidate
      className="enter-settle grid gap-4 rounded-panel border border-border bg-surface p-4 md:grid-cols-[1fr_2fr_auto] md:items-start"
    >
      {create.isError ? (
        <InlineAlert tone="danger" className="md:col-span-3">
          {errorMessage(create.error, t)}
        </InlineAlert>
      ) : null}
      <TextField
        label={t("fields.name")}
        placeholder={t("fields.namePlaceholder")}
        autoFocus
        error={formState.errors.name ? t("validation.required", { ns: "auth" }) : undefined}
        {...register("name", { required: true, maxLength: 100 })}
      />
      <TextField
        label={t("fields.description")}
        placeholder={t("fields.descriptionPlaceholder")}
        {...register("description", { maxLength: 500 })}
      />
      <div className="flex gap-2 md:mt-[26px]">
        <Button variant="ghost" onClick={onCancel}>
          {t("actions.cancel", { ns: "common" })}
        </Button>
        <Button type="submit" busy={create.isPending}>
          {t("createSubmit")}
        </Button>
      </div>
    </form>
  );
}
