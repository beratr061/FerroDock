import { Link } from "@tanstack/react-router";

import { useTranslation } from "@/shared/i18n";
import { NAV_ITEMS } from "./navigation";

export function BottomNav() {
  const { t } = useTranslation();
  return (
    <nav
      aria-label={t("nav.label")}
      className="fixed inset-x-0 bottom-0 z-20 grid grid-cols-3 border-t border-border bg-surface pb-[env(safe-area-inset-bottom)] md:hidden"
    >
      {NAV_ITEMS.map(({ to, key, Icon }) => (
        <Link
          key={to}
          to={to}
          className="press flex h-14 flex-col items-center justify-center gap-0.5 text-[12px] text-muted"
          activeProps={{ className: "text-text font-medium" }}
        >
          <Icon aria-hidden size={22} />
          {t(`nav.${key}`)}
        </Link>
      ))}
    </nav>
  );
}
