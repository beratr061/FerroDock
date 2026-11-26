import { useForm } from "react-hook-form";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, Checkbox, InlineAlert, TextField } from "@/shared/ui";

import { useAddDomain } from "./api";
import { BasicAuthFields } from "./BasicAuthFields";
import { EMPTY_DOMAIN_FORM, type DomainFormValues } from "./formValues";

interface AddDomainFormProps {
  applicationId: string;
  compose: boolean;
  onDone: () => void;
}

export function AddDomainForm({ applicationId, compose, onDone }: AddDomainFormProps) {
  const { t } = useTranslation("applications");
  const add = useAddDomain(applicationId);
  const { register, handleSubmit, formState, control } = useForm<DomainFormValues>({ defaultValues: EMPTY_DOMAIN_FORM });
  const required = t("validation.required", { ns: "auth" });

  const onSubmit = handleSubmit((values) => {
    const body = {
      host: values.host,
      container_port: Number(values.port),
      path_prefix: values.path.trim() === "" ? null : values.path,
      https: values.https,
      service: compose ? values.service.trim() : null,
      basic_auth: values.protect ? { username: values.authUser.trim(), password: values.authPassword } : null,
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
      {compose ? (
        <TextField
          label={t("domains.service")}
          placeholder="web"
          hint={t("domains.serviceHint")}
          spellCheck={false}
          error={formState.errors.service ? required : undefined}
          {...register("service", { required: true })}
        />
      ) : null}
      <BasicAuthFields control={control} register={register} errors={formState.errors} />
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-wrap gap-x-5 gap-y-2">
          <Checkbox label={t("domains.https")} {...register("https")} />
          <Checkbox label={t("domains.protect")} {...register("protect")} />
        </div>
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
