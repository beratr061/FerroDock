import { useCallback, useLayoutEffect, useRef, useState } from "react";
import type { ForwardedRef } from "react";

const VALUE = Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, "value");

function forward(ref: ForwardedRef<HTMLSelectElement>, element: HTMLSelectElement | null): void {
  if (typeof ref === "function") {
    ref(element);
  } else if (ref) {
    ref.current = element;
  }
}

function reportWrites(element: HTMLSelectElement, report: (value: string) => void): void {
  Object.defineProperty(element, "value", {
    configurable: true,
    get: () => VALUE?.get?.call(element) as string,
    set: (next: string) => {
      VALUE?.set?.call(element, next);
      report(element.value);
    },
  });
}

export function useNativeSelect(ref: ForwardedRef<HTMLSelectElement>, optionsKey: string) {
  const element = useRef<HTMLSelectElement | null>(null);
  const [value, setValue] = useState("");

  const attach = useCallback(
    (node: HTMLSelectElement | null) => {
      element.current = node;
      if (node) {
        reportWrites(node, setValue);
        setValue(node.value);
      }
      forward(ref, node);
    },
    [ref],
  );

  useLayoutEffect(() => {
    if (element.current) {
      setValue(element.current.value);
    }
  }, [optionsKey]);

  const choose = (next: string): void => {
    const node = element.current;
    if (!node) {
      return;
    }
    node.value = next;
    node.dispatchEvent(new Event("change", { bubbles: true }));
  };

  return { attach, value, setValue, choose };
}
