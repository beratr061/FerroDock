import { HardDrivesIcon } from "@phosphor-icons/react";
import { useTranslation } from "react-i18next";

import { EmptyState } from "@/shared/ui";

export function ServersPlaceholder() {
  const { t } = useTranslation();
  return (
    <div className="flex flex-col gap-1">
      <h1 className="text-xl text-text">{t("nav.servers")}</h1>
      <EmptyState
        icon={<HardDrivesIcon size={22} aria-hidden />}
        title={t("servers.title")}
        description={t("servers.description")}
      />
    </div>
  );
}
