
import { useTranslation } from "@/shared/i18n";
import { Button, CopyButton, InlineAlert } from "@/shared/ui";

interface CreatedKeyNoticeProps {
  secret: string;
  onDismiss: () => void;
}

export function CreatedKeyNotice({ secret, onDismiss }: CreatedKeyNoticeProps) {
  const { t } = useTranslation("settings");

  return (
    <InlineAlert tone="success">
      <div className="flex flex-col gap-3">
        <p className="font-medium">{t("apiKeys.created.title")}</p>
        <p className="text-text">{t("apiKeys.created.body")}</p>
        <code className="block overflow-x-auto rounded-control border border-border bg-surface px-3 py-2 text-[13px] text-text">
          {secret}
        </code>
        <div className="flex gap-2">
          <CopyButton value={secret} />
          <Button variant="ghost" onClick={onDismiss}>
            {t("apiKeys.created.done")}
          </Button>
        </div>
      </div>
    </InlineAlert>
  );
}
