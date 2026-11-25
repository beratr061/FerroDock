import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";

import { api, unwrap, unwrapEmpty, type Schemas } from "@/shared/api/client";

export type CurrentUser = Schemas["MeResponse"];

export const setupStatusQuery = queryOptions({
  queryKey: ["setup-status"],
  queryFn: () => unwrap(api.GET("/api/v1/setup")),
  staleTime: Number.POSITIVE_INFINITY,
});

export const meQuery = queryOptions({
  queryKey: ["me"],
  queryFn: () => unwrap(api.GET("/api/v1/auth/me")),
  staleTime: 60_000,
  retry: false,
});

export function useSignIn() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: Schemas["SignInRequest"]) =>
      unwrap(api.POST("/api/v1/auth/sign-in", { body })),
    onSuccess: (user) => queryClient.setQueryData(meQuery.queryKey, user),
  });
}

export function useCompleteSetup() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: Schemas["SetupRequest"]) => unwrap(api.POST("/api/v1/setup", { body })),
    onSuccess: (user) => {
      queryClient.setQueryData(setupStatusQuery.queryKey, { required: false, token_required: false });
      queryClient.setQueryData(meQuery.queryKey, user);
    },
  });
}

export function useSignOut() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => unwrapEmpty(api.POST("/api/v1/auth/sign-out")),
    onSettled: () => queryClient.clear(),
  });
}
