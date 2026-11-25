import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";

import { api, unwrap, unwrapEmpty, type Schemas } from "@/shared/api/client";

export type Project = Schemas["ProjectDto"];
export type Environment = Schemas["EnvironmentDto"];

const PROJECTS_KEY = ["projects"] as const;

export const projectsQuery = queryOptions({
  queryKey: PROJECTS_KEY,
  queryFn: () => unwrap(api.GET("/api/v1/projects")),
});

export function projectQuery(id: string) {
  return queryOptions({
    queryKey: [...PROJECTS_KEY, id],
    queryFn: () => unwrap(api.GET("/api/v1/projects/{id}", { params: { path: { id } } })),
    retry: false,
  });
}

function useInvalidateProjects() {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: PROJECTS_KEY });
}

export function useCreateProject() {
  const invalidate = useInvalidateProjects();
  return useMutation({
    mutationFn: (body: Schemas["CreateProjectRequest"]) => unwrap(api.POST("/api/v1/projects", { body })),
    onSuccess: invalidate,
  });
}

export function useUpdateProject(id: string) {
  const invalidate = useInvalidateProjects();
  return useMutation({
    mutationFn: (body: Schemas["UpdateProjectRequest"]) =>
      unwrap(api.PATCH("/api/v1/projects/{id}", { params: { path: { id } }, body })),
    onSuccess: invalidate,
  });
}

export function useDeleteProject(id: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => unwrapEmpty(api.DELETE("/api/v1/projects/{id}", { params: { path: { id } } })),
    onSuccess: () => {
      queryClient.removeQueries({ queryKey: [...PROJECTS_KEY, id] });
      return queryClient.invalidateQueries({ queryKey: PROJECTS_KEY, exact: true });
    },
  });
}

export function useAddEnvironment(projectId: string) {
  const invalidate = useInvalidateProjects();
  return useMutation({
    mutationFn: (name: string) =>
      unwrap(
        api.POST("/api/v1/projects/{id}/environments", {
          params: { path: { id: projectId } },
          body: { name },
        }),
      ),
    onSuccess: invalidate,
  });
}

export function useRemoveEnvironment(projectId: string) {
  const invalidate = useInvalidateProjects();
  return useMutation({
    mutationFn: (environmentId: string) =>
      unwrapEmpty(
        api.DELETE("/api/v1/projects/{id}/environments/{environment_id}", {
          params: { path: { id: projectId, environment_id: environmentId } },
        }),
      ),
    onSuccess: invalidate,
  });
}
