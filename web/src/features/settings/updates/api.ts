import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";

import { api, unwrap, type Schemas } from "@/shared/api/client";

export type UpdateRecord = Schemas["UpdateRecordDto"];
export type UpdateCheck = Schemas["UpdateCheckDto"];

const OVERVIEW_KEY = ["system", "update"] as const;
const RESTART_POLL_MS = 2_000;

export const updateOverviewQuery = queryOptions({
  queryKey: OVERVIEW_KEY,
  queryFn: () => unwrap(api.GET("/api/v1/system/update")),
  retry: false,
});

export const updateCheckQuery = queryOptions({
  queryKey: [...OVERVIEW_KEY, "check"],
  queryFn: () => unwrap(api.GET("/api/v1/system/update/check")),
  retry: false,
  enabled: false,
});

export function useStartUpdate(rollback: boolean) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () =>
      unwrap(rollback ? api.POST("/api/v1/system/update/rollback") : api.POST("/api/v1/system/update")),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: OVERVIEW_KEY }),
  });
}

async function serverVersion(): Promise<string | null> {
  try {
    const response = await fetch("/api/v1/health", { cache: "no-store" });
    if (!response.ok) {
      return null;
    }
    const body = (await response.json()) as { version?: string };
    return body.version ?? null;
  } catch {
    return null;
  }
}

export async function waitForRestart(startedFrom: string, signal: AbortSignal): Promise<string> {
  let sawDowntime = false;
  while (!signal.aborted) {
    await new Promise((resolve) => setTimeout(resolve, RESTART_POLL_MS));
    const version = await serverVersion();
    if (version === null) {
      sawDowntime = true;
    } else if (sawDowntime || version !== startedFrom) {
      return version;
    }
  }
  return startedFrom;
}
