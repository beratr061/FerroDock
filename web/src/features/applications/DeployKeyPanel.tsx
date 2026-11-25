import { useState } from "react";
import { KeyIcon } from "@phosphor-icons/react";
import { useTranslation } from "react-i18next";

import { errorMessage } from "@/shared/lib/errors";
import { Button, CopyButton } from "@/shared/ui";

import { useRotateDeployKey, type Application } from "./api";

export function DeployKeyPanel({ application }: { application: Application }) {
  const { t } = useTranslation("applications");
  const rotate = useRotateDeployKey(application.id);
  const [confirming, setConfirming] = useState(false);
  if (!application.deploy_key) {
    return null;
  }

  const onRotate = (): void => rotate.mutate(undefined, { onSuccess: () => setConfirming(false) });

  return (
    <section className="flex flex-col gap-3">
      <div className="flex flex-col gap-1">
        <h2 className="flex items-center gap-2 text-lg text-text">
          <KeyIcon size={18} aria-hidden />
          {t("deployKey.title")}
        </h2>
        <p className="max-w-2xl text-muted">{t("deployKey.description")}</p>
      </div>
      <div className="flex flex-col gap-3 rounded-panel border border-border bg-surface p-4">
        <code className="block overflow-x-auto rounded-control border border-border bg-surface-raised px-3 py-2 font-mono text-[12px] break-all text-text">
          {application.deploy_key}
        </code>
        {rotate.isError ? <p className="text-[13px] text-danger">{errorMessage(rotate.error, t)}</p> : null}
        <div className="flex flex-wrap gap-2">
          <CopyButton value={application.deploy_key} />
          {confirming ? (
            <>
              <Button variant="ghost" onClick={() => setConfirming(false)}>
                {t("actions.cancel", { ns: "common" })}
              </Button>
              <Button variant="danger" busy={rotate.isPending} onClick={onRotate}>
                {t("deployKey.confirmRotate")}
              </Button>
            </>
          ) : (
            <Button variant="ghost" onClick={() => setConfirming(true)}>
              {t("deployKey.rotate")}
            </Button>
          )}
        </div>
      </div>
    </section>
  );
}
