
import { useTranslation } from "@/shared/i18n";
import { LanguageSelect, ThemeSwitcher } from "@/shared/ui";

export function AppearancePage() {
  const { t } = useTranslation("settings");
  return (
    <section className="flex max-w-xl flex-col gap-8">
      <div className="flex flex-col gap-1">
        <h2 className="text-lg text-text">{t("appearance.title")}</h2>
        <p className="text-muted">{t("appearance.description")}</p>
      </div>
      <div className="flex flex-col gap-2">
        <p className="font-medium text-text">{t("appearance.theme")}</p>
        <ThemeSwitcher />
      </div>
      <LanguageSelect className="max-w-xs" />
    </section>
  );
}
