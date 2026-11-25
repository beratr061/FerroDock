import { describe, expect, it } from "vitest";

import { parseEnvFile, serializeEnvFile } from "./envFile";

describe("env file parsing", () => {
  it("reads common dotenv syntax", () => {
    const text = [
      "# comment",
      "export NODE_ENV=production",
      'DATABASE_URL="postgres://u:p@db:5432/app"',
      "GREETING='hello # not a comment'",
      "PORT=3000 # inline comment",
      'MULTI="line one\\nline two"',
      "",
      "not a variable",
      "PORT=4000",
    ].join("\n");
    const parsed = parseEnvFile(text);
    expect(parsed.variables).toEqual([
      { key: "NODE_ENV", value: "production" },
      { key: "DATABASE_URL", value: "postgres://u:p@db:5432/app" },
      { key: "GREETING", value: "hello # not a comment" },
      { key: "PORT", value: "4000" },
      { key: "MULTI", value: "line one\nline two" },
    ]);
    expect(parsed.invalidLines).toEqual([8]);
  });

  it("round trips values that need quoting", () => {
    const variables = [
      { key: "PLAIN", value: "abc-123" },
      { key: "SPACED", value: 'say "hi"\nnow' },
    ];
    expect(parseEnvFile(serializeEnvFile(variables)).variables).toEqual(variables);
  });
});
