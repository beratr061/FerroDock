import { useState } from "react";
import { useForm } from "react-hook-form";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, CopyButton, InlineAlert, TextField } from "@/shared/ui";

import { useExportMasterKey } from "./api";

export function MasterKeyPanel() {
  const { t } = useTranslation("settings");
  const exporter = useExportMasterKey();
  const [key, setKey] = useState<string | null>(null);
  const { register, handleSubmit, reset, formState } = useForm<{ password: string }>();

  const onSubmit = handleSubmit((values) =>
    exporter.mutate(values.password, {
      onSuccess: (result) => {
        setKey(result.master_key);
        reset();
      },
    }),
  );

  return (
    <section className="flex flex-col gap-3">
      <div className="flex flex-col gap-1">
        <h3 className="font-medium text-text">{t("backups.masterKey.title")}</h3>
        <p className="max-w-2xl text-muted">{t("backups.masterKey.description")}</p>
      </div>
      {key ? (
        <InlineAlert tone="warning">
          <div className="flex flex-col gap-3">
            <p className="text-text">{t("backups.masterKey.revealed")}</p>
            <code className="block overflow-x-auto rounded-control border border-border bg-surface px-3 py-2 font-mono text-[13px] break-all text-text">
              {key}
            </code>
            <div className="flex gap-2">
              <CopyButton value={key} />
              <Button variant="ghost" onClick={() => setKey(null)}>
                {t("backups.masterKey.hide")}
              </Button>
            </div>
          </div>
        </InlineAlert>
      ) : (
        <form onSubmit={onSubmit} noValidate className="flex flex-wrap items-end gap-2">
          <TextField
            className="min-w-60 flex-1 md:max-w-sm"
            type="password"
            autoComplete="current-password"
            label={t("backups.masterKey.password")}
            error={formState.errors.password ? t("validation.required", { ns: "auth" }) : undefined}
            {...register("password", { required: true })}
          />
          <Button type="submit" variant="secondary" busy={exporter.isPending}>
            {t("backups.masterKey.reveal")}
          </Button>
        </form>
      )}
      {exporter.isError ? <InlineAlert tone="danger">{errorMessage(exporter.error, t)}</InlineAlert> : null}
    </section>
  );
}
