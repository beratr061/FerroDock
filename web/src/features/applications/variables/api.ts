import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";

import { api, unwrap, type Schemas } from "@/shared/api/client";

export type Variable = Schemas["VariableDto"];
export type VariableInput = Schemas["VariableInputDto"];

export function variablesQuery(applicationId: string) {
  return queryOptions({
    queryKey: ["variables", applicationId],
    queryFn: () =>
      unwrap(api.GET("/api/v1/applications/{id}/variables", { params: { path: { id: applicationId } } })),
  });
}

export function useSaveVariables(applicationId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (variables: VariableInput[]) =>
      unwrap(
        api.PUT("/api/v1/applications/{id}/variables", {
          params: { path: { id: applicationId } },
          body: { variables },
        }),
      ),
    onSuccess: (saved) => queryClient.setQueryData(["variables", applicationId], saved),
  });
}

export async function revealVariable(applicationId: string, key: string): Promise<string> {
  const revealed = await unwrap(
    api.POST("/api/v1/applications/{id}/variables/{key}/reveal", {
      params: { path: { id: applicationId, key } },
    }),
  );
  return revealed.value;
}
