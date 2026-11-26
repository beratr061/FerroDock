import { describe, expect, it } from "vitest";

import { dataFrame, decodeFrame, reconnectDelay, resizeFrame } from "./protocol";

describe("terminal framing", () => {
  it("prefixes input and control frames", () => {
    expect([...dataFrame("ls\r")]).toEqual([0, 108, 115, 13]);
    const resize = resizeFrame(120, 40);
    expect(resize[0]).toBe(1);
    expect(JSON.parse(new TextDecoder().decode(resize.subarray(1)))).toEqual({ type: "resize", cols: 120, rows: 40 });
  });

  it("decodes output and exit frames", () => {
    const output = decodeFrame(new Uint8Array([0, 104, 105]).buffer);
    expect(output?.kind === "data" ? [...output.bytes] : null).toEqual([104, 105]);
    const exit = new TextEncoder().encode(JSON.stringify({ type: "exit", code: 130 }));
    const frame = new Uint8Array(exit.length + 1);
    frame[0] = 1;
    frame.set(exit, 1);
    expect(decodeFrame(frame.buffer)).toEqual({ kind: "exit", code: 130 });
    expect(decodeFrame(new Uint8Array([7]).buffer)).toBeNull();
  });

  it("decodes the session greeting and ignores broken control frames", () => {
    const hello = new TextEncoder().encode(JSON.stringify({ type: "session", id: "abc" }));
    const frame = new Uint8Array(hello.length + 1);
    frame[0] = 1;
    frame.set(hello, 1);
    expect(decodeFrame(frame.buffer)).toEqual({ kind: "session", id: "abc" });
    expect(decodeFrame(new Uint8Array([1, 123]).buffer)).toBeNull();
  });

  it("backs off reconnects up to ten seconds", () => {
    expect([0, 1, 2, 3, 4, 9].map(reconnectDelay)).toEqual([1000, 2000, 4000, 8000, 10000, 10000]);
  });
});
