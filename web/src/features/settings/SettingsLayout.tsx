import { Link, Outlet } from "@tanstack/react-router";
import { useTranslation } from "@/shared/i18n";

const SECTIONS = [
  { to: "/settings/api-keys", key: "apiKeys" },
  { to: "/settings/appearance", key: "appearance" },
  { to: "/settings/backups", key: "backups" },
  { to: "/settings/updates", key: "updates" },
] as const;

export function SettingsLayout() {
  const { t } = useTranslation("settings");
  return (
    <div className="flex flex-col gap-6">
      <h1 className="text-xl text-text">{t("title")}</h1>
      <div className="grid gap-8 md:grid-cols-[180px_minmax(0,1fr)]">
        <nav aria-label={t("title")} className="flex gap-1 overflow-x-auto md:flex-col">
          {SECTIONS.map((section) => (
            <Link
              key={section.to}
              to={section.to}
              className="press rounded-control px-3 py-2 whitespace-nowrap text-muted hover:bg-surface-hover hover:text-text"
              activeProps={{ className: "bg-surface-raised text-text font-medium" }}
            >
              {t(`${section.key}.title`)}
            </Link>
          ))}
        </nav>
        <Outlet />
      </div>
    </div>
  );
}
