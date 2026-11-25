import { DesktopIcon, MoonIcon, SunIcon } from "@phosphor-icons/react";
import { useTranslation } from "react-i18next";

import { useTheme, type ThemePreference } from "../hooks/useTheme";
import { cx } from "../lib/cx";

const OPTIONS: Array<{ value: ThemePreference; Icon: typeof SunIcon }> = [
  { value: "system", Icon: DesktopIcon },
  { value: "light", Icon: SunIcon },
  { value: "dark", Icon: MoonIcon },
];

export function ThemeSwitcher() {
  const { t } = useTranslation();
  const [preference, setPreference] = useTheme();
  return (
    <div
      role="radiogroup"
      aria-label={t("theme.label")}
      className="inline-flex rounded-control border border-border bg-surface p-0.5"
    >
      {OPTIONS.map(({ value, Icon }) => (
        <button
          key={value}
          type="button"
          role="radio"
          aria-checked={preference === value}
          title={t(`theme.${value}`)}
          onClick={() => setPreference(value)}
          className={cx(
            "press grid size-8 place-items-center rounded-[5px]",
            preference === value ? "bg-surface-raised text-text" : "text-muted hover:text-text",
          )}
        >
          <Icon aria-hidden size={16} weight={preference === value ? "bold" : "regular"} />
          <span className="sr-only">{t(`theme.${value}`)}</span>
        </button>
      ))}
    </div>
  );
}
