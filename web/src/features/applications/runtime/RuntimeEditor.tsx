import { PlusIcon } from "@phosphor-icons/react";
import { useFieldArray, useForm, useWatch } from "react-hook-form";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert } from "@/shared/ui";

import { useSaveRuntime, type RuntimeConfig } from "./api";
import { MountFields } from "./MountFields";
import { PortFields } from "./PortFields";
import { NEW_MOUNT, NEW_PORT, toForm, toRequest, type RuntimeForm } from "./runtimeForm";

interface RuntimeEditorProps {
  applicationId: string;
  saved: RuntimeConfig;
  blueGreen: boolean;
}

export function RuntimeEditor({ applicationId, saved, blueGreen }: RuntimeEditorProps) {
  const { t } = useTranslation("applications");
  const save = useSaveRuntime(applicationId);
  const { control, register, handleSubmit, formState } = useForm<RuntimeForm>({ defaultValues: toForm(saved) });
  const mounts = useFieldArray({ control, name: "mounts" });
  const ports = useFieldArray({ control, name: "ports" });
  const watchedMounts = useWatch({ control, name: "mounts" });
  const sharedVolume = blueGreen && watchedMounts.some((mount) => !mount.readOnly);
  const replaces = blueGreen && ports.fields.length > 0;

  const onSubmit = handleSubmit((values) => {
    save.mutate(toRequest(values));
  });

  return (
    <form onSubmit={onSubmit} noValidate className="flex flex-col gap-4">
      {save.isError ? <InlineAlert tone="danger">{errorMessage(save.error, t)}</InlineAlert> : null}
      {sharedVolume ? <InlineAlert tone="warning">{t("runtime.sharedVolume")}</InlineAlert> : null}
      {replaces ? <InlineAlert tone="info">{t("runtime.portsRecreate")}</InlineAlert> : null}
      <div className="flex flex-col gap-2">
        <h3 className="font-medium text-text">{t("runtime.mounts")}</h3>
        <div className="overflow-hidden rounded-panel border border-border bg-surface">
          {mounts.fields.length === 0 ? <p className="px-4 py-5 text-muted">{t("runtime.noMounts")}</p> : null}
          <ul>
            {mounts.fields.map((field, index) => (
              <MountFields key={field.id} index={index} control={control} register={register} errors={formState.errors} onRemove={() => mounts.remove(index)} />
            ))}
          </ul>
        </div>
      </div>
      <div className="flex flex-col gap-2">
        <h3 className="font-medium text-text">{t("runtime.ports")}</h3>
        <div className="overflow-hidden rounded-panel border border-border bg-surface">
          {ports.fields.length === 0 ? <p className="px-4 py-5 text-muted">{t("runtime.noPorts")}</p> : null}
          <ul>
            {ports.fields.map((field, index) => (
              <PortFields key={field.id} index={index} register={register} errors={formState.errors} onRemove={() => ports.remove(index)} />
            ))}
          </ul>
        </div>
      </div>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-wrap gap-2">
          <Button variant="secondary" icon={<PlusIcon size={16} weight="bold" aria-hidden />} onClick={() => mounts.append(NEW_MOUNT)}>
            {t("runtime.addMount")}
          </Button>
          <Button variant="secondary" icon={<PlusIcon size={16} weight="bold" aria-hidden />} onClick={() => ports.append(NEW_PORT)}>
            {t("runtime.addPort")}
          </Button>
        </div>
        <Button type="submit" busy={save.isPending} disabled={!formState.isDirty}>
          {t("runtime.save")}
        </Button>
      </div>
    </form>
  );
}
