import { useEffect, useState } from "react";

import { fetchDeploymentLog, isActive, type DeploymentStatus } from "./api";

const FOLLOW_INTERVAL_MS = 1_200;
const MAX_KEPT_CHARS = 400_000;

interface LogState {
  text: string;
  status: DeploymentStatus | null;
  failed: boolean;
}

function delay(ms: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve) => {
    const timer = setTimeout(resolve, ms);
    signal.addEventListener("abort", () => {
      clearTimeout(timer);
      resolve();
    });
  });
}

async function follow(id: string, signal: AbortSignal, update: (next: (prev: LogState) => LogState) => void) {
  let offset = 0;
  while (!signal.aborted) {
    try {
      const chunk = await fetchDeploymentLog(id, offset);
      if (signal.aborted) {
        return;
      }
      const drained = chunk.next_offset === offset;
      offset = chunk.next_offset;
      update((prev) => ({ text: (prev.text + chunk.text).slice(-MAX_KEPT_CHARS), status: chunk.status, failed: false }));
      if (drained && !isActive(chunk.status)) {
        return;
      }
      if (drained) {
        await delay(FOLLOW_INTERVAL_MS, signal);
      }
    } catch {
      update((prev) => ({ ...prev, failed: true }));
      await delay(FOLLOW_INTERVAL_MS * 3, signal);
    }
  }
}

export function useDeploymentLog(id: string): LogState {
  const [state, setState] = useState<LogState>({ text: "", status: null, failed: false });
  useEffect(() => {
    const controller = new AbortController();
    void follow(id, controller.signal, setState);
    return () => controller.abort();
  }, [id]);
  return state;
}
