import { useWatch, type Control, type FieldErrors, type UseFormRegister } from "react-hook-form";

import { useTranslation } from "@/shared/i18n";
import { TextField } from "@/shared/ui";

import type { DomainFormValues } from "./formValues";

interface BasicAuthFieldsProps {
  control: Control<DomainFormValues>;
  register: UseFormRegister<DomainFormValues>;
  errors: FieldErrors<DomainFormValues>;
}

const USERNAME_PATTERN = /^[^\s:]{1,64}$/;

export function BasicAuthFields({ control, register, errors }: BasicAuthFieldsProps) {
  const { t } = useTranslation("applications");
  const protect = useWatch({ control, name: "protect" });
  if (!protect) {
    return null;
  }
  return (
    <div className="enter-settle grid gap-4 md:grid-cols-2">
      <TextField
        label={t("domains.authUser")}
        autoComplete="off"
        spellCheck={false}
        error={errors.authUser ? t("domains.authUserInvalid") : undefined}
        {...register("authUser", { required: true, pattern: USERNAME_PATTERN })}
      />
      <TextField
        label={t("domains.authPassword")}
        type="password"
        autoComplete="new-password"
        hint={t("domains.authPasswordHint")}
        error={errors.authPassword ? t("domains.authPasswordHint") : undefined}
        {...register("authPassword", { required: true, minLength: 8, maxLength: 72 })}
      />
    </div>
  );
}
