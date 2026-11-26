import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

const tokensCss = readFileSync(new URL("./tokens.css", import.meta.url), "utf8");

type Palette = Record<string, string>;

function block(selector: string): Palette {
  const start = tokensCss.indexOf(`${selector} {`);
  const body = tokensCss.slice(start, tokensCss.indexOf("}", start));
  const entries = [...body.matchAll(/--([\w-]+):\s*(#[0-9a-fA-F]{6})\s*;/g)];
  return Object.fromEntries(entries.map(([, name = "", value = ""]) => [name, value]));
}

function channel(value: number): number {
  const scaled = value / 255;
  return scaled <= 0.03928 ? scaled / 12.92 : ((scaled + 0.055) / 1.055) ** 2.4;
}

function luminance(hex: string): number {
  const [r = 0, g = 0, b = 0] = [1, 3, 5].map((offset) => parseInt(hex.slice(offset, offset + 2), 16));
  return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
}

function ratio(foreground: string, background: string): number {
  const [light, dark] = [luminance(foreground), luminance(background)].sort((a, b) => b - a);
  return ((light ?? 0) + 0.05) / ((dark ?? 0) + 0.05);
}

const light = block(":root");
const themes: Array<[string, Palette]> = [
  ["light", light],
  ["dark", { ...light, ...block(':root[data-theme="dark"]') }],
];
const SURFACES = ["bg", "surface", "surface-raised", "surface-hover"];
const STATUSES = ["success", "warning", "danger", "info"];
const AAA = 7;
const AA = 4.5;
const UI = 3;

interface Rule {
  foreground: string;
  backgrounds: string[];
  minimum: number;
}

const RULES: Rule[] = [
  { foreground: "text", backgrounds: SURFACES, minimum: AAA },
  { foreground: "text-muted", backgrounds: SURFACES, minimum: AAA },
  { foreground: "text-subtle", backgrounds: SURFACES, minimum: AA },
  { foreground: "border-strong", backgrounds: SURFACES, minimum: UI },
  { foreground: "focus-ring", backgrounds: SURFACES, minimum: UI },
  { foreground: "on-primary", backgrounds: ["primary"], minimum: AA },
  { foreground: "on-primary", backgrounds: ["primary-hover"], minimum: AA },
  ...STATUSES.map((status) => ({
    foreground: `status-${status}`,
    backgrounds: ["surface", "surface-raised"],
    minimum: AA,
  })),
];

describe.each(themes)("%s theme contrast", (_theme, palette) => {
  it.each(RULES)("$foreground meets $minimum:1", ({ foreground, backgrounds, minimum }) => {
    const color = palette[foreground];
    expect(color, `missing token --${foreground}`).toBeDefined();
    for (const background of backgrounds) {
      const ground = palette[background];
      expect(ground, `missing token --${background}`).toBeDefined();
      const measured = ratio(color ?? "", ground ?? "");
      expect(measured, `--${foreground} on --${background}`).toBeGreaterThanOrEqual(minimum);
    }
  });
});
