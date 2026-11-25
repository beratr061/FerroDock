import type { ReactNode } from "react";

import { useTranslation } from "@/shared/i18n";
import { LanguageSelect, Logo, ThemeSwitcher } from "@/shared/ui";

interface AuthLayoutProps {
  title: string;
  description: string;
  children: ReactNode;
}

export function AuthLayout({ title, description, children }: AuthLayoutProps) {
  const { t } = useTranslation("auth");
  return (
    <div className="grid min-h-dvh lg:grid-cols-[minmax(0,5fr)_minmax(0,6fr)]">
      <aside className="hidden flex-col justify-between border-r border-border bg-surface p-10 lg:flex">
        <Logo />
        <div className="flex max-w-sm flex-col gap-3">
          <p className="text-2xl leading-snug font-semibold text-text">{t("pitch.title")}</p>
          <p className="text-muted">{t("pitch.body")}</p>
        </div>
        <p className="font-mono text-[12px] text-subtle">Apache-2.0</p>
      </aside>
      <main className="flex flex-col px-4 py-6 sm:px-10">
        <div className="flex items-center justify-between gap-3">
          <Logo className="lg:invisible" compact />
          <div className="flex items-center gap-2">
            <LanguageSelect hideLabel className="w-36" />
            <ThemeSwitcher />
          </div>
        </div>
        <div className="flex flex-1 items-center">
          <section className="enter-settle mx-auto flex w-full max-w-sm flex-col gap-6 py-10 lg:mx-0 lg:ml-[12%]">
            <header className="flex flex-col gap-1.5">
              <h1 className="text-[22px] text-text">{title}</h1>
              <p className="text-muted">{description}</p>
            </header>
            {children}
          </section>
        </div>
      </main>
    </div>
  );
}
