import { useState } from "react";
import { GlobeIcon, PlusIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert } from "@/shared/ui";

import { AddDomainForm } from "./AddDomainForm";
import { DomainRow } from "./DomainRow";
import { domainsQuery } from "./domains";

export function DomainsPanel({ applicationId }: { applicationId: string }) {
  const { t } = useTranslation("applications");
  const domains = useQuery(domainsQuery(applicationId));
  const [adding, setAdding] = useState(false);
  const list = domains.data?.domains ?? [];

  return (
    <section className="flex flex-col gap-3">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-col gap-1">
          <h2 className="flex items-center gap-2 text-lg text-text">
            <GlobeIcon size={18} aria-hidden />
            {t("domains.title")}
          </h2>
          <p className="max-w-2xl text-muted">{t("domains.description")}</p>
        </div>
        {adding ? null : (
          <Button variant="secondary" icon={<PlusIcon size={16} weight="bold" aria-hidden />} onClick={() => setAdding(true)}>
            {t("domains.new")}
          </Button>
        )}
      </header>
      {domains.data?.proxy_enabled === false ? <InlineAlert tone="warning">{t("domains.proxyDisabled")}</InlineAlert> : null}
      {domains.isError ? <InlineAlert tone="danger">{errorMessage(domains.error, t)}</InlineAlert> : null}
      <div className="overflow-hidden rounded-panel border border-border bg-surface">
        {adding ? <AddDomainForm applicationId={applicationId} onDone={() => setAdding(false)} /> : null}
        {domains.data && list.length === 0 && !adding ? <p className="px-4 py-6 text-muted">{t("domains.empty")}</p> : null}
        <ul>
          {list.map((domain) => (
            <DomainRow key={domain.id} applicationId={applicationId} domain={domain} />
          ))}
        </ul>
      </div>
    </section>
  );
}
