import type { Application, BuildKind, SourceRequest, Strategy } from "./api";

export type SourceKind = "git" | "image";

export interface SourceFormValues {
  kind: SourceKind;
  url: string;
  branch: string;
  buildKind: BuildKind;
  buildPath: string;
  image: string;
  strategy: Strategy;
}

export const EMPTY_SOURCE: SourceFormValues = {
  kind: "git",
  url: "",
  branch: "main",
  buildKind: "dockerfile",
  buildPath: "",
  image: "",
  strategy: "recreate",
};

function optional(value: string): string | null {
  const trimmed = value.trim();
  return trimmed === "" ? null : trimmed;
}

export function toSourceRequest(values: SourceFormValues): SourceRequest {
  if (values.kind === "image") {
    return { kind: "image", image: values.image.trim() };
  }
  return {
    kind: "git",
    url: values.url.trim(),
    branch: optional(values.branch),
    build_kind: values.buildKind,
    build_path: optional(values.buildPath),
    build_context: null,
  };
}

export function sourceValues(application: Application): SourceFormValues {
  const source = application.source;
  if (source.kind === "image") {
    return { ...EMPTY_SOURCE, kind: "image", image: source.image, strategy: application.strategy };
  }
  return {
    ...EMPTY_SOURCE,
    url: source.url,
    branch: source.branch,
    buildKind: source.build_kind,
    buildPath: source.build_path,
    strategy: application.strategy,
  };
}

export function sourceSummary(application: Application): string {
  const source = application.source;
  return source.kind === "image" ? source.image : `${source.url} · ${source.branch}`;
}

export function isCompose(application: Application): boolean {
  return application.source.kind === "git" && application.source.build_kind === "compose";
}
