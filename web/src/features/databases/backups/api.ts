import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";

import { api, unwrap, unwrapEmpty, type Schemas } from "@/shared/api/client";

export type BackupRun = Schemas["BackupRunDto"];
export type BackupPolicy = Schemas["BackupPolicyDto"];
export type PolicyRequest = Schemas["BackupPolicyRequest"];

const POLL_MS = 2_000;

function active(runs: BackupRun[] | undefined): boolean {
  return (runs ?? []).some((run) => run.status === "queued" || run.status === "running");
}

export const destinationsQuery = queryOptions({
  queryKey: ["backup-destinations"],
  queryFn: () => unwrap(api.GET("/api/v1/backup-destinations")),
});

export function policyQuery(databaseId: string) {
  return queryOptions({
    queryKey: ["backup-policy", databaseId],
    queryFn: () =>
      unwrap(api.GET("/api/v1/databases/{id}/backup-policy", { params: { path: { id: databaseId } } })),
  });
}

export function runsQuery(databaseId: string) {
  return queryOptions({
    queryKey: ["backup-runs", databaseId],
    queryFn: () => unwrap(api.GET("/api/v1/databases/{id}/backups", { params: { path: { id: databaseId } } })),
    refetchInterval: (query) => (active(query.state.data) ? POLL_MS : false),
  });
}

function useInvalidate(prefix: string, databaseId: string) {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: [prefix, databaseId] });
}

export function useSavePolicy(databaseId: string) {
  const invalidate = useInvalidate("backup-policy", databaseId);
  return useMutation({
    mutationFn: (body: PolicyRequest) =>
      unwrap(api.PUT("/api/v1/databases/{id}/backup-policy", { params: { path: { id: databaseId } }, body })),
    onSuccess: invalidate,
  });
}

export function useRemovePolicy(databaseId: string) {
  const invalidate = useInvalidate("backup-policy", databaseId);
  return useMutation({
    mutationFn: () =>
      unwrapEmpty(api.DELETE("/api/v1/databases/{id}/backup-policy", { params: { path: { id: databaseId } } })),
    onSuccess: invalidate,
  });
}

export function useBackUpNow(databaseId: string) {
  const invalidate = useInvalidate("backup-runs", databaseId);
  return useMutation({
    mutationFn: (destinationId: string) =>
      unwrap(
        api.POST("/api/v1/databases/{id}/backups", {
          params: { path: { id: databaseId } },
          body: { destination_id: destinationId },
        }),
      ),
    onSuccess: invalidate,
  });
}

export function useRestore(databaseId: string) {
  const invalidate = useInvalidate("backup-runs", databaseId);
  return useMutation({
    mutationFn: (runId: string) =>
      unwrap(api.POST("/api/v1/database-backups/{id}/restore", { params: { path: { id: runId } } })),
    onSuccess: invalidate,
  });
}

export function useDeleteRun(databaseId: string) {
  const invalidate = useInvalidate("backup-runs", databaseId);
  return useMutation({
    mutationFn: (runId: string) =>
      unwrapEmpty(api.DELETE("/api/v1/database-backups/{id}", { params: { path: { id: runId } } })),
    onSuccess: invalidate,
  });
}
