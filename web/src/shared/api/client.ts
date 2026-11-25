import createClient from "openapi-fetch";

import type { components, paths } from "./schema";

export type Schemas = components["schemas"];

export const api = createClient<paths>({ baseUrl: "", credentials: "same-origin" });

export class ApiError extends Error {
  readonly code: string;
  readonly params: Record<string, unknown>;
  readonly status: number;

  constructor(code: string, status: number, params: Record<string, unknown> = {}) {
    super(code);
    this.code = code;
    this.status = status;
    this.params = params;
  }
}

function isEnvelope(value: unknown): value is Schemas["ErrorEnvelope"] {
  return typeof value === "object" && value !== null && "error" in value;
}

function toApiError(error: unknown, status: number): ApiError {
  if (isEnvelope(error)) {
    return new ApiError(error.error.code, status, error.error.params);
  }
  return new ApiError(status === 429 ? "RATE_LIMITED" : "NETWORK", status);
}

interface FetchResult<T> {
  data?: T;
  error?: unknown;
  response: Response;
}

export async function unwrap<T>(request: Promise<FetchResult<T>>): Promise<T> {
  const { data, error, response } = await request;
  if (response.ok && data !== undefined) {
    return data;
  }
  throw toApiError(error, response.status);
}

export async function unwrapEmpty(request: Promise<FetchResult<unknown>>): Promise<void> {
  const { error, response } = await request;
  if (!response.ok) {
    throw toApiError(error, response.status);
  }
}
