import { useCallback, useMemo, useSyncExternalStore } from "react";

import {
  addBundle,
  COMMON_NAMESPACE,
  FALLBACK_LANGUAGE,
  translate,
  type Bundle,
  type TFunction,
  type TranslateOptions,
} from "./translate";

export type { TFunction } from "./translate";

const bundleLoaders = import.meta.glob<Bundle>("../../../locales/*/*.json", { import: "default" });
const metaFiles = import.meta.glob<{ name: string }>("../../../locales/*/meta.json", {
  import: "default",
  eager: true,
});

const LOCALE_PATTERN = /locales\/([^/]+)\/([^/]+)\.json$/;
const STORAGE_KEY = "fd-locale";

export interface Language {
  code: string;
  name: string;
}

export const languages: Language[] = Object.entries(metaFiles)
  .map(([path, meta]) => ({ code: LOCALE_PATTERN.exec(path)?.[1] ?? FALLBACK_LANGUAGE, name: meta.name }))
  .sort((a, b) => a.name.localeCompare(b.name));

let current = FALLBACK_LANGUAGE;
const listeners = new Set<() => void>();

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function snapshot(): string {
  return current;
}

function supported(code: string | null | undefined): string | undefined {
  const base = code?.toLowerCase().split("-")[0];
  return languages.some((language) => language.code === base) ? base : undefined;
}

function readStored(): string | null {
  try {
    return localStorage.getItem(STORAGE_KEY);
  } catch {
    return null;
  }
}

async function loadLanguage(code: string): Promise<void> {
  const entries = Object.entries(bundleLoaders).filter(([path]) => {
    const match = LOCALE_PATTERN.exec(path);
    return match?.[1] === code && match[2] !== "meta";
  });
  await Promise.all(
    entries.map(async ([path, load]) => {
      const namespace = LOCALE_PATTERN.exec(path)?.[2];
      if (namespace) {
        addBundle(code, namespace, await load());
      }
    }),
  );
}

function activate(language: string): void {
  current = language;
  document.documentElement.lang = language;
  listeners.forEach((listener) => listener());
}

export function initialLanguage(): string {
  return supported(readStored()) ?? supported(navigator.language) ?? FALLBACK_LANGUAGE;
}

export async function setupI18n(): Promise<void> {
  const language = initialLanguage();
  await Promise.all([loadLanguage(FALLBACK_LANGUAGE), language === FALLBACK_LANGUAGE ? null : loadLanguage(language)]);
  activate(language);
}

export async function changeLanguage(code: string): Promise<void> {
  const language = supported(code) ?? FALLBACK_LANGUAGE;
  await loadLanguage(language);
  activate(language);
  try {
    localStorage.setItem(STORAGE_KEY, language);
  } catch {
    return;
  }
}

export function useTranslation(namespace: string = COMMON_NAMESPACE): { t: TFunction; i18n: { language: string } } {
  const language = useSyncExternalStore(subscribe, snapshot, snapshot);
  const t = useCallback(
    (key: string, options?: TranslateOptions) => translate(language, namespace, key, options),
    [language, namespace],
  );
  return useMemo(() => ({ t, i18n: { language } }), [t, language]);
}
