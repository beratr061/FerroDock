import { useRef, useState } from "react";
import type { KeyboardEvent } from "react";

const PAGE = 10;
const TYPEAHEAD_MS = 500;

interface ListboxOptions {
  labels: string[];
  selected: number;
  onCommit: (index: number) => void;
}

function clamp(index: number, count: number): number {
  return Math.min(Math.max(index, 0), count - 1);
}

function useTypeahead(labels: string[]) {
  const buffer = useRef("");
  const timer = useRef<number | undefined>(undefined);
  return (key: string, from: number): number => {
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => (buffer.current = ""), TYPEAHEAD_MS);
    const repeated = buffer.current.length > 0 && [...buffer.current].every((letter) => letter === key);
    buffer.current += key;
    const needle = repeated ? key : buffer.current;
    const start = repeated ? from + 1 : from;
    const ordered = labels.map((_, offset) => (start + offset) % labels.length);
    return ordered.find((index) => labels[index]?.toLocaleLowerCase().startsWith(needle)) ?? -1;
  };
}

export function useListbox({ labels, selected, onCommit }: ListboxOptions) {
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(selected);
  const typeahead = useTypeahead(labels);
  const count = labels.length;

  const show = (index: number): void => {
    setActive(clamp(index, count));
    setOpen(true);
  };
  const close = (): void => setOpen(false);
  const commit = (index: number): void => {
    onCommit(index);
    setOpen(false);
  };

  const moves: Record<string, () => number> = {
    ArrowDown: () => active + 1,
    ArrowUp: () => active - 1,
    Home: () => 0,
    End: () => count - 1,
    PageDown: () => active + PAGE,
    PageUp: () => active - PAGE,
  };

  const whenClosed = (event: KeyboardEvent): void => {
    const openers: Record<string, number> = { ArrowDown: selected, Enter: selected, " ": selected, ArrowUp: 0, Home: 0, End: count - 1 };
    const target = openers[event.key];
    if (target !== undefined) {
      event.preventDefault();
      show(target);
    }
  };

  const whenOpen = (event: KeyboardEvent): void => {
    const move = moves[event.key];
    if (move && !(event.altKey && event.key === "ArrowUp")) {
      event.preventDefault();
      setActive(clamp(move(), count));
    } else if (event.key === "Enter" || event.key === " " || (event.altKey && event.key === "ArrowUp")) {
      event.preventDefault();
      commit(active);
    } else if (event.key === "Escape") {
      event.preventDefault();
      close();
    } else if (event.key === "Tab") {
      commit(active);
    }
  };

  const onKeyDown = (event: KeyboardEvent): void => {
    if (event.key.length === 1 && event.key !== " " && !event.ctrlKey && !event.metaKey) {
      const match = typeahead(event.key.toLocaleLowerCase(), open ? active : selected);
      if (match >= 0) {
        show(match);
      }
      return;
    }
    if (open) {
      whenOpen(event);
    } else {
      whenClosed(event);
    }
  };

  return { open, active, setActive, show, close, commit, onKeyDown };
}
