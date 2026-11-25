import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";

import { api, unwrap, type Schemas } from "@/shared/api/client";
import { isRealtimeConnected } from "@/shared/realtime";

export type Deployment = Schemas["DeploymentDto"];
export type DeploymentStatus = Schemas["DeploymentStatusDto"];

const ACTIVE_REFRESH_MS = 2_000;

export function isActive(status: DeploymentStatus): boolean {
  return status === "queued" || status === "running";
}

export function deploymentsQuery(applicationId: string) {
  return queryOptions({
    queryKey: ["deployments", applicationId],
    queryFn: () =>
      unwrap(api.GET("/api/v1/applications/{id}/deployments", { params: { path: { id: applicationId } } })),
    refetchInterval: (query) => {
      const active = query.state.data?.some((deployment) => isActive(deployment.status)) ?? false;
      return active && !isRealtimeConnected() ? ACTIVE_REFRESH_MS : false;
    },
  });
}

export function fetchDeploymentLog(id: string, offset: number) {
  return unwrap(api.GET("/api/v1/deployments/{id}/log", { params: { path: { id }, query: { offset } } }));
}

function useInvalidateDeployments(applicationId: string) {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: ["deployments", applicationId] });
}

export function useDeploy(applicationId: string) {
  const invalidate = useInvalidateDeployments(applicationId);
  return useMutation({
    mutationFn: () =>
      unwrap(api.POST("/api/v1/applications/{id}/deployments", { params: { path: { id: applicationId } } })),
    onSuccess: invalidate,
  });
}

export function useRollback(applicationId: string) {
  const invalidate = useInvalidateDeployments(applicationId);
  return useMutation({
    mutationFn: (id: string) => unwrap(api.POST("/api/v1/deployments/{id}/rollback", { params: { path: { id } } })),
    onSuccess: invalidate,
  });
}

export function useCancelDeployment(applicationId: string) {
  const invalidate = useInvalidateDeployments(applicationId);
  return useMutation({
    mutationFn: (id: string) => unwrap(api.POST("/api/v1/deployments/{id}/cancel", { params: { path: { id } } })),
    onSuccess: invalidate,
  });
}
