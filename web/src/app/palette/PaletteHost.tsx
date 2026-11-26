import { lazy, Suspense, useEffect, useState } from "react";

import { api, unwrap } from "@/shared/api/client";

const CommandPalette = lazy(() => import("./CommandPalette"));

const sources = {
  projects: () => unwrap(api.GET("/api/v1/projects")),
  applications: (id: string) => unwrap(api.GET("/api/v1/projects/{id}/applications", { params: { path: { id } } })),
  databases: (id: string) => unwrap(api.GET("/api/v1/projects/{id}/databases", { params: { path: { id } } })),
};

export type PaletteSources = typeof sources;

export function PaletteHost() {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    const onKey = (event: KeyboardEvent): void => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setOpen((current) => !current);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  if (!open) {
    return null;
  }
  return (
    <Suspense fallback={null}>
      <CommandPalette sources={sources} onClose={() => setOpen(false)} />
    </Suspense>
  );
}
