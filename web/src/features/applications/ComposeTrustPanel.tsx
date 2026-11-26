import { useState } from "react";
import { ShieldWarningIcon } from "@phosphor-icons/react";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert } from "@/shared/ui";

import { useSetComposeTrust, type Application } from "./api";

export function ComposeTrustPanel({ application }: { application: Application }) {
  const { t } = useTranslation("applications");
  const trust = useSetComposeTrust(application.id);
  const [confirming, setConfirming] = useState(false);
  const trusted = application.compose_trusted;

  return (
    <section className="flex flex-col gap-3">
      <header className="flex flex-col gap-1">
        <h2 className="flex items-center gap-2 text-lg text-text">
          <ShieldWarningIcon size={18} aria-hidden />
          {t("composeTrust.title")}
        </h2>
        <p className="max-w-2xl text-muted">{t("composeTrust.description")}</p>
      </header>
      <div className="flex flex-col gap-3 rounded-panel border border-border bg-surface p-4">
        {trusted ? <InlineAlert tone="warning">{t("composeTrust.active")}</InlineAlert> : null}
        {trust.isError ? <InlineAlert tone="danger">{errorMessage(trust.error, t)}</InlineAlert> : null}
        <div className="flex flex-wrap items-center gap-2">
          {trusted ? (
            <Button variant="secondary" busy={trust.isPending} onClick={() => trust.mutate(false)}>
              {t("composeTrust.revoke")}
            </Button>
          ) : null}
          {!trusted && !confirming ? (
            <Button variant="ghost" onClick={() => setConfirming(true)}>
              {t("composeTrust.grant")}
            </Button>
          ) : null}
          {!trusted && confirming ? (
            <>
              <Button variant="ghost" onClick={() => setConfirming(false)}>
                {t("actions.cancel", { ns: "common" })}
              </Button>
              <Button variant="danger" busy={trust.isPending} onClick={() => trust.mutate(true, { onSuccess: () => setConfirming(false) })}>
                {t("composeTrust.confirm")}
              </Button>
            </>
          ) : null}
        </div>
      </div>
    </section>
  );
}
