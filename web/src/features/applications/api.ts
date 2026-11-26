import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";

import { api, unwrap, unwrapEmpty, type Schemas } from "@/shared/api/client";

export type Application = Schemas["ApplicationDto"];
export type Environment = Schemas["EnvironmentDto"];
export type SourceRequest = Schemas["SourceRequest"];
export type Strategy = Schemas["StrategyDto"];
export type BuildKind = Schemas["BuildKindDto"];

const APPLICATIONS_KEY = ["applications"] as const;

export function applicationsQuery(projectId: string) {
  return queryOptions({
    queryKey: [...APPLICATIONS_KEY, "project", projectId],
    queryFn: () =>
      unwrap(api.GET("/api/v1/projects/{id}/applications", { params: { path: { id: projectId } } })),
  });
}

export function containersQuery(applicationId: string) {
  return queryOptions({
    queryKey: ["application-containers", applicationId],
    queryFn: () =>
      unwrap(api.GET("/api/v1/applications/{id}/containers", { params: { path: { id: applicationId } } })),
  });
}

export function applicationQuery(id: string) {
  return queryOptions({
    queryKey: [...APPLICATIONS_KEY, id],
    queryFn: () => unwrap(api.GET("/api/v1/applications/{id}", { params: { path: { id } } })),
    retry: false,
  });
}

function useInvalidateApplications() {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: APPLICATIONS_KEY });
}

export function useCreateApplication(environmentId: string) {
  const invalidate = useInvalidateApplications();
  return useMutation({
    mutationFn: (body: Schemas["CreateApplicationRequest"]) =>
      unwrap(
        api.POST("/api/v1/environments/{id}/applications", {
          params: { path: { id: environmentId } },
          body,
        }),
      ),
    onSuccess: invalidate,
  });
}

export function useUpdateApplication(id: string) {
  const invalidate = useInvalidateApplications();
  return useMutation({
    mutationFn: (body: Schemas["UpdateApplicationRequest"]) =>
      unwrap(api.PATCH("/api/v1/applications/{id}", { params: { path: { id } }, body })),
    onSuccess: invalidate,
  });
}

export function useRotateDeployKey(id: string) {
  const invalidate = useInvalidateApplications();
  return useMutation({
    mutationFn: () => unwrap(api.POST("/api/v1/applications/{id}/deploy-key", { params: { path: { id } } })),
    onSuccess: invalidate,
  });
}

export function useSetComposeTrust(id: string) {
  const invalidate = useInvalidateApplications();
  return useMutation({
    mutationFn: (trusted: boolean) =>
      unwrap(api.PUT("/api/v1/applications/{id}/compose-trust", { params: { path: { id } }, body: { trusted } })),
    onSuccess: invalidate,
  });
}

export function useDeleteApplication(id: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (removeData: boolean) =>
      unwrapEmpty(api.DELETE("/api/v1/applications/{id}", { params: { path: { id }, query: { remove_data: removeData } } })),
    onSuccess: () => {
      queryClient.removeQueries({ queryKey: [...APPLICATIONS_KEY, id] });
      return queryClient.invalidateQueries({ queryKey: [...APPLICATIONS_KEY, "project"] });
    },
  });
}
