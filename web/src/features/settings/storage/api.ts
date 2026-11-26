import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";

import { api, unwrap, unwrapEmpty, type Schemas } from "@/shared/api/client";

export type Destination = Schemas["BackupDestinationDto"];
export type NewDestination = Schemas["CreateBackupDestinationRequest"];

const KEY = ["backup-destinations"] as const;

export const destinationsQuery = queryOptions({
  queryKey: KEY,
  queryFn: () => unwrap(api.GET("/api/v1/backup-destinations")),
});

function useInvalidate() {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: KEY });
}

export function useCreateDestination() {
  const invalidate = useInvalidate();
  return useMutation({
    mutationFn: (body: NewDestination) => unwrap(api.POST("/api/v1/backup-destinations", { body })),
    onSuccess: invalidate,
  });
}

export function useTestDestination(id: string) {
  return useMutation({
    mutationFn: () =>
      unwrapEmpty(api.POST("/api/v1/backup-destinations/{id}/test", { params: { path: { id } } })),
  });
}

export function useDeleteDestination(id: string) {
  const invalidate = useInvalidate();
  return useMutation({
    mutationFn: () => unwrapEmpty(api.DELETE("/api/v1/backup-destinations/{id}", { params: { path: { id } } })),
    onSuccess: invalidate,
  });
}
