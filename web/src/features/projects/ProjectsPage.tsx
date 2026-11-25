import { CubeIcon, PlusIcon } from "@phosphor-icons/react";
import { useTranslation } from "react-i18next";

import { Button, EmptyState } from "@/shared/ui";

export function ProjectsPage() {
  const { t } = useTranslation("projects");
  return (
    <div className="flex flex-col gap-6">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-col gap-1">
          <h1 className="text-xl text-text">{t("title")}</h1>
          <p className="text-muted">{t("subtitle")}</p>
        </div>
        <Button icon={<PlusIcon size={16} weight="bold" aria-hidden />} disabled title={t("comingSoon")}>
          {t("create")}
        </Button>
      </header>
      <EmptyState
        icon={<CubeIcon size={22} aria-hidden />}
        title={t("empty.title")}
        description={t("empty.description")}
      />
    </div>
  );
}
