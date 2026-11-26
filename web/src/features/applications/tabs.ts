export const APPLICATION_TABS = ["deployments", "logs", "environment", "domains", "storage", "settings"] as const;

export type ApplicationTab = (typeof APPLICATION_TABS)[number];

export function applicationTab(value: string | undefined): ApplicationTab {
  return APPLICATION_TABS.find((tab) => tab === value) ?? "deployments";
}
