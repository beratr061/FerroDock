import { useState } from "react";
import { CheckIcon, CopyIcon } from "@phosphor-icons/react";
import { useTranslation } from "react-i18next";

import { Button, InlineAlert } from "@/shared/ui";

interface CreatedKeyNoticeProps {
  secret: string;
  onDismiss: () => void;
}

export function CreatedKeyNotice({ secret, onDismiss }: CreatedKeyNoticeProps) {
  const { t } = useTranslation("settings");
  const [copied, setCopied] = useState(false);

  const copy = async (): Promise<void> => {
    await navigator.clipboard.writeText(secret);
    setCopied(true);
  };

  return (
    <InlineAlert tone="success">
      <div className="flex flex-col gap-3">
        <p className="font-medium">{t("apiKeys.created.title")}</p>
        <p className="text-text">{t("apiKeys.created.body")}</p>
        <code className="block overflow-x-auto rounded-control border border-border bg-surface px-3 py-2 text-[13px] text-text">
          {secret}
        </code>
        <div className="flex gap-2">
          <Button
            variant="secondary"
            onClick={() => void copy()}
            icon={copied ? <CheckIcon size={16} weight="bold" aria-hidden /> : <CopyIcon size={16} aria-hidden />}
          >
            {copied ? t("apiKeys.created.copied") : t("apiKeys.created.copy")}
          </Button>
          <Button variant="ghost" onClick={onDismiss}>
            {t("apiKeys.created.done")}
          </Button>
        </div>
      </div>
    </InlineAlert>
  );
}
