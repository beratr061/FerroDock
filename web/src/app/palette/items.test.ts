import { describe, expect, it } from "vitest";

import { matches, type PaletteItem } from "./items";

const item: PaletteItem = { id: "1", kind: "database", label: "Sipariş DB", detail: "Mağaza · postgres", to: "/databases/1" };

describe("palette matching", () => {
  it("matches every word, ignoring case and accents", () => {
    expect(matches(item, "")).toBe(true);
    expect(matches(item, "siparis")).toBe(true);
    expect(matches(item, "magaza POSTGRES")).toBe(true);
    expect(matches(item, "siparis redis")).toBe(false);
  });
});
