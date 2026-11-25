import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";

import { api, unwrap, unwrapEmpty, type Schemas } from "@/shared/api/client";

export type ApiKey = Schemas["ApiKeyDto"];
export type Role = Schemas["RoleDto"];

export const apiKeysQuery = queryOptions({
  queryKey: ["api-keys"],
  queryFn: () => unwrap(api.GET("/api/v1/api-keys")),
});

export function useCreateApiKey() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: Schemas["CreateApiKeyRequest"]) =>
      unwrap(api.POST("/api/v1/api-keys", { body })),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: apiKeysQuery.queryKey }),
  });
}

export function useRevokeApiKey() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) =>
      unwrapEmpty(api.DELETE("/api/v1/api-keys/{id}", { params: { path: { id } } })),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: apiKeysQuery.queryKey }),
  });
}
