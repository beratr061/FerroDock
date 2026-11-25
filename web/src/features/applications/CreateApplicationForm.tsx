import { useNavigate } from "@tanstack/react-router";
import { useForm } from "react-hook-form";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert, TextField } from "@/shared/ui";

import { useCreateApplication } from "./api";
import { EMPTY_SOURCE, toSourceRequest, type SourceFormValues } from "./sourceForm";
import { SourceFields } from "./SourceFields";

type FormValues = SourceFormValues & { name: string; description: string };

interface CreateApplicationFormProps {
  environmentId: string;
  onCancel: () => void;
}

export function CreateApplicationForm({ environmentId, onCancel }: CreateApplicationFormProps) {
  const { t } = useTranslation("applications");
  const navigate = useNavigate();
  const create = useCreateApplication(environmentId);
  const form = useForm<FormValues>({ defaultValues: { ...EMPTY_SOURCE, name: "", description: "" } });

  const onSubmit = form.handleSubmit((values) => {
    const body = {
      name: values.name,
      description: values.description,
      source: toSourceRequest(values),
      strategy: values.strategy,
    };
    create.mutate(body, {
      onSuccess: (application) =>
        void navigate({ to: "/applications/$applicationId", params: { applicationId: application.id } }),
    });
  });

  return (
    <form onSubmit={onSubmit} noValidate className="enter-settle flex flex-col gap-4 border-b border-border p-4">
      {create.isError ? <InlineAlert tone="danger">{errorMessage(create.error, t)}</InlineAlert> : null}
      <div className="grid gap-4 md:grid-cols-[1fr_2fr]">
        <TextField
          label={t("fields.name")}
          placeholder={t("fields.namePlaceholder")}
          autoFocus
          error={form.formState.errors.name ? t("validation.required", { ns: "auth" }) : undefined}
          {...form.register("name", { required: true, maxLength: 100 })}
        />
        <TextField label={t("fields.description")} {...form.register("description", { maxLength: 500 })} />
      </div>
      <SourceFields form={form} />
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
