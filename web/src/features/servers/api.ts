import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";

import { api, unwrap, unwrapEmpty, type Schemas } from "@/shared/api/client";

export type Container = Schemas["ContainerDto"];
export type ContainerAction = Schemas["ContainerActionDto"];

const REFRESH_INTERVAL_MS = 5_000;

export const engineInfoQuery = queryOptions({
  queryKey: ["docker", "info"],
  queryFn: () => unwrap(api.GET("/api/v1/docker/info")),
  staleTime: 60_000,
  retry: false,
});

export function containersQuery(includeStopped: boolean) {
  return queryOptions({
    queryKey: ["docker", "containers", includeStopped],
    queryFn: () =>
      unwrap(api.GET("/api/v1/docker/containers", { params: { query: { all: includeStopped } } })),
    refetchInterval: REFRESH_INTERVAL_MS,
    retry: false,
  });
}

export function useContainerAction() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ reference, action }: { reference: string; action: ContainerAction }) =>
      unwrapEmpty(
        api.POST("/api/v1/docker/containers/{reference}/actions", {
          params: { path: { reference } },
          body: { action },
        }),
      ),
    onSettled: () => queryClient.invalidateQueries({ queryKey: ["docker", "containers"] }),
  });
}
