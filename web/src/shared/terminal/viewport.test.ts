import { describe, expect, it } from "vitest";

import { keyboardInset } from "./viewport";

describe("keyboard inset", () => {
  it("is the part of the layout viewport the keyboard covers", () => {
    expect(keyboardInset({ layoutHeight: 812, visibleHeight: 812, visibleTop: 0 })).toBe(0);
    expect(keyboardInset({ layoutHeight: 812, visibleHeight: 476, visibleTop: 0 })).toBe(336);
    expect(keyboardInset({ layoutHeight: 812, visibleHeight: 476, visibleTop: 120 })).toBe(216);
    expect(keyboardInset({ layoutHeight: 812, visibleHeight: 900, visibleTop: 0 })).toBe(0);
  });
});
