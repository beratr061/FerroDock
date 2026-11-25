import { useState } from "react";
import { ArrowSquareOutIcon, LockSimpleIcon } from "@phosphor-icons/react";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, StatusBadge, type StatusTone } from "@/shared/ui";

import { useCheckDns, useRemoveDomain, type DnsVerdict, type Domain } from "./api";

const VERDICT_TONES: Record<DnsVerdict, StatusTone> = {
  points_here: "success",
  proxied: "info",
  unverified: "neutral",
  points_elsewhere: "warning",
  unresolved: "danger",
};

function address(domain: Domain): string {
  const scheme = domain.https ? "https" : "http";
  const path = domain.path_prefix === "/" ? "" : domain.path_prefix;
  return `${scheme}://${domain.host}${path}`;
}

export function DomainRow({ applicationId, domain }: { applicationId: string; domain: Domain }) {
  const { t } = useTranslation("applications");
  const remove = useRemoveDomain(applicationId);
  const dns = useCheckDns(applicationId);
  const [confirming, setConfirming] = useState(false);
  const failure = remove.error ?? dns.error;

  return (
    <li className="flex flex-col gap-2 border-b border-border px-4 py-3 last:border-b-0">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="min-w-0">
          <a href={address(domain)} target="_blank" rel="noreferrer noopener" className="inline-flex max-w-full items-center gap-1.5 truncate font-medium text-text hover:underline">
            {domain.https ? <LockSimpleIcon size={14} aria-hidden /> : null}
            {address(domain)}
            <ArrowSquareOutIcon size={14} aria-hidden className="text-subtle" />
          </a>
          <p className="font-mono text-[12px] text-subtle">{t("domains.portLabel", { port: domain.container_port })}</p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button variant="ghost" busy={dns.isPending} onClick={() => dns.mutate(domain.id)}>
            {t("domains.checkDns")}
          </Button>
          {confirming ? (
            <Button variant="danger" busy={remove.isPending} onClick={() => remove.mutate(domain.id)}>
              {t("domains.confirmRemove")}
            </Button>
          ) : (
            <Button variant="ghost" onClick={() => setConfirming(true)}>
              {t("domains.remove")}
            </Button>
          )}
        </div>
      </div>
      {dns.data ? (
        <div className="enter-settle flex flex-wrap items-center gap-2 text-[13px] text-muted">
          <StatusBadge tone={VERDICT_TONES[dns.data.verdict]} label={t(`domains.verdict.${dns.data.verdict}`)} />
          <span>{t(`domains.verdictHint.${dns.data.verdict}`)}</span>
          {dns.data.addresses.length > 0 ? (
            <span className="font-mono text-[12px] text-subtle">{dns.data.addresses.join(", ")}</span>
          ) : null}
        </div>
      ) : null}
      {failure ? <p className="text-[13px] text-danger">{errorMessage(failure, t)}</p> : null}
    </li>
  );
}
