import { describe, expect, it } from "vitest";

import { addBundle, translate } from "./translate";

addBundle("en", "common", { errors: { UNKNOWN: "Something went wrong", NAME: "Up to {{max}} characters" } });
addBundle("en", "apps", { title: "Applications", greeting: "Hello {{name}}" });
addBundle("tr", "apps", { title: "Uygulamalar" });

describe("translate", () => {
  it("prefers the active language and namespace", () => {
    expect(translate("tr", "apps", "title")).toBe("Uygulamalar");
    expect(translate("en", "apps", "greeting", { name: "Ada" })).toBe("Hello Ada");
  });

  it("falls back to english, the common namespace and defaults", () => {
    expect(translate("tr", "apps", "greeting", { name: "Ada" })).toBe("Hello Ada");
    expect(translate("tr", "apps", "errors.NAME", { max: 100 })).toBe("Up to 100 characters");
    expect(translate("en", "other", "title", { ns: "apps" })).toBe("Applications");
    expect(translate("en", "apps", "errors.MISSING", { defaultValue: "fallback" })).toBe("fallback");
    expect(translate("en", "apps", "missing.key")).toBe("missing.key");
  });

  it("keeps unknown placeholders visible", () => {
    expect(translate("en", "apps", "greeting")).toBe("Hello {{name}}");
  });
});
