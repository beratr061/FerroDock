import { lazy, Suspense } from "react";

import { useWorkspace } from "./store";

const TerminalWorkspace = lazy(() => import("./TerminalWorkspace"));

export function TerminalHost() {
  const workspace = useWorkspace();
  if (workspace.tabs.length === 0) {
    return null;
  }
  return (
    <Suspense fallback={null}>
      <TerminalWorkspace />
    </Suspense>
  );
}
