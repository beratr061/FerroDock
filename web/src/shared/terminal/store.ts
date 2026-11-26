import { useSyncExternalStore } from "react";

export type SplitDirection = "row" | "column";

export interface Pane {
  id: string;
  sessionId: string | null;
}

export interface Tab {
  id: string;
  title: string;
  container: string;
  split: SplitDirection;
  panes: Pane[];
}

export interface Workspace {
  visible: boolean;
  maximized: boolean;
  activeTab: string | null;
  focusedPane: string | null;
  tabs: Tab[];
}

const STORAGE_KEY = "fd-terminals";
const MAX_PANES = 2;
const EMPTY: Workspace = { visible: false, maximized: false, activeTab: null, focusedPane: null, tabs: [] };

function newId(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(16));
  return Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join("");
}

function restore(): Workspace {
  try {
    const raw = sessionStorage.getItem(STORAGE_KEY);
    return raw ? { ...EMPTY, ...(JSON.parse(raw) as Workspace) } : EMPTY;
  } catch {
    return EMPTY;
  }
}

let state: Workspace = restore();
const listeners = new Set<() => void>();

function update(next: Workspace): void {
  state = next;
  try {
    sessionStorage.setItem(STORAGE_KEY, JSON.stringify(next));
  } catch {
    return;
  } finally {
    listeners.forEach((listener) => listener());
  }
}

function mapTab(tabId: string, change: (tab: Tab) => Tab): Tab[] {
  return state.tabs.map((tab) => (tab.id === tabId ? change(tab) : tab));
}

export function useWorkspace(): Workspace {
  return useSyncExternalStore(
    (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    () => state,
  );
}

export function openTerminal(container: string, title: string): void {
  const pane: Pane = { id: newId(), sessionId: null };
  const tab: Tab = { id: newId(), title, container, split: "row", panes: [pane] };
  update({ ...state, visible: true, tabs: [...state.tabs, tab], activeTab: tab.id, focusedPane: pane.id });
}

export function selectTab(tabId: string): void {
  const tab = state.tabs.find((candidate) => candidate.id === tabId);
  update({ ...state, activeTab: tabId, focusedPane: tab?.panes[0]?.id ?? null });
}

export function closeTab(tabId: string): void {
  const tabs = state.tabs.filter((tab) => tab.id !== tabId);
  const active = state.activeTab === tabId ? (tabs.at(-1)?.id ?? null) : state.activeTab;
  const focused = tabs.find((tab) => tab.id === active)?.panes[0]?.id ?? null;
  update({ ...state, tabs, activeTab: active, focusedPane: focused, visible: tabs.length > 0 && state.visible });
}

export function splitTab(tabId: string, split: SplitDirection): void {
  const pane: Pane = { id: newId(), sessionId: null };
  const tabs = mapTab(tabId, (tab) =>
    tab.panes.length >= MAX_PANES ? { ...tab, split } : { ...tab, split, panes: [...tab.panes, pane] },
  );
  update({ ...state, tabs, focusedPane: pane.id });
}

export function closePane(tabId: string, paneId: string): void {
  const tab = state.tabs.find((candidate) => candidate.id === tabId);
  if (!tab || tab.panes.length <= 1) {
    closeTab(tabId);
    return;
  }
  const panes = tab.panes.filter((pane) => pane.id !== paneId);
  update({ ...state, tabs: mapTab(tabId, (current) => ({ ...current, panes })), focusedPane: panes[0]?.id ?? null });
}

export function rememberSession(paneId: string, sessionId: string | null): void {
  const tabs = state.tabs.map((tab) => ({
    ...tab,
    panes: tab.panes.map((pane) => (pane.id === paneId ? { ...pane, sessionId } : pane)),
  }));
  update({ ...state, tabs });
}

export function focusPane(paneId: string): void {
  if (state.focusedPane !== paneId) {
    update({ ...state, focusedPane: paneId });
  }
}

export function setVisible(visible: boolean): void {
  update({ ...state, visible: visible && state.tabs.length > 0 });
}

export function toggleMaximized(): void {
  update({ ...state, maximized: !state.maximized });
}

export function closeAll(): void {
  update(EMPTY);
}

const senders = new Map<string, (text: string) => void>();

export function registerSender(paneId: string, send: (text: string) => void): () => void {
  senders.set(paneId, send);
  return () => senders.delete(paneId);
}

export function sendToFocused(text: string): void {
  if (state.focusedPane) {
    senders.get(state.focusedPane)?.(text);
  }
}
