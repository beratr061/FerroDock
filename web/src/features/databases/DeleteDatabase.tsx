import { useState } from "react";
import { useNavigate } from "@tanstack/react-router";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button } from "@/shared/ui";

import { useDeleteDatabase, type Database } from "./api";

export function DeleteDatabase({ database }: { database: Database }) {
  const { t } = useTranslation("databases");
  const remove = useDeleteDatabase(database.id);
  const navigate = useNavigate();
  const [confirming, setConfirming] = useState(false);
  const [removeData, setRemoveData] = useState(false);
  const onDelete = (): void =>
    remove.mutate(removeData, {
      onSuccess: () => void navigate({ to: "/projects/$projectId", params: { projectId: database.project_id } }),
    });

  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-lg text-text">{t("settings")}</h2>
      <div className="flex flex-col gap-3 rounded-panel border border-border bg-surface p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex max-w-xl flex-col gap-1">
            <p className="font-medium text-text">{t("delete.title")}</p>
            <p className="text-muted">{t("delete.description")}</p>
          </div>
          {confirming ? null : (
            <Button variant="danger" onClick={() => setConfirming(true)}>
              {t("delete.action")}
            </Button>
          )}
        </div>
        {confirming ? (
          <div className="flex flex-wrap items-center justify-between gap-3 border-t border-border pt-3">
            <label className="inline-flex items-center gap-2 text-text">
              <input
                type="checkbox"
                className="size-4 accent-current"
                checked={removeData}
                onChange={(event) => setRemoveData(event.target.checked)}
              />
              {t("delete.removeData")}
            </label>
            <div className="flex gap-2">
              <Button variant="ghost" onClick={() => setConfirming(false)}>
                {t("actions.cancel", { ns: "common" })}
              </Button>
              <Button variant="danger" busy={remove.isPending} onClick={onDelete}>
                {removeData ? t("delete.confirmWithData") : t("delete.confirm")}
              </Button>
            </div>
          </div>
        ) : null}
        {remove.isError ? <p className="text-[13px] text-danger">{errorMessage(remove.error, t)}</p> : null}
      </div>
    </section>
  );
}
