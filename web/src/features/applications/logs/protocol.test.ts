import { describe, expect, it } from "vitest";

import { MAX_LOG_LINES, appendCapped, parseLogLine, type LogEntry } from "./protocol";

describe("container log frames", () => {
  it("accepts stdout and stderr lines only", () => {
    expect(parseLogLine(JSON.stringify({ source: "stderr", text: "boom\n" }))).toEqual({ source: "stderr", text: "boom\n" });
    expect(parseLogLine(JSON.stringify({ source: "stdin", text: "x" }))).toBeNull();
    expect(parseLogLine("not json")).toBeNull();
    expect(parseLogLine(new ArrayBuffer(2))).toBeNull();
  });

  it("keeps only the newest lines", () => {
    const entry = (id: number): LogEntry => ({ id, source: "stdout", text: `${id}` });
    const full = Array.from({ length: MAX_LOG_LINES }, (_, index) => entry(index));
    const next = appendCapped(full, [entry(MAX_LOG_LINES)]);
    expect(next).toHaveLength(MAX_LOG_LINES);
    expect(next[0]?.id).toBe(1);
    expect(next.at(-1)?.id).toBe(MAX_LOG_LINES);
  });
});
