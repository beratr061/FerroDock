import { TrashIcon } from "@phosphor-icons/react";
import { useWatch, type Control, type FieldErrors, type UseFormRegister } from "react-hook-form";

import { useTranslation } from "@/shared/i18n";
import { Button, SelectField, TextField } from "@/shared/ui";

import type { RuntimeForm } from "./runtimeForm";

interface MountFieldsProps {
  index: number;
  control: Control<RuntimeForm>;
  register: UseFormRegister<RuntimeForm>;
  errors: FieldErrors<RuntimeForm>;
  onRemove: () => void;
}

export function MountFields({ index, control, register, errors, onRemove }: MountFieldsProps) {
  const { t } = useTranslation("applications");
  const kind = useWatch({ control, name: `mounts.${index}.kind` });
  const failed = errors.mounts?.[index];
  const required = t("validation.required", { ns: "auth" });

  return (
    <li className="grid gap-3 border-b border-border px-4 py-3 last:border-b-0 md:grid-cols-[9rem_1fr_1fr_auto] md:items-end">
      <SelectField
        label={t("runtime.kind")}
        options={[
          { value: "volume", label: t("runtime.volume") },
          { value: "bind", label: t("runtime.bind") },
        ]}
        {...register(`mounts.${index}.kind`)}
      />
      <TextField
        label={kind === "bind" ? t("runtime.hostPath") : t("runtime.volumeName")}
        placeholder={kind === "bind" ? "/srv/uploads" : "data"}
        spellCheck={false}
        error={failed?.source ? required : undefined}
        {...register(`mounts.${index}.source`, { required: true })}
      />
      <TextField
        label={t("runtime.containerPath")}
        placeholder="/var/lib/data"
        spellCheck={false}
        error={failed?.target ? required : undefined}
        {...register(`mounts.${index}.target`, { required: true })}
      />
      <div className="flex items-center gap-3 md:h-10">
        <label className="inline-flex items-center gap-2 text-text">
          <input type="checkbox" className="size-4 accent-current" {...register(`mounts.${index}.readOnly`)} />
          {t("runtime.readOnly")}
        </label>
        <Button variant="ghost" aria-label={t("runtime.remove")} icon={<TrashIcon size={16} aria-hidden />} onClick={onRemove} />
      </div>
    </li>
  );
}
