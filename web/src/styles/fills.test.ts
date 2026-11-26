import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

const SOURCE = fileURLToPath(new URL("..", import.meta.url));

const TRANSLUCENT_CLASSES = [
  /\b(?:bg|fill|from|via|to|border|ring|divide|outline|text)-[\w-]+\/\d+/,
  /\b(?:bg|border|text)-opacity-\d+/,
  /\bopacity-(?!100\b)\d+/,
  /\b(?:bg|border|text)-[\w-]+-tint\b/,
];

const TRANSLUCENT_CSS = [/color-mix\(/, /\b(?:rgba|hsla)\(/, /#[0-9a-fA-F]{8}\b/];

function files(directory: string): string[] {
  return readdirSync(directory).flatMap((name) => {
    const path = join(directory, name);
    if (statSync(path).isDirectory()) {
      return files(path);
    }
    return /\.(tsx?|css)$/.test(name) && !name.includes(".test.") ? [path] : [];
  });
}

function offending(path: string): string[] {
  const patterns = path.endsWith(".css") ? TRANSLUCENT_CSS : TRANSLUCENT_CLASSES;
  return readFileSync(path, "utf8")
    .split("\n")
    .filter((line) => !line.includes("--shadow-overlay"))
    .filter((line) => patterns.some((pattern) => pattern.test(line)))
    .map((line) => `${path}: ${line.trim()}`);
}

describe("fills", () => {
  it("are always solid tokens", () => {
    expect(files(SOURCE).flatMap(offending)).toEqual([]);
  });
});
