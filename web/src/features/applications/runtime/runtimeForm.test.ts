import { describe, expect, it } from "vitest";

import { toForm, toRequest, validPort } from "./runtimeForm";

describe("runtime form", () => {
  it("round-trips mounts and ports", () => {
    const config = {
      mounts: [{ kind: "volume" as const, source: "data", target: "/var/lib/data", read_only: true }],
      ports: [{ published: 5432, target: 5432, protocol: "tcp" as const }],
    };
    expect(toRequest(toForm(config))).toEqual(config);
  });

  it("trims paths and validates port numbers", () => {
    const form = { mounts: [{ kind: "bind" as const, source: " /srv ", target: " /app ", readOnly: false }], ports: [] };
    expect(toRequest(form).mounts[0]).toEqual({ kind: "bind", source: "/srv", target: "/app", read_only: false });
    expect(validPort("65535")).toBe(true);
    expect(validPort("0")).toBe(false);
    expect(validPort("70000")).toBe(false);
    expect(validPort("80a")).toBe(false);
  });
});
