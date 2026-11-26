import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";

import { api, unwrap, unwrapEmpty, type Schemas } from "@/shared/api/client";

export type Webhook = Schemas["WebhookDto"];

function webhookKey(id: string) {
  return ["applications", id, "webhook"] as const;
}

export function webhookQuery(id: string) {
  return queryOptions({
    queryKey: webhookKey(id),
    queryFn: () => unwrap(api.GET("/api/v1/applications/{id}/webhook", { params: { path: { id } } })),
  });
}

export function useRotateWebhook(id: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => unwrap(api.POST("/api/v1/applications/{id}/webhook", { params: { path: { id } } })),
    onSuccess: (created) => queryClient.setQueryData(webhookKey(id), created.webhook),
  });
}

export function useDisableWebhook(id: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => unwrapEmpty(api.DELETE("/api/v1/applications/{id}/webhook", { params: { path: { id } } })),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: webhookKey(id) }),
  });
}

export function webhookUrl(id: string): string {
  return `${window.location.origin}/api/v1/webhooks/${id}`;
}
