import { socketUrl } from "@/shared/logs";

export function logsUrl(applicationId: string, container: string): string {
  return socketUrl(`/api/v1/applications/${encodeURIComponent(applicationId)}/logs`, { container });
}
