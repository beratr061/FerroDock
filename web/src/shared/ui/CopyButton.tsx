import { useState } from "react";
import { CheckIcon, CopyIcon } from "@phosphor-icons/react";
import { useTranslation } from "react-i18next";

import { Button } from "./Button";

export function CopyButton({ value }: { value: string }) {
  const { t } = useTranslation("common");
  const [copied, setCopied] = useState(false);

  const copy = async (): Promise<void> => {
    await navigator.clipboard.writeText(value);
    setCopied(true);
  };

  return (
    <Button
      variant="secondary"
      onClick={() => void copy()}
      icon={copied ? <CheckIcon size={16} weight="bold" aria-hidden /> : <CopyIcon size={16} aria-hidden />}
    >
      {copied ? t("actions.copied") : t("actions.copy")}
    </Button>
  );
}
