import { useEffect } from "react";
import {
  ArrowsInIcon,
  ArrowsOutIcon,
  CaretDownIcon,
  SquareSplitHorizontalIcon,
  SquareSplitVerticalIcon,
  TerminalWindowIcon,
  XIcon,
} from "@phosphor-icons/react";

import { useTranslation } from "../i18n";
import { cx } from "../lib/cx";

import { KeyBar } from "./KeyBar";
import {
  closeAll,
  closePane,
  closeTab,
  selectTab,
  setVisible,
  splitTab,
  toggleMaximized,
  useWorkspace,
  type Tab,
  type Workspace,
} from "./store";
import { TerminalPane } from "./TerminalPane";

const ICON_BUTTON = "press grid size-8 place-items-center rounded-control text-muted hover:bg-surface-hover hover:text-text";

function TabStrip({ workspace }: { workspace: Workspace }) {
  const { t } = useTranslation("common");
  return (
    <div role="tablist" aria-label={t("terminal.tabs")} className="flex min-w-0 flex-1 gap-1 overflow-x-auto">
      {workspace.tabs.map((tab) => (
        <div
          key={tab.id}
          className={cx(
            "flex shrink-0 items-center rounded-control border",
            tab.id === workspace.activeTab ? "border-border-strong bg-bg text-text" : "border-transparent text-muted",
          )}
        >
          <button
            type="button"
            role="tab"
            aria-selected={tab.id === workspace.activeTab}
            onClick={() => selectTab(tab.id)}
            className="press flex max-w-48 items-center gap-1.5 px-2 py-1 font-mono text-[12px] hover:text-text"
          >
            <TerminalWindowIcon size={14} aria-hidden />
            <span className="truncate">{tab.title}</span>
          </button>
          <button type="button" title={t("terminal.closeTab")} onClick={() => closeTab(tab.id)} className="press grid size-6 place-items-center rounded-control hover:text-text">
            <XIcon size={12} aria-hidden />
          </button>
        </div>
      ))}
    </div>
  );
}

function Actions({ workspace, active }: { workspace: Workspace; active: Tab | undefined }) {
  const { t } = useTranslation("common");
  return (
    <div className="flex shrink-0 items-center gap-1">
      {active ? (
        <>
          <button type="button" className={ICON_BUTTON} title={t("terminal.splitRight")} onClick={() => splitTab(active.id, "row")}>
            <SquareSplitHorizontalIcon size={16} aria-hidden />
          </button>
          <button type="button" className={ICON_BUTTON} title={t("terminal.splitDown")} onClick={() => splitTab(active.id, "column")}>
            <SquareSplitVerticalIcon size={16} aria-hidden />
          </button>
        </>
      ) : null}
      <button type="button" className={ICON_BUTTON} title={t(workspace.maximized ? "terminal.restore" : "terminal.maximize")} onClick={toggleMaximized}>
        {workspace.maximized ? <ArrowsInIcon size={16} aria-hidden /> : <ArrowsOutIcon size={16} aria-hidden />}
      </button>
      <button type="button" className={ICON_BUTTON} title={t("terminal.hide")} onClick={() => setVisible(false)}>
        <CaretDownIcon size={16} aria-hidden />
      </button>
      <button type="button" className={ICON_BUTTON} title={t("terminal.closeAll")} onClick={closeAll}>
        <XIcon size={16} aria-hidden />
      </button>
    </div>
  );
}

function TabBody({ tab, workspace }: { tab: Tab; workspace: Workspace }) {
  return (
    <div
      role="tabpanel"
      hidden={tab.id !== workspace.activeTab}
      className={cx("min-h-0 flex-1 gap-2", tab.id === workspace.activeTab ? "flex" : "hidden", tab.split === "column" ? "flex-col" : "flex-col md:flex-row")}
    >
      {tab.panes.map((pane) => (
        <TerminalPane
          key={pane.id}
          pane={pane}
          container={tab.container}
          focused={pane.id === workspace.focusedPane}
          onClose={() => closePane(tab.id, pane.id)}
        />
      ))}
    </div>
  );
}

function ReopenButton({ count }: { count: number }) {
  const { t } = useTranslation("common");
  return (
    <button
      type="button"
      onClick={() => setVisible(true)}
      className="press fixed right-4 bottom-20 z-40 inline-flex items-center gap-2 rounded-control border border-border-strong bg-surface px-3 py-2 font-mono text-[12px] text-text md:bottom-4"
    >
      <TerminalWindowIcon size={16} aria-hidden />
      {t("terminal.reopen", { count })}
    </button>
  );
}

export default function TerminalWorkspace() {
  const { t } = useTranslation("common");
  const workspace = useWorkspace();
  const active = workspace.tabs.find((tab) => tab.id === workspace.activeTab);

  useEffect(() => {
    const onKey = (event: KeyboardEvent): void => {
      if (event.key === "Escape" && event.ctrlKey) {
        setVisible(false);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  if (workspace.tabs.length === 0) {
    return null;
  }
  return (
    <>
      {workspace.visible ? null : <ReopenButton count={workspace.tabs.length} />}
      <section
        aria-label={t("terminal.workspace")}
        hidden={!workspace.visible}
        className={cx("terminal-drawer fixed inset-x-0 bottom-0 z-40 flex-col gap-2 border-t border-border-strong bg-surface px-3 pt-2 pb-3", workspace.visible ? "flex" : "hidden", workspace.maximized && "terminal-drawer-maximized")}
      >
        <header className="flex items-center gap-2">
          <TabStrip workspace={workspace} />
          <Actions workspace={workspace} active={active} />
        </header>
        {workspace.tabs.map((tab) => (
          <TabBody key={tab.id} tab={tab} workspace={workspace} />
        ))}
        <KeyBar />
      </section>
    </>
  );
}
