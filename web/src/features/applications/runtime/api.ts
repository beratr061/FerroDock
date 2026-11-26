import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";

import { api, unwrap, type Schemas } from "@/shared/api/client";

export type RuntimeConfig = Schemas["RuntimeConfigDto"];
export type MountKind = Schemas["MountKindDto"];
export type Protocol = Schemas["ProtocolDto"];

export function runtimeQuery(applicationId: string) {
  return queryOptions({
    queryKey: ["runtime", applicationId],
    queryFn: () =>
      unwrap(api.GET("/api/v1/applications/{id}/runtime", { params: { path: { id: applicationId } } })),
  });
}

export function useSaveRuntime(applicationId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: RuntimeConfig) =>
      unwrap(api.PUT("/api/v1/applications/{id}/runtime", { params: { path: { id: applicationId } }, body })),
    onSuccess: (saved) => queryClient.setQueryData(["runtime", applicationId], saved),
  });
}
