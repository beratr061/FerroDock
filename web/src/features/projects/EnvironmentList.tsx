import { useState } from "react";
import { PlusIcon } from "@phosphor-icons/react";
import { useForm } from "react-hook-form";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert, TextField } from "@/shared/ui";

import { useAddEnvironment, useRemoveEnvironment, type Environment, type Project } from "./api";

function EnvironmentRow({ projectId, environment, removable }: {
  projectId: string;
  environment: Environment;
  removable: boolean;
}) {
  const { t } = useTranslation("projects");
  const remove = useRemoveEnvironment(projectId);
  const [confirming, setConfirming] = useState(false);

  return (
    <li className="flex flex-wrap items-center justify-between gap-3 border-b border-border px-4 py-3 last:border-b-0">
      <div className="min-w-0">
        <p className="truncate font-medium text-text">{environment.name}</p>
        <p className="truncate font-mono text-[12px] text-subtle">{environment.slug}</p>
        {remove.isError ? <p className="text-[13px] text-danger">{errorMessage(remove.error, t)}</p> : null}
      </div>
      {removable && confirming ? (
        <div className="flex gap-2">
          <Button variant="ghost" onClick={() => setConfirming(false)}>
            {t("actions.cancel", { ns: "common" })}
          </Button>
          <Button variant="danger" busy={remove.isPending} onClick={() => remove.mutate(environment.id)}>
            {t("environments.confirmRemove")}
          </Button>
        </div>
      ) : null}
      {removable && !confirming ? (
        <Button variant="ghost" onClick={() => setConfirming(true)}>
          {t("environments.remove")}
        </Button>
      ) : null}
    </li>
  );
}

function AddEnvironmentForm({ projectId, onDone }: { projectId: string; onDone: () => void }) {
  const { t } = useTranslation("projects");
  const add = useAddEnvironment(projectId);
  const { register, handleSubmit, formState } = useForm<{ name: string }>();
  const onSubmit = handleSubmit((values) => add.mutate(values.name, { onSuccess: onDone }));

  return (
    <form onSubmit={onSubmit} noValidate className="enter-settle flex flex-col gap-3 border-t border-border p-4">
      {add.isError ? <InlineAlert tone="danger">{errorMessage(add.error, t)}</InlineAlert> : null}
      <div className="flex flex-wrap items-end gap-2">
        <TextField
          className="min-w-48 flex-1"
          label={t("environments.name")}
          placeholder={t("environments.namePlaceholder")}
          autoFocus
          error={formState.errors.name ? t("validation.required", { ns: "auth" }) : undefined}
          {...register("name", { required: true, maxLength: 100 })}
        />
        <Button variant="ghost" onClick={onDone}>
          {t("actions.cancel", { ns: "common" })}
        </Button>
        <Button type="submit" busy={add.isPending}>
          {t("environments.add")}
        </Button>
      </div>
    </form>
  );
}

export function EnvironmentList({ project }: { project: Project }) {
  const { t } = useTranslation("projects");
  const [adding, setAdding] = useState(false);
  const removable = project.environments.length > 1;

  return (
    <section className="flex flex-col gap-3">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-col gap-1">
          <h2 className="text-lg text-text">{t("environments.title")}</h2>
          <p className="text-muted">{t("environments.description")}</p>
        </div>
        {adding ? null : (
          <Button
            variant="secondary"
            icon={<PlusIcon size={16} weight="bold" aria-hidden />}
            onClick={() => setAdding(true)}
          >
            {t("environments.new")}
          </Button>
        )}
      </header>
      <div className="overflow-hidden rounded-panel border border-border bg-surface">
        <ul>
          {project.environments.map((environment) => (
            <EnvironmentRow
              key={environment.id}
              projectId={project.id}
              environment={environment}
              removable={removable}
            />
          ))}
        </ul>
        {adding ? <AddEnvironmentForm projectId={project.id} onDone={() => setAdding(false)} /> : null}
      </div>
    </section>
  );
}
