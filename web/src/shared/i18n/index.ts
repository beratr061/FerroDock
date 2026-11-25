import i18next from "i18next";
import { initReactI18next } from "react-i18next";

type Bundle = Record<string, unknown>;

const bundleLoaders = import.meta.glob<Bundle>("../../../locales/*/*.json", { import: "default" });
const metaFiles = import.meta.glob<{ name: string }>("../../../locales/*/meta.json", {
  import: "default",
  eager: true,
});

const LOCALE_PATTERN = /locales\/([^/]+)\/([^/]+)\.json$/;
const STORAGE_KEY = "fd-locale";
const FALLBACK = "en";

export interface Language {
  code: string;
  name: string;
}

export const languages: Language[] = Object.entries(metaFiles)
  .map(([path, meta]) => ({ code: LOCALE_PATTERN.exec(path)?.[1] ?? FALLBACK, name: meta.name }))
  .sort((a, b) => a.name.localeCompare(b.name));

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
        i18next.addResourceBundle(code, namespace, await load(), true, true);
      }
    }),
  );
}

export function initialLanguage(): string {
  return supported(readStored()) ?? supported(navigator.language) ?? FALLBACK;
}

export async function setupI18n(): Promise<void> {
  const language = initialLanguage();
  await i18next.use(initReactI18next).init({
    lng: language,
    fallbackLng: FALLBACK,
    defaultNS: "common",
    interpolation: { escapeValue: false },
    resources: {},
  });
  await Promise.all([loadLanguage(FALLBACK), language === FALLBACK ? null : loadLanguage(language)]);
  document.documentElement.lang = language;
}

export async function changeLanguage(code: string): Promise<void> {
  const language = supported(code) ?? FALLBACK;
  await loadLanguage(language);
  await i18next.changeLanguage(language);
  document.documentElement.lang = language;
  try {
    localStorage.setItem(STORAGE_KEY, language);
  } catch {
    return;
  }
}
