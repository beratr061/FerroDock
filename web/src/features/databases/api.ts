import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";

import { api, unwrap, unwrapEmpty, type Schemas } from "@/shared/api/client";

export type Database = Schemas["DatabaseDto"];
export type DatabaseEngine = Schemas["DatabaseEngineDto"];
export type DatabaseAction = Schemas["DatabaseActionDto"];
export type DatabaseCredentials = Schemas["DatabaseCredentialsDto"];

const DATABASES_KEY = ["databases"] as const;
const POLL_WHILE_DEPLOYING_MS = 2_000;

function pollWhileDeploying(data: Database | Database[] | undefined): number | false {
  const list = Array.isArray(data) ? data : data ? [data] : [];
  return list.some((database) => database.status === "deploying") ? POLL_WHILE_DEPLOYING_MS : false;
}

export const enginesQuery = queryOptions({
  queryKey: ["database-engines"],
  queryFn: () => unwrap(api.GET("/api/v1/database-engines")),
  staleTime: Infinity,
});

export function databasesQuery(projectId: string) {
  return queryOptions({
    queryKey: [...DATABASES_KEY, "project", projectId],
    queryFn: () => unwrap(api.GET("/api/v1/projects/{id}/databases", { params: { path: { id: projectId } } })),
    refetchInterval: (query) => pollWhileDeploying(query.state.data),
  });
}

export function databaseQuery(id: string) {
  return queryOptions({
    queryKey: [...DATABASES_KEY, id],
    queryFn: () => unwrap(api.GET("/api/v1/databases/{id}", { params: { path: { id } } })),
    refetchInterval: (query) => pollWhileDeploying(query.state.data),
    retry: false,
  });
}

export function deployLogQuery(id: string, deploying: boolean) {
  return queryOptions({
    queryKey: [...DATABASES_KEY, id, "log"],
    queryFn: () => unwrap(api.GET("/api/v1/databases/{id}/log", { params: { path: { id } } })),
    refetchInterval: deploying ? POLL_WHILE_DEPLOYING_MS : false,
  });
}

function useInvalidateDatabases() {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: DATABASES_KEY });
}

export function useCreateDatabase(environmentId: string) {
  const invalidate = useInvalidateDatabases();
  return useMutation({
    mutationFn: (body: Schemas["CreateDatabaseRequest"]) =>
      unwrap(api.POST("/api/v1/environments/{id}/databases", { params: { path: { id: environmentId } }, body })),
    onSuccess: invalidate,
  });
}

export function useDatabaseAction(id: string) {
  const invalidate = useInvalidateDatabases();
  return useMutation({
    mutationFn: (action: DatabaseAction) =>
      unwrap(api.POST("/api/v1/databases/{id}/actions", { params: { path: { id } }, body: { action } })),
    onSuccess: invalidate,
  });
}

export function useSetExternalPort(id: string) {
  const invalidate = useInvalidateDatabases();
  return useMutation({
    mutationFn: (externalPort: number | null) =>
      unwrap(api.PATCH("/api/v1/databases/{id}", { params: { path: { id } }, body: { external_port: externalPort } })),
    onSuccess: invalidate,
  });
}

export function useSetLimits(id: string) {
  const invalidate = useInvalidateDatabases();
  return useMutation({
    mutationFn: (body: Schemas["DatabaseLimitsRequest"]) =>
      unwrap(api.PUT("/api/v1/databases/{id}/limits", { params: { path: { id } }, body })),
    onSuccess: invalidate,
  });
}

export function useRevealCredentials(id: string) {
  return useMutation({
    mutationFn: () => unwrap(api.POST("/api/v1/databases/{id}/credentials", { params: { path: { id } } })),
  });
}

export function useDeleteDatabase(id: string) {
  const invalidate = useInvalidateDatabases();
  return useMutation({
    mutationFn: (removeData: boolean) =>
      unwrapEmpty(
        api.DELETE("/api/v1/databases/{id}", { params: { path: { id }, query: { remove_data: removeData } } }),
      ),
    onSuccess: invalidate,
  });
}
