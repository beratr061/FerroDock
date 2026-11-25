import { useEffect, useState } from "react";

import { isRealtimeConnected, waitForSignal } from "@/shared/realtime";

import { fetchDeploymentLog, isActive, type DeploymentStatus } from "./api";

const FOLLOW_INTERVAL_MS = 1_200;
const REALTIME_SAFETY_MS = 10_000;
const MAX_KEPT_CHARS = 400_000;

interface LogState {
  text: string;
  status: DeploymentStatus | null;
  failed: boolean;
}

function waitForMore(id: string, signal: AbortSignal): Promise<void> {
  const timeout = isRealtimeConnected() ? REALTIME_SAFETY_MS : FOLLOW_INTERVAL_MS;
  return waitForSignal((event) => event.type === "deployment_log" && event.deployment_id === id, timeout, signal);
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
        await waitForMore(id, signal);
      }
    } catch {
      update((prev) => ({ ...prev, failed: true }));
      await waitForSignal(() => false, FOLLOW_INTERVAL_MS * 3, signal);
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
