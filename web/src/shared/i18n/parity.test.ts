import { describe, expect, it } from "vitest";

type Tree = { [key: string]: Tree | string };

const bundles = import.meta.glob<Tree>("../../../locales/*/*.json", {
  import: "default",
  eager: true,
});

const PATH_PATTERN = /locales\/([^/]+)\/([^/]+)\.json$/;

function flatten(tree: Tree, prefix = ""): string[] {
  return Object.entries(tree).flatMap(([key, value]) => {
    const path = prefix ? `${prefix}.${key}` : key;
    return typeof value === "string" ? [path] : flatten(value, path);
  });
}

function placeholders(tree: Tree, prefix = ""): Record<string, string> {
  return Object.fromEntries(
    Object.entries(tree).flatMap(([key, value]) => {
      const path = prefix ? `${prefix}.${key}` : key;
      if (typeof value !== "string") {
        return Object.entries(placeholders(value, path));
      }
      const names = [...value.matchAll(/\{\{(\w+)\}\}/g)].map((match) => match[1]).sort();
      return [[path, names.join(",")]];
    }),
  );
}

const byLanguage = new Map<string, Map<string, Tree>>();
for (const [path, tree] of Object.entries(bundles)) {
  const [, language = "", namespace = ""] = PATH_PATTERN.exec(path) ?? [];
  if (namespace !== "meta") {
    const namespaces = byLanguage.get(language) ?? new Map<string, Tree>();
    namespaces.set(namespace, tree);
    byLanguage.set(language, namespaces);
  }
}

const reference = byLanguage.get("en") ?? new Map<string, Tree>();
const others = [...byLanguage.keys()].filter((language) => language !== "en");

describe("locale parity", () => {
  it("has English as the reference language", () => {
    expect(reference.size).toBeGreaterThan(0);
    expect(others.length).toBeGreaterThan(0);
  });

  it.each(others)("%s defines the same namespaces as English", (language) => {
    const namespaces = [...(byLanguage.get(language)?.keys() ?? [])].sort();
    expect(namespaces).toEqual([...reference.keys()].sort());
  });

  it.each(others)("%s defines the same keys and placeholders as English", (language) => {
    for (const [namespace, englishTree] of reference) {
      const translated = byLanguage.get(language)?.get(namespace) ?? {};
      expect(flatten(translated).sort(), `${language}/${namespace}`).toEqual(flatten(englishTree).sort());
      expect(placeholders(translated), `${language}/${namespace}`).toEqual(placeholders(englishTree));
    }
  });
});
