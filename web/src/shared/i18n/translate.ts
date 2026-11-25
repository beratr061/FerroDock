export type Bundle = { [key: string]: Bundle | string };
export type TranslateOptions = { ns?: string; defaultValue?: string } & Record<string, unknown>;
export type TFunction = (key: string, options?: TranslateOptions) => string;

export const FALLBACK_LANGUAGE = "en";
export const COMMON_NAMESPACE = "common";

const bundles = new Map<string, Map<string, Bundle>>();

export function addBundle(language: string, namespace: string, bundle: Bundle): void {
  const namespaces = bundles.get(language) ?? new Map<string, Bundle>();
  namespaces.set(namespace, bundle);
  bundles.set(language, namespaces);
}

function lookup(language: string, namespace: string, key: string): string | undefined {
  let node: Bundle | string | undefined = bundles.get(language)?.get(namespace);
  for (const part of key.split(".")) {
    if (node === undefined || typeof node === "string") {
      return undefined;
    }
    node = node[part];
  }
  return typeof node === "string" ? node : undefined;
}

function interpolate(template: string, values: Record<string, unknown>): string {
  return template.replace(/\{\{(\w+)\}\}/g, (match, name: string) => {
    const value = values[name];
    return value === undefined || value === null ? match : String(value);
  });
}

export function translate(language: string, namespace: string, key: string, options: TranslateOptions = {}): string {
  const namespaces = [...new Set([options.ns ?? namespace, COMMON_NAMESPACE])];
  const languages = [...new Set([language, FALLBACK_LANGUAGE])];
  for (const lang of languages) {
    for (const ns of namespaces) {
      const found = lookup(lang, ns, key);
      if (found !== undefined) {
        return interpolate(found, options);
      }
    }
  }
  return options.defaultValue ?? key;
}
