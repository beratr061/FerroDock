import { useEffect } from "react";

const INSET_PROPERTY = "--fd-keyboard-inset";
const VISIBLE_PROPERTY = "--fd-visible-height";

export interface ViewportSize {
  layoutHeight: number;
  visibleHeight: number;
  visibleTop: number;
}

export function keyboardInset(size: ViewportSize): number {
  return Math.max(0, Math.round(size.layoutHeight - size.visibleHeight - size.visibleTop));
}

function apply(viewport: VisualViewport): void {
  const root = document.documentElement.style;
  const size = { layoutHeight: window.innerHeight, visibleHeight: viewport.height, visibleTop: viewport.offsetTop };
  root.setProperty(INSET_PROPERTY, `${keyboardInset(size)}px`);
  root.setProperty(VISIBLE_PROPERTY, `${Math.round(viewport.height)}px`);
}

export function useKeyboardInset(): void {
  useEffect(() => {
    const viewport = window.visualViewport;
    if (!viewport) {
      return undefined;
    }
    const update = (): void => apply(viewport);
    update();
    viewport.addEventListener("resize", update);
    viewport.addEventListener("scroll", update);
    return () => {
      viewport.removeEventListener("resize", update);
      viewport.removeEventListener("scroll", update);
      document.documentElement.style.removeProperty(INSET_PROPERTY);
      document.documentElement.style.removeProperty(VISIBLE_PROPERTY);
    };
  }, []);
}
