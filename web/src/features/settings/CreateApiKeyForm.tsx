import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";

import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert, SelectField, TextField } from "@/shared/ui";

import { useCreateApiKey, type Role } from "./api";

interface FormValues {
  name: string;
  role: Role;
  expiresInDays: string;
}

const ROLES: Role[] = ["viewer", "developer", "admin", "owner"];
const EXPIRY_OPTIONS = ["30", "90", "365", "never"];

interface CreateApiKeyFormProps {
  onCreated: (key: string) => void;
  onCancel: () => void;
}

export function CreateApiKeyForm({ onCreated, onCancel }: CreateApiKeyFormProps) {
  const { t } = useTranslation("settings");
  const create = useCreateApiKey();
  const { register, handleSubmit, formState } = useForm<FormValues>({
    defaultValues: { role: "developer", expiresInDays: "90" },
  });

  const onSubmit = handleSubmit((values) => {
    const days = values.expiresInDays === "never" ? null : Number(values.expiresInDays);
    create.mutate(
      { name: values.name, role: values.role, expires_in_days: days },
      { onSuccess: (created) => onCreated(created.key) },
    );
  });

  return (
    <form
      onSubmit={onSubmit}
      noValidate
      className="enter-settle grid gap-4 rounded-panel border border-border bg-surface p-4 sm:grid-cols-[2fr_1fr_1fr_auto] sm:items-end"
    >
      {create.isError ? (
        <InlineAlert tone="danger" className="sm:col-span-4">
          {errorMessage(create.error, t)}
        </InlineAlert>
      ) : null}
      <TextField
        label={t("apiKeys.fields.name")}
        placeholder={t("apiKeys.fields.namePlaceholder")}
        autoFocus
        error={formState.errors.name ? t("validation.required", { ns: "auth" }) : undefined}
        {...register("name", { required: true, maxLength: 100 })}
      />
      <SelectField
        label={t("apiKeys.fields.role")}
        options={ROLES.map((role) => ({ value: role, label: t(`roles.${role}`) }))}
        {...register("role")}
      />
      <SelectField
        label={t("apiKeys.fields.expiry")}
        options={EXPIRY_OPTIONS.map((value) => ({ value, label: t(`apiKeys.expiry.${value}`) }))}
        {...register("expiresInDays")}
      />
      <div className="flex gap-2">
        <Button variant="ghost" onClick={onCancel}>
          {t("actions.cancel", { ns: "common" })}
        </Button>
        <Button type="submit" busy={create.isPending}>
          {t("apiKeys.create")}
        </Button>
      </div>
    </form>
  );
}
