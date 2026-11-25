import { useState } from "react";

import { useTranslation } from "@/shared/i18n";
import { Button } from "@/shared/ui";

import { useRevokeApiKey, type ApiKey } from "./api";

function formatDate(value: string | null | undefined, locale: string, fallback: string): string {
  if (!value) {
    return fallback;
  }
  return new Intl.DateTimeFormat(locale, { dateStyle: "medium" }).format(new Date(value));
}

export function ApiKeyRow({ apiKey }: { apiKey: ApiKey }) {
  const { t, i18n } = useTranslation("settings");
  const revoke = useRevokeApiKey();
  const [confirming, setConfirming] = useState(false);

  return (
    <li className="grid gap-x-6 gap-y-1 border-b border-border px-4 py-3.5 last:border-b-0 sm:grid-cols-[2fr_1fr_1fr_1fr_auto] sm:items-center">
      <div className="min-w-0">
        <p className="truncate font-medium text-text">{apiKey.name}</p>
        <p className="font-mono text-[12px] text-subtle">{apiKey.prefix}…</p>
      </div>
      <p className="text-muted">{t(`roles.${apiKey.role}`)}</p>
      <p className="text-muted">
        {formatDate(apiKey.last_used_at, i18n.language, t("apiKeys.neverUsed"))}
      </p>
      <p className="text-muted">
        {formatDate(apiKey.expires_at, i18n.language, t("apiKeys.expiry.never"))}
      </p>
      <div className="flex justify-end gap-2">
        {confirming ? (
          <>
            <Button variant="ghost" onClick={() => setConfirming(false)}>
              {t("actions.cancel", { ns: "common" })}
            </Button>
            <Button variant="danger" busy={revoke.isPending} onClick={() => revoke.mutate(apiKey.id)}>
              {t("apiKeys.confirmRevoke")}
            </Button>
          </>
        ) : (
          <Button variant="ghost" onClick={() => setConfirming(true)}>
            {t("apiKeys.revoke")}
          </Button>
        )}
      </div>
    </li>
  );
}
