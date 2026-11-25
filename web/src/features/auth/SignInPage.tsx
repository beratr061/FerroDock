import { useNavigate } from "@tanstack/react-router";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";

import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert, TextField } from "@/shared/ui";

import { useSignIn } from "./api";
import { AuthLayout } from "./AuthLayout";

interface SignInValues {
  email: string;
  password: string;
}

export function SignInPage() {
  const { t } = useTranslation("auth");
  const navigate = useNavigate();
  const signIn = useSignIn();
  const { register, handleSubmit, formState } = useForm<SignInValues>();

  const onSubmit = handleSubmit((values) =>
    signIn.mutate(values, { onSuccess: () => void navigate({ to: "/projects" }) }),
  );

  return (
    <AuthLayout title={t("signIn.title")} description={t("signIn.description")}>
      <form className="flex flex-col gap-4" onSubmit={onSubmit} noValidate>
        {signIn.isError ? <InlineAlert tone="danger">{errorMessage(signIn.error, t)}</InlineAlert> : null}
        <TextField
          label={t("fields.email")}
          type="email"
          autoComplete="username"
          autoFocus
          error={formState.errors.email ? t("validation.required") : undefined}
          {...register("email", { required: true })}
        />
        <TextField
          label={t("fields.password")}
          type="password"
          autoComplete="current-password"
          error={formState.errors.password ? t("validation.required") : undefined}
          {...register("password", { required: true })}
        />
        <Button type="submit" busy={signIn.isPending} className="mt-2 w-full">
          {signIn.isPending ? t("signIn.submitting") : t("signIn.submit")}
        </Button>
      </form>
    </AuthLayout>
  );
}
