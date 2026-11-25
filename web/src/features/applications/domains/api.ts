import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";

import { api, unwrap, unwrapEmpty, type Schemas } from "@/shared/api/client";

export type Domain = Schemas["DomainDto"];
export type DnsVerdict = Schemas["DnsVerdictDto"];

export function domainsQuery(applicationId: string) {
  return queryOptions({
    queryKey: ["domains", applicationId],
    queryFn: () =>
      unwrap(api.GET("/api/v1/applications/{id}/domains", { params: { path: { id: applicationId } } })),
  });
}

function useInvalidateDomains(applicationId: string) {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: ["domains", applicationId] });
}

export function useAddDomain(applicationId: string) {
  const invalidate = useInvalidateDomains(applicationId);
  return useMutation({
    mutationFn: (body: Schemas["CreateDomainRequest"]) =>
      unwrap(api.POST("/api/v1/applications/{id}/domains", { params: { path: { id: applicationId } }, body })),
    onSuccess: invalidate,
  });
}

export function useRemoveDomain(applicationId: string) {
  const invalidate = useInvalidateDomains(applicationId);
  return useMutation({
    mutationFn: (domainId: string) =>
      unwrapEmpty(
        api.DELETE("/api/v1/applications/{id}/domains/{domain_id}", {
          params: { path: { id: applicationId, domain_id: domainId } },
        }),
      ),
    onSuccess: invalidate,
  });
}

export function useCheckDns(applicationId: string) {
  return useMutation({
    mutationFn: (domainId: string) =>
      unwrap(
        api.GET("/api/v1/applications/{id}/domains/{domain_id}/dns", {
          params: { path: { id: applicationId, domain_id: domainId } },
        }),
      ),
  });
}
