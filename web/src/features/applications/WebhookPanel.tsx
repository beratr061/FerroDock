import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { GitBranchIcon } from "@phosphor-icons/react";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, CopyButton, InlineAlert } from "@/shared/ui";

import type { Application } from "./api";
import { useDisableWebhook, useRotateWebhook, webhookQuery, webhookUrl, type Webhook } from "./webhookApi";

function Value({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex flex-col gap-1.5">
      <span className="text-[13px] text-muted">{label}</span>
      <div className="flex flex-wrap items-center gap-2">
        <code className="min-w-0 flex-1 overflow-x-auto rounded-control border border-border bg-surface-raised px-3 py-2 font-mono text-[12px] break-all text-text">
          {value}
        </code>
        <CopyButton value={value} />
      </div>
    </div>
  );
}

function LastDelivery({ webhook }: { webhook: Webhook }) {
  const { t, i18n } = useTranslation("applications");
  if (!webhook.last_delivery_at || !webhook.last_outcome) {
    return <p className="text-[13px] text-subtle">{t("webhook.noDelivery")}</p>;
  }
  const when = new Intl.DateTimeFormat(i18n.language, { dateStyle: "medium", timeStyle: "short" });
  return (
    <p className="text-[13px] text-muted">
      {t("webhook.lastDelivery", {
        when: when.format(new Date(webhook.last_delivery_at)),
        outcome: t(`webhook.outcomes.${webhook.last_outcome}`, { defaultValue: webhook.last_outcome }),
      })}
    </p>
  );
}

interface ActionsProps {
  enabled: boolean;
  rotate: ReturnType<typeof useRotateWebhook>;
  disable: ReturnType<typeof useDisableWebhook>;
}

function Actions({ enabled, rotate, disable }: ActionsProps) {
  const { t } = useTranslation("applications");
  const [confirming, setConfirming] = useState(false);
  const onRotate = (): void => rotate.mutate(undefined, { onSuccess: () => setConfirming(false) });

  if (!enabled) {
    return (
      <Button className="self-start" busy={rotate.isPending} onClick={onRotate}>
        {t("webhook.enable")}
      </Button>
    );
  }
  return (
    <div className="flex flex-wrap items-center gap-2">
      {confirming ? (
        <>
          <Button variant="ghost" onClick={() => setConfirming(false)}>
            {t("actions.cancel", { ns: "common" })}
          </Button>
          <Button variant="danger" busy={rotate.isPending} onClick={onRotate}>
            {t("webhook.confirmRotate")}
          </Button>
        </>
      ) : (
        <Button variant="ghost" onClick={() => setConfirming(true)}>
          {t("webhook.rotate")}
        </Button>
      )}
      <Button variant="secondary" busy={disable.isPending} onClick={() => disable.mutate()}>
        {t("webhook.disable")}
      </Button>
    </div>
  );
}

function SecretNotice({ secret }: { secret: string }) {
  const { t } = useTranslation("applications");
  return (
    <>
      <Value label={t("webhook.secret")} value={secret} />
      <InlineAlert tone="warning">{t("webhook.secretOnce")}</InlineAlert>
      <p className="max-w-2xl text-[13px] text-muted">{t("webhook.setup")}</p>
    </>
  );
}

function useWebhookState(applicationId: string) {
  const webhook = useQuery(webhookQuery(applicationId));
  const rotate = useRotateWebhook(applicationId);
  const disable = useDisableWebhook(applicationId);
  const enabled = webhook.data?.enabled ?? false;
  return {
    webhook: enabled ? webhook.data : undefined,
    rotate,
    disable,
    enabled,
    secret: enabled ? rotate.data?.secret : undefined,
    failure: rotate.error ?? disable.error ?? webhook.error,
  };
}

function Heading({ application }: { application: Application }) {
  const { t } = useTranslation("applications");
  const branch = application.source.kind === "git" ? application.source.branch : "";
  return (
    <header className="flex flex-col gap-1">
      <h2 className="flex items-center gap-2 text-lg text-text">
        <GitBranchIcon size={18} aria-hidden />
        {t("webhook.title")}
      </h2>
      <p className="max-w-2xl text-muted">{t("webhook.description", { branch })}</p>
    </header>
  );
}

export function WebhookPanel({ application }: { application: Application }) {
  const { t } = useTranslation("applications");
  const { webhook, rotate, disable, enabled, secret, failure } = useWebhookState(application.id);

  return (
    <section className="flex flex-col gap-3">
      <Heading application={application} />
      <div className="flex flex-col gap-4 rounded-panel border border-border bg-surface p-4">
        {failure ? <InlineAlert tone="danger">{errorMessage(failure, t)}</InlineAlert> : null}
        {enabled ? <Value label={t("webhook.url")} value={webhookUrl(application.id)} /> : null}
        {secret ? <SecretNotice secret={secret} /> : null}
        {webhook && !secret ? <LastDelivery webhook={webhook} /> : null}
        <Actions enabled={enabled} rotate={rotate} disable={disable} />
      </div>
    </section>
  );
}
