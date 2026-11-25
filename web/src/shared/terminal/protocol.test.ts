import { describe, expect, it } from "vitest";

import { dataFrame, decodeFrame, resizeFrame } from "./protocol";

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
});
