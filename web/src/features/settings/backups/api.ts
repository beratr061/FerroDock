import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";

import { api, unwrap, unwrapEmpty, type Schemas } from "@/shared/api/client";

export type Backup = Schemas["BackupDto"];

const BACKUPS_KEY = ["system", "backups"] as const;
const PENDING_REFRESH_MS = 2_000;

export function backupsQuery(waitingForNew: boolean) {
  return queryOptions({
    queryKey: BACKUPS_KEY,
    queryFn: () => unwrap(api.GET("/api/v1/system/backups")),
    retry: false,
    refetchInterval: waitingForNew ? PENDING_REFRESH_MS : false,
  });
}

export function backupDownloadUrl(name: string): string {
  return `/api/v1/system/backups/${encodeURIComponent(name)}`;
}

export function useCreateBackup() {
  return useMutation({
    mutationFn: () => unwrap(api.POST("/api/v1/system/backups")),
  });
}

export function useDeleteBackup() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (name: string) =>
      unwrapEmpty(api.DELETE("/api/v1/system/backups/{name}", { params: { path: { name } } })),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: BACKUPS_KEY }),
  });
}

export function useExportMasterKey() {
  return useMutation({
    mutationFn: (password: string) => unwrap(api.POST("/api/v1/system/master-key", { body: { password } })),
  });
}
