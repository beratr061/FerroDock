import { Link } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";

import { Logo } from "@/shared/ui";

import { NAV_ITEMS } from "./navigation";
import { UserMenu } from "./UserMenu";

export function Sidebar() {
  const { t } = useTranslation();
  return (
    <aside className="sticky top-0 hidden h-dvh w-16 shrink-0 flex-col border-r border-border bg-surface md:flex lg:w-60">
      <div className="flex h-14 items-center px-4">
        <span className="lg:hidden">
          <Logo compact />
        </span>
        <span className="hidden lg:block">
          <Logo />
        </span>
      </div>
      <nav aria-label={t("nav.label")} className="flex flex-1 flex-col gap-1 px-2.5 py-3">
        {NAV_ITEMS.map(({ to, key, Icon }) => (
          <Link
            key={to}
            to={to}
            title={t(`nav.${key}`)}
            className="press flex h-10 items-center gap-3 rounded-control px-3 text-muted hover:bg-surface-hover hover:text-text"
            activeProps={{ className: "bg-surface-raised text-text font-medium" }}
          >
            <Icon aria-hidden size={20} />
            <span className="hidden lg:inline">{t(`nav.${key}`)}</span>
          </Link>
        ))}
      </nav>
      <div className="border-t border-border p-2.5">
        <UserMenu />
      </div>
    </aside>
  );
}
