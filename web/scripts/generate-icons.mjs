import { writeFileSync } from "node:fs";

const ICONS = {
  Cube: ["regular"],
  GearSix: ["regular"],
  HardDrives: ["regular"],
  SignOut: ["regular"],
  Copy: ["regular"],
  Check: ["bold"],
  CaretDown: ["bold"],
  CheckCircle: ["bold"],
  Info: ["bold"],
  Warning: ["bold"],
  WarningCircle: ["bold"],
  Sun: ["regular", "bold"],
  Moon: ["regular", "bold"],
  Desktop: ["regular", "bold"],
};

function flatten(element) {
  if (Array.isArray(element)) {
    return element.flatMap(flatten);
  }
  if (!element || typeof element !== "object") {
    return [];
  }
  const { children, ...attributes } = element.props ?? {};
  if (typeof element.type === "string") {
    return [[element.type, attributes]];
  }
  return flatten(children);
}

const glyphs = {};
for (const [name, weights] of Object.entries(ICONS)) {
  const module = await import(new URL(`../node_modules/@phosphor-icons/react/dist/defs/${name}.es.js`, import.meta.url).href);
  glyphs[name] = Object.fromEntries(weights.map((weight) => [weight, flatten(module.default.get(weight))]));
}

const source = `export type Glyph = [string, Record<string, string>][];

export const GLYPHS = ${JSON.stringify(glyphs, null, 2)} satisfies Record<string, Partial<Record<"regular" | "bold", Glyph>>>;
`;
writeFileSync(new URL("../src/shared/icons/glyphs.ts", import.meta.url), source);
console.log(`wrote ${Object.keys(glyphs).length} icons`);
