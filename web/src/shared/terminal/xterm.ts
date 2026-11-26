import type { FitAddon } from "@xterm/addon-fit";
import type { SearchAddon } from "@xterm/addon-search";
import type { Terminal } from "@xterm/xterm";

import { withStyleNonce } from "./styleNonce";

export interface MountedTerminal {
  term: Terminal;
  fit: FitAddon;
  search: SearchAddon;
}

function token(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

function theme() {
  return {
    background: token("--bg"),
    foreground: token("--text"),
    cursor: token("--text"),
    cursorAccent: token("--bg"),
    selectionBackground: token("--surface-hover"),
    selectionForeground: token("--text"),
  };
}

export async function mountTerminal(element: HTMLElement): Promise<MountedTerminal> {
  const [xterm, fit, links, search] = await Promise.all([
    import("@xterm/xterm"),
    import("@xterm/addon-fit"),
    import("@xterm/addon-web-links"),
    import("@xterm/addon-search"),
    import("@xterm/xterm/css/xterm.css"),
  ]);
  const term = new xterm.Terminal({
    fontFamily: '"JetBrains Mono", ui-monospace, monospace',
    fontSize: 13,
    cursorBlink: true,
    scrollback: 5000,
    allowProposedApi: true,
    theme: theme(),
  });
  const mounted = { term, fit: new fit.FitAddon(), search: new search.SearchAddon() };
  term.loadAddon(mounted.fit);
  term.loadAddon(mounted.search);
  term.loadAddon(new links.WebLinksAddon());
  withStyleNonce(() => term.open(element));
  return mounted;
}

export function fitVisible(mounted: MountedTerminal, element: HTMLElement): void {
  if (element.clientWidth > 0 && element.clientHeight > 0) {
    mounted.fit.fit();
  }
}
