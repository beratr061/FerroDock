import { TrashIcon } from "@phosphor-icons/react";
import type { FieldErrors, UseFormRegister } from "react-hook-form";

import { useTranslation } from "@/shared/i18n";
import { Button, SelectField, TextField } from "@/shared/ui";

import { validPort, type RuntimeForm } from "./runtimeForm";

interface PortFieldsProps {
  index: number;
  register: UseFormRegister<RuntimeForm>;
  errors: FieldErrors<RuntimeForm>;
  onRemove: () => void;
}

export function PortFields({ index, register, errors, onRemove }: PortFieldsProps) {
  const { t } = useTranslation("applications");
  const failed = errors.ports?.[index];
  const invalid = t("domains.portInvalid");

  return (
    <li className="grid gap-3 border-b border-border px-4 py-3 last:border-b-0 md:grid-cols-[1fr_1fr_8rem_auto] md:items-end">
      <TextField
        label={t("runtime.publishedPort")}
        inputMode="numeric"
        placeholder="5432"
        error={failed?.published ? invalid : undefined}
        {...register(`ports.${index}.published`, { validate: validPort })}
      />
      <TextField
        label={t("runtime.targetPort")}
        inputMode="numeric"
        placeholder="5432"
        error={failed?.target ? invalid : undefined}
        {...register(`ports.${index}.target`, { validate: validPort })}
      />
      <SelectField
        label={t("runtime.protocol")}
        options={[
          { value: "tcp", label: "TCP" },
          { value: "udp", label: "UDP" },
        ]}
        {...register(`ports.${index}.protocol`)}
      />
      <div className="flex items-center md:h-10">
        <Button variant="ghost" aria-label={t("runtime.remove")} icon={<TrashIcon size={16} aria-hidden />} onClick={onRemove} />
      </div>
    </li>
  );
}
