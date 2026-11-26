import { useForm, useWatch } from "react-hook-form";

import { useTranslation } from "../i18n";
import { errorMessage } from "../lib/errors";
import { Button, InlineAlert, SelectField, TextField } from "../ui";

import { useRemovePolicy, useSavePolicy, type BackupPolicy, type BackupTarget } from "./api";

const PRESET_NAMES: Record<string, string> = {
  "0 3 * * *": "daily",
  "0 */6 * * *": "sixHours",
  "0 * * * *": "hourly",
  "0 3 * * 0": "weekly",
};
const PRESETS = Object.keys(PRESET_NAMES);
const CUSTOM = "custom";
const MAX_RETENTION = 365;

interface FormValues {
  destinationId: string;
  preset: string;
  schedule: string;
  retention: string;
  enabled: boolean;
}

interface Option {
  value: string;
  label: string;
}

function defaults(policy: BackupPolicy | null, destinations: Option[]): FormValues {
  const schedule = policy?.schedule ?? "0 3 * * *";
  return {
    destinationId: policy?.destination_id ?? destinations[0]?.value ?? "",
    preset: PRESETS.includes(schedule) ? schedule : CUSTOM,
    schedule,
    retention: String(policy?.retention ?? 7),
    enabled: policy?.enabled ?? true,
  };
}

interface PolicyFormProps {
  target: BackupTarget;
  policy: BackupPolicy | null;
  destinations: Option[];
}

export function PolicyForm({ target, policy, destinations }: PolicyFormProps) {
  const { t, i18n } = useTranslation("common");
  const save = useSavePolicy(target);
  const remove = useRemovePolicy(target);
  const form = useForm<FormValues>({ defaultValues: defaults(policy, destinations) });
  const preset = useWatch({ control: form.control, name: "preset" });
  const next = policy ? new Intl.DateTimeFormat(i18n.language, { dateStyle: "medium", timeStyle: "short" }).format(new Date(policy.next_run_at)) : null;

  const onSubmit = form.handleSubmit((values) =>
    save.mutate({
      destination_id: values.destinationId,
      schedule: values.preset === CUSTOM ? values.schedule : values.preset,
      retention: Number(values.retention),
      enabled: values.enabled,
    }),
  );

  return (
    <form onSubmit={onSubmit} noValidate className="flex flex-col gap-4 p-4">
      {save.isError ? <InlineAlert tone="danger">{errorMessage(save.error, t)}</InlineAlert> : null}
      <div className="grid gap-4 md:grid-cols-3">
        <SelectField label={t("backups.destination")} options={destinations} {...form.register("destinationId")} />
        <SelectField
          label={t("backups.schedule")}
          options={[...PRESETS.map((value) => ({ value, label: t(`backups.presets.${PRESET_NAMES[value] ?? "custom"}`) })), { value: CUSTOM, label: t("backups.presets.custom") }]}
          {...form.register("preset")}
        />
        <TextField
          label={t("backups.retention")}
          inputMode="numeric"
          error={form.formState.errors.retention ? t("backups.retentionInvalid", { max: MAX_RETENTION }) : undefined}
          {...form.register("retention", { validate: (value) => Number.isInteger(Number(value)) && Number(value) >= 1 && Number(value) <= MAX_RETENTION })}
        />
      </div>
      {preset === CUSTOM ? (
        <TextField label={t("backups.cron")} hint={t("backups.cronHint")} spellCheck={false} className="max-w-sm" {...form.register("schedule", { required: true })} />
      ) : null}
      <div className="flex flex-wrap items-center justify-between gap-3">
        <label className="inline-flex items-center gap-2 text-text">
          <input type="checkbox" className="size-4 accent-current" {...form.register("enabled")} />
          {t("backups.enabled")}
        </label>
        <div className="flex flex-wrap items-center gap-2">
          {next && policy?.enabled ? <span className="text-[13px] text-subtle">{t("backups.nextRun", { time: next })}</span> : null}
          {policy ? (
            <Button variant="ghost" busy={remove.isPending} onClick={() => remove.mutate()}>
              {t("backups.removeSchedule")}
            </Button>
          ) : null}
          <Button type="submit" busy={save.isPending} disabled={!form.formState.isDirty && policy !== null}>
            {t("backups.saveSchedule")}
          </Button>
        </div>
      </div>
    </form>
  );
}
