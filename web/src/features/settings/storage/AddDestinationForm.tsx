import { useForm, useWatch } from "react-hook-form";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, Checkbox, InlineAlert, SegmentedControl, TextField } from "@/shared/ui";

import { useCreateDestination, type NewDestination } from "./api";

type Kind = "local" | "s3";

interface FormValues {
  kind: Kind;
  name: string;
  prefix: string;
  endpoint: string;
  region: string;
  bucket: string;
  accessKeyId: string;
  secretAccessKey: string;
  pathStyle: boolean;
}

const EMPTY: FormValues = {
  kind: "local",
  name: "",
  prefix: "",
  endpoint: "",
  region: "us-east-1",
  bucket: "",
  accessKeyId: "",
  secretAccessKey: "",
  pathStyle: false,
};

function toRequest(values: FormValues): NewDestination {
  const s3 =
    values.kind === "s3"
      ? {
          endpoint: values.endpoint.trim() || null,
          region: values.region,
          bucket: values.bucket,
          access_key_id: values.accessKeyId,
          secret_access_key: values.secretAccessKey,
          path_style: values.pathStyle,
        }
      : null;
  return { name: values.name, path_prefix: values.prefix || null, s3 };
}

function S3Fields({ form }: { form: ReturnType<typeof useForm<FormValues>> }) {
  const { t } = useTranslation("settings");
  const required = t("validation.required", { ns: "auth" });
  const errors = form.formState.errors;
  return (
    <>
      <div className="grid gap-4 md:grid-cols-3">
        <TextField label={t("storage.endpoint")} placeholder="https://s3.eu-central-1.amazonaws.com" hint={t("storage.endpointHint")} spellCheck={false} {...form.register("endpoint")} />
        <TextField label={t("storage.region")} spellCheck={false} error={errors.region ? required : undefined} {...form.register("region", { required: true })} />
        <TextField label={t("storage.bucket")} spellCheck={false} error={errors.bucket ? required : undefined} {...form.register("bucket", { required: true })} />
      </div>
      <div className="grid gap-4 md:grid-cols-2">
        <TextField label={t("storage.accessKeyId")} spellCheck={false} autoComplete="off" error={errors.accessKeyId ? required : undefined} {...form.register("accessKeyId", { required: true })} />
        <TextField label={t("storage.secretAccessKey")} type="password" autoComplete="new-password" error={errors.secretAccessKey ? required : undefined} {...form.register("secretAccessKey", { required: true })} />
      </div>
      <Checkbox label={t("storage.pathStyle")} {...form.register("pathStyle")} />
    </>
  );
}

export function AddDestinationForm({ onDone }: { onDone: () => void }) {
  const { t } = useTranslation("settings");
  const create = useCreateDestination();
  const form = useForm<FormValues>({ defaultValues: EMPTY });
  const kind = useWatch({ control: form.control, name: "kind" });
  const onSubmit = form.handleSubmit((values) => create.mutate(toRequest(values), { onSuccess: onDone }));

  return (
    <form onSubmit={onSubmit} noValidate className="enter-settle flex flex-col gap-4 border-b border-border p-4">
      {create.isError ? <InlineAlert tone="danger">{errorMessage(create.error, t)}</InlineAlert> : null}
      <SegmentedControl<Kind>
        label={t("storage.kind")}
        value={kind}
        segments={[
          { value: "local", label: t("storage.local") },
          { value: "s3", label: t("storage.s3") },
        ]}
        onChange={(next) => form.setValue("kind", next)}
      />
      <div className="grid gap-4 md:grid-cols-2">
        <TextField
          label={t("storage.name")}
          placeholder={kind === "s3" ? "Offsite S3" : t("storage.localPlaceholder")}
          error={form.formState.errors.name ? t("validation.required", { ns: "auth" }) : undefined}
          {...form.register("name", { required: true, maxLength: 100 })}
        />
        <TextField label={t("storage.prefix")} placeholder="ferrodock" spellCheck={false} {...form.register("prefix")} />
      </div>
      {kind === "s3" ? <S3Fields form={form} /> : <p className="text-muted">{t("storage.localNote")}</p>}
      <div className="flex justify-end gap-2">
        <Button variant="ghost" onClick={onDone}>
          {t("actions.cancel", { ns: "common" })}
        </Button>
        <Button type="submit" busy={create.isPending}>
          {t("storage.add")}
        </Button>
      </div>
    </form>
  );
}
