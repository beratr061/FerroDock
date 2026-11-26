import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";

import { api, unwrap, unwrapEmpty, type Schemas } from "../api/client";

export type BackupRun = Schemas["BackupRunDto"];
export type BackupPolicy = Schemas["BackupPolicyDto"];
export type PolicyRequest = Schemas["BackupPolicyRequest"];

export type BackupTarget =
  | { kind: "database"; id: string }
  | { kind: "volume"; applicationId: string; volume: string };

const POLL_MS = 2_000;

function active(runs: BackupRun[] | undefined): boolean {
  return (runs ?? []).some((run) => run.status === "queued" || run.status === "running");
}

function targetKey(target: BackupTarget): string {
  return target.kind === "database" ? target.id : `${target.applicationId}/${target.volume}`;
}

function volumePath(target: { applicationId: string; volume: string }) {
  return { params: { path: { id: target.applicationId, volume: target.volume } } };
}

function fetchPolicy(target: BackupTarget) {
  return target.kind === "database"
    ? unwrap(api.GET("/api/v1/databases/{id}/backup-policy", { params: { path: { id: target.id } } }))
    : unwrap(api.GET("/api/v1/applications/{id}/volumes/{volume}/backup-policy", volumePath(target)));
}

function fetchRuns(target: BackupTarget) {
  return target.kind === "database"
    ? unwrap(api.GET("/api/v1/databases/{id}/backups", { params: { path: { id: target.id } } }))
    : unwrap(api.GET("/api/v1/applications/{id}/volumes/{volume}/backups", volumePath(target)));
}

function putPolicy(target: BackupTarget, body: PolicyRequest) {
  return target.kind === "database"
    ? unwrap(api.PUT("/api/v1/databases/{id}/backup-policy", { params: { path: { id: target.id } }, body }))
    : unwrap(api.PUT("/api/v1/applications/{id}/volumes/{volume}/backup-policy", { ...volumePath(target), body }));
}

function deletePolicy(target: BackupTarget) {
  return target.kind === "database"
    ? unwrapEmpty(api.DELETE("/api/v1/databases/{id}/backup-policy", { params: { path: { id: target.id } } }))
    : unwrapEmpty(api.DELETE("/api/v1/applications/{id}/volumes/{volume}/backup-policy", volumePath(target)));
}

function postBackup(target: BackupTarget, destinationId: string) {
  const body = { destination_id: destinationId };
  return target.kind === "database"
    ? unwrap(api.POST("/api/v1/databases/{id}/backups", { params: { path: { id: target.id } }, body }))
    : unwrap(api.POST("/api/v1/applications/{id}/volumes/{volume}/backups", { ...volumePath(target), body }));
}

export const destinationsQuery = queryOptions({
  queryKey: ["backup-destinations"],
  queryFn: () => unwrap(api.GET("/api/v1/backup-destinations")),
});

export function policyQuery(target: BackupTarget) {
  return queryOptions({
    queryKey: ["backup-policy", targetKey(target)],
    queryFn: () => fetchPolicy(target),
  });
}

export function runsQuery(target: BackupTarget) {
  return queryOptions({
    queryKey: ["backup-runs", targetKey(target)],
    queryFn: () => fetchRuns(target),
    refetchInterval: (query) => (active(query.state.data) ? POLL_MS : false),
  });
}

function useInvalidate(prefix: string, target: BackupTarget) {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: [prefix, targetKey(target)] });
}

export function useSavePolicy(target: BackupTarget) {
  const invalidate = useInvalidate("backup-policy", target);
  return useMutation({ mutationFn: (body: PolicyRequest) => putPolicy(target, body), onSuccess: invalidate });
}

export function useRemovePolicy(target: BackupTarget) {
  const invalidate = useInvalidate("backup-policy", target);
  return useMutation({ mutationFn: () => deletePolicy(target), onSuccess: invalidate });
}

export function useBackUpNow(target: BackupTarget) {
  const invalidate = useInvalidate("backup-runs", target);
  return useMutation({ mutationFn: (destinationId: string) => postBackup(target, destinationId), onSuccess: invalidate });
}

export function useRestore(target: BackupTarget) {
  const invalidate = useInvalidate("backup-runs", target);
  return useMutation({
    mutationFn: (runId: string) =>
      unwrap(api.POST("/api/v1/backup-runs/{id}/restore", { params: { path: { id: runId } } })),
    onSuccess: invalidate,
  });
}

export function useDeleteRun(target: BackupTarget) {
  const invalidate = useInvalidate("backup-runs", target);
  return useMutation({
    mutationFn: (runId: string) =>
      unwrapEmpty(api.DELETE("/api/v1/backup-runs/{id}", { params: { path: { id: runId } } })),
    onSuccess: invalidate,
  });
}
