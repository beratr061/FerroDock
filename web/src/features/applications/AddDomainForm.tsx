import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";

import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert, TextField } from "@/shared/ui";

import { useAddDomain } from "./domains";

interface FormValues {
  host: string;
  port: string;
  path: string;
  https: boolean;
}

export function AddDomainForm({ applicationId, onDone }: { applicationId: string; onDone: () => void }) {
  const { t } = useTranslation("applications");
  const add = useAddDomain(applicationId);
  const { register, handleSubmit, formState } = useForm<FormValues>({
    defaultValues: { host: "", port: "80", path: "", https: true },
  });
  const required = t("validation.required", { ns: "auth" });

  const onSubmit = handleSubmit((values) => {
    const body = {
      host: values.host,
      container_port: Number(values.port),
      path_prefix: values.path.trim() === "" ? null : values.path,
      https: values.https,
    };
    add.mutate(body, { onSuccess: onDone });
  });

  return (
    <form onSubmit={onSubmit} noValidate className="enter-settle flex flex-col gap-4 border-b border-border p-4">
      {add.isError ? <InlineAlert tone="danger">{errorMessage(add.error, t)}</InlineAlert> : null}
      <div className="grid gap-4 md:grid-cols-[2fr_1fr_1fr]">
        <TextField
          label={t("domains.host")}
          placeholder="app.example.com"
          autoFocus
          spellCheck={false}
          error={formState.errors.host ? required : undefined}
          {...register("host", { required: true })}
        />
        <TextField
          label={t("domains.port")}
          inputMode="numeric"
          hint={t("domains.portHint")}
          error={formState.errors.port ? t("domains.portInvalid") : undefined}
          {...register("port", { required: true, pattern: /^\d{1,5}$/, validate: (v) => Number(v) >= 1 && Number(v) <= 65535 })}
        />
        <TextField label={t("domains.path")} placeholder="/" spellCheck={false} {...register("path")} />
      </div>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <label className="inline-flex items-center gap-2 text-text">
          <input type="checkbox" className="size-4 accent-current" {...register("https")} />
          {t("domains.https")}
        </label>
        <div className="flex gap-2">
          <Button variant="ghost" onClick={onDone}>
            {t("actions.cancel", { ns: "common" })}
          </Button>
          <Button type="submit" busy={add.isPending}>
            {t("domains.add")}
          </Button>
        </div>
      </div>
    </form>
  );
}
