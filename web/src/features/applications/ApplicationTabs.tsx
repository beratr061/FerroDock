import { useEffect, useRef } from "react";
import type { Icon } from "@phosphor-icons/react";
import {
  BracketsCurlyIcon,
  GearSixIcon,
  GlobeIcon,
  HardDrivesIcon,
  RocketLaunchIcon,
  ScrollIcon,
} from "@phosphor-icons/react";
import { Link } from "@tanstack/react-router";

import { useTranslation } from "@/shared/i18n";
import { cx } from "@/shared/lib/cx";

import { APPLICATION_TABS, type ApplicationTab } from "./tabs";

const ICONS: Record<ApplicationTab, Icon> = {
  deployments: RocketLaunchIcon,
  logs: ScrollIcon,
  environment: BracketsCurlyIcon,
  domains: GlobeIcon,
  storage: HardDrivesIcon,
  settings: GearSixIcon,
};

interface ApplicationTabsProps {
  applicationId: string;
  active: ApplicationTab;
}

export function ApplicationTabs({ applicationId, active }: ApplicationTabsProps) {
  const { t } = useTranslation("applications");
  const strip = useRef<HTMLElement>(null);

  useEffect(() => {
    strip.current?.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: "nearest", inline: "nearest" });
  }, [active]);

  return (
    <nav ref={strip} aria-label={t("tabs.label")} className="-mx-1 overflow-x-auto overflow-y-hidden border-b border-border">
      <div role="tablist" className="flex w-max gap-1 px-1">
        {APPLICATION_TABS.map((tab) => {
          const TabIcon = ICONS[tab];
          const selected = tab === active;
          return (
            <Link
              key={tab}
              role="tab"
              aria-selected={selected}
              to="/applications/$applicationId"
              params={{ applicationId }}
              search={{ tab }}
              replace
              className={cx(
                "press -mb-px flex items-center gap-2 border-b-2 px-3 py-2.5 whitespace-nowrap",
                selected ? "border-text font-medium text-text" : "border-transparent text-muted hover:text-text",
              )}
            >
              <TabIcon size={16} aria-hidden />
              {t(`tabs.${tab}`)}
            </Link>
          );
        })}
      </div>
    </nav>
  );
}
