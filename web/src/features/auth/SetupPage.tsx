import { valibotResolver } from "@hookform/resolvers/valibot";
import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useForm } from "react-hook-form";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert, TextField } from "@/shared/ui";

import { setupStatusQuery, useCompleteSetup } from "./api";
import { AuthLayout } from "./AuthLayout";
import { PASSWORD_MIN, setupSchema, type SetupValues } from "./setupSchema";

export function SetupPage() {
  const { t, i18n } = useTranslation("auth");
  const navigate = useNavigate();
  const setup = useCompleteSetup();
  const status = useQuery(setupStatusQuery);
  const tokenRequired = status.data?.token_required === true;
  const { register, handleSubmit, formState } = useForm<SetupValues>({
    resolver: valibotResolver(setupSchema),
    mode: "onTouched",
  });
  const fieldError = (name: keyof SetupValues): string | undefined => {
    const message = formState.errors[name]?.message;
    return message ? t(message, { min: PASSWORD_MIN }) : undefined;
  };

  const onSubmit = handleSubmit((values) =>
    setup.mutate(
      {
        email: values.email,
        name: values.name,
        password: values.password,
        organization_name: values.organizationName,
        locale: i18n.language,
        setup_token: tokenRequired ? (values.setupToken ?? "") : null,
      },
      { onSuccess: () => void navigate({ to: "/projects" }) },
    ),
  );

  return (
    <AuthLayout title={t("setup.title")} description={t("setup.description")}>
      <form className="flex flex-col gap-4" onSubmit={onSubmit} noValidate>
        {setup.isError ? <InlineAlert tone="danger">{errorMessage(setup.error, t)}</InlineAlert> : null}
        {tokenRequired ? (
          <TextField
            label={t("fields.setupToken")}
            hint={t("setup.tokenHint")}
            autoComplete="off"
            spellCheck={false}
            autoFocus
            {...register("setupToken")}
          />
        ) : null}
        <TextField label={t("fields.name")} autoComplete="name" autoFocus={!tokenRequired} error={fieldError("name")} {...register("name")} />
        <TextField label={t("fields.email")} type="email" autoComplete="email" error={fieldError("email")} {...register("email")} />
        <TextField
          label={t("fields.organization")}
          hint={t("setup.organizationHint")}
          error={fieldError("organizationName")}
          {...register("organizationName")}
        />
        <TextField
          label={t("fields.password")}
          type="password"
          autoComplete="new-password"
          hint={t("validation.passwordMin", { min: PASSWORD_MIN })}
          error={fieldError("password")}
          {...register("password")}
        />
        <Button type="submit" busy={setup.isPending} className="mt-2 w-full">
          {setup.isPending ? t("setup.submitting") : t("setup.submit")}
        </Button>
      </form>
    </AuthLayout>
  );
}
