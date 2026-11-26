import { GaugeIcon } from "@phosphor-icons/react";
import { useForm } from "react-hook-form";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert, TextField } from "@/shared/ui";

import { useSetLimits, type Database } from "./api";

const MIN_MEMORY_MB = 64;
const MAX_MEMORY_MB = 1_048_576;
const MIN_CPUS = 0.05;
const MAX_CPUS = 256;
const MILLIS_PER_CPU = 1000;

interface FormValues {
  memory: string;
  cpus: string;
}

function inRange(value: string, min: number, max: number): boolean {
  if (value.trim() === "") {
    return true;
  }
  const number = Number(value);
  return Number.isFinite(number) && number >= min && number <= max;
}

function toRequest(values: FormValues) {
  const memory = values.memory.trim();
  const cpus = values.cpus.trim();
  return {
    memory_mb: memory === "" ? null : Math.round(Number(memory)),
    cpu_millis: cpus === "" ? null : Math.round(Number(cpus) * MILLIS_PER_CPU),
  };
}

export function LimitsPanel({ database }: { database: Database }) {
  const { t } = useTranslation("databases");
  const save = useSetLimits(database.id);
  const form = useForm<FormValues>({
    defaultValues: {
      memory: database.memory_limit_mb == null ? "" : String(database.memory_limit_mb),
      cpus: database.cpu_limit_millis == null ? "" : String(database.cpu_limit_millis / MILLIS_PER_CPU),
    },
  });
  const errors = form.formState.errors;

  return (
    <section className="flex flex-col gap-3">
      <header className="flex flex-col gap-1">
        <h2 className="flex items-center gap-2 text-lg text-text">
          <GaugeIcon size={18} aria-hidden />
          {t("limits.title")}
        </h2>
        <p className="max-w-2xl text-muted">{t("limits.description")}</p>
      </header>
      <form
        onSubmit={form.handleSubmit((values) => save.mutate(toRequest(values)))}
        noValidate
        className="flex flex-col gap-3 rounded-panel border border-border bg-surface p-4"
      >
        {save.isError ? <InlineAlert tone="danger">{errorMessage(save.error, t)}</InlineAlert> : null}
        <div className="flex flex-wrap items-end gap-3">
          <TextField
            label={t("limits.memory")}
            inputMode="numeric"
            placeholder={t("limits.unlimited")}
            className="w-44"
            error={errors.memory ? t("limits.memoryInvalid", { min: MIN_MEMORY_MB }) : undefined}
            {...form.register("memory", { validate: (value) => inRange(value, MIN_MEMORY_MB, MAX_MEMORY_MB) })}
          />
          <TextField
            label={t("limits.cpus")}
            inputMode="decimal"
            placeholder={t("limits.unlimited")}
            className="w-44"
            error={errors.cpus ? t("limits.cpusInvalid", { min: MIN_CPUS, max: MAX_CPUS }) : undefined}
            {...form.register("cpus", { validate: (value) => inRange(value, MIN_CPUS, MAX_CPUS) })}
          />
          <Button type="submit" busy={save.isPending} disabled={!form.formState.isDirty}>
            {t("limits.save")}
          </Button>
        </div>
      </form>
    </section>
  );
}
