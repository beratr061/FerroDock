import type { StatusTone } from "@/shared/ui";

import type { Database } from "./api";

const ENGINE_LABELS: Record<string, string> = {
  postgres: "PostgreSQL",
  mysql: "MySQL",
  mariadb: "MariaDB",
  mongo: "MongoDB",
  redis: "Redis",
  valkey: "Valkey",
};

export function engineLabel(engine: string): string {
  return ENGINE_LABELS[engine] ?? engine;
}

export function engineSummary(database: Database): string {
  return `${engineLabel(database.engine)} ${database.version}`;
}

export function statusTone(database: Database): StatusTone {
  if (database.status === "failed") {
    return "danger";
  }
  if (database.status === "deploying") {
    return "info";
  }
  return database.running ? "success" : "neutral";
}

export function statusKey(database: Database): string {
  if (database.status === "running" && !database.running) {
    return "status.down";
  }
  return `status.${database.status}`;
}

export function internalAddress(database: Database): string {
  return `${database.internal_host}:${database.internal_port}`;
}
