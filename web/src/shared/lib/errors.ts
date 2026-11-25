import type { TFunction } from "i18next";

import { ApiError } from "../api/client";

export function errorMessage(error: unknown, t: TFunction): string {
  if (error instanceof ApiError) {
    const reason = typeof error.params["reason"] === "string" ? error.params["reason"] : undefined;
    const key = reason ?? error.code;
    return t(`errors.${key}`, { ...error.params, defaultValue: t("errors.UNKNOWN") });
  }
  return t("errors.NETWORK");
}
