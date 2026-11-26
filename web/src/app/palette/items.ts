import { useEffect, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";

import type { PaletteSources } from "./PaletteHost";

export type PaletteKind = "page" | "project" | "application" | "database";

export interface PaletteItem {
  id: string;
  kind: PaletteKind;
  label: string;
  detail: string;
  to: string;
}

const PAGES: { key: string; to: string }[] = [
  { key: "nav.projects", to: "/projects" },
  { key: "nav.servers", to: "/servers" },
  { key: "palette.pages.apiKeys", to: "/settings/api-keys" },
  { key: "palette.pages.backups", to: "/settings/backups" },
  { key: "palette.pages.storage", to: "/settings/storage" },
  { key: "palette.pages.updates", to: "/settings/updates" },
  { key: "palette.pages.appearance", to: "/settings/appearance" },
];

function normalize(text: string): string {
  return text.toLocaleLowerCase().normalize("NFKD").replace(/\p{Diacritic}/gu, "");
}

export function matches(item: PaletteItem, query: string): boolean {
  const words = normalize(query).split(/\s+/).filter(Boolean);
  const haystack = normalize(`${item.label} ${item.detail}`);
  return words.every((word) => haystack.includes(word));
}

type QueryClient = ReturnType<typeof useQueryClient>;

async function projectItems(client: QueryClient, sources: PaletteSources, project: { id: string; name: string; slug: string }) {
  const [applications, databases] = await Promise.all([
    client.fetchQuery({ queryKey: ["applications", "project", project.id], queryFn: () => sources.applications(project.id) }),
    client.fetchQuery({ queryKey: ["databases", "project", project.id], queryFn: () => sources.databases(project.id) }),
  ]);
  const own: PaletteItem = { id: project.id, kind: "project", label: project.name, detail: project.slug, to: `/projects/${project.id}` };
  return [
    own,
    ...applications.map((application): PaletteItem => ({
      id: application.id,
      kind: "application",
      label: application.name,
      detail: project.name,
      to: `/applications/${application.id}`,
    })),
    ...databases.map((database): PaletteItem => ({
      id: database.id,
      kind: "database",
      label: database.name,
      detail: `${project.name} · ${database.engine}`,
      to: `/databases/${database.id}`,
    })),
  ];
}

async function loadAll(client: QueryClient, sources: PaletteSources): Promise<PaletteItem[]> {
  const projects = await client.fetchQuery({ queryKey: ["projects"], queryFn: sources.projects });
  const nested = await Promise.all(projects.map((project) => projectItems(client, sources, project)));
  return nested.flat();
}

export function usePaletteItems(sources: PaletteSources, t: (key: string) => string): PaletteItem[] {
  const client = useQueryClient();
  const [loaded, setLoaded] = useState<PaletteItem[]>([]);

  useEffect(() => {
    let current = true;
    loadAll(client, sources)
      .then((items) => current && setLoaded(items))
      .catch(() => current && setLoaded([]));
    return () => {
      current = false;
    };
  }, [client, sources]);

  const pages = PAGES.map((page): PaletteItem => ({ id: page.to, kind: "page", label: t(page.key), detail: page.to, to: page.to }));
  return [...pages, ...loaded];
}
