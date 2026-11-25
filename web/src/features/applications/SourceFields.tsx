import type { UseFormReturn } from "react-hook-form";

import { useTranslation } from "@/shared/i18n";
import { SegmentedControl, SelectField, TextField } from "@/shared/ui";

import type { SourceFormValues, SourceKind } from "./sourceForm";

type SourceForm = UseFormReturn<SourceFormValues & { name: string; description: string }>;

export function SourceFields({ form }: { form: SourceForm }) {
  const { t } = useTranslation("applications");
  const { register, watch, setValue, formState } = form;
  const kind = watch("kind");
  const buildKind = watch("buildKind");
  const required = t("validation.required", { ns: "auth" });

  return (
    <div className="flex flex-col gap-4">
      <SegmentedControl<SourceKind>
        label={t("source.label")}
        value={kind}
        segments={[
          { value: "git", label: t("source.git") },
          { value: "image", label: t("source.image") },
        ]}
        onChange={(value) => setValue("kind", value, { shouldDirty: true })}
      />
      {kind === "image" ? (
        <TextField
          label={t("fields.image")}
          placeholder="ghcr.io/acme/web:1.4.2"
          spellCheck={false}
          error={formState.errors.image ? required : undefined}
          {...register("image", { required: true })}
        />
      ) : (
        <div className="grid gap-4 md:grid-cols-[2fr_1fr]">
          <TextField
            label={t("fields.url")}
            placeholder="git@github.com:acme/web.git"
            hint={t("fields.urlHint")}
            spellCheck={false}
            error={formState.errors.url ? required : undefined}
            {...register("url", { required: true })}
          />
          <TextField label={t("fields.branch")} placeholder="main" spellCheck={false} {...register("branch")} />
          <SelectField
            label={t("fields.buildKind")}
            options={[
              { value: "dockerfile", label: t("build.dockerfile") },
              { value: "compose", label: t("build.compose") },
            ]}
            {...register("buildKind")}
          />
          <TextField
            label={t("fields.buildPath")}
            placeholder={buildKind === "compose" ? "docker-compose.yml" : "Dockerfile"}
            spellCheck={false}
            {...register("buildPath")}
          />
        </div>
      )}
      <SelectField
        label={t("fields.strategy")}
        className="md:max-w-xs"
        options={[
          { value: "recreate", label: t("strategy.recreate") },
          { value: "blue_green", label: t("strategy.blue_green") },
        ]}
        {...register("strategy")}
      />
    </div>
  );
}
