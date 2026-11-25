import { CaretRightIcon } from "@phosphor-icons/react";
import { Link } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";

import type { Project } from "./api";

export function ProjectRow({ project }: { project: Project }) {
  const { t } = useTranslation("projects");
  return (
    <li className="border-b border-border last:border-b-0">
      <Link
        to="/projects/$projectId"
        params={{ projectId: project.id }}
        className="group grid gap-x-6 gap-y-2 px-4 py-3.5 hover:bg-surface-hover md:grid-cols-[minmax(0,2fr)_minmax(0,3fr)_auto] md:items-center"
      >
        <div className="min-w-0">
          <p className="truncate font-medium text-text">{project.name}</p>
          <p className="truncate font-mono text-[12px] text-subtle">{project.slug}</p>
        </div>
        <p className="truncate text-muted">{project.description || t("noDescription")}</p>
        <div className="flex items-center gap-3 md:justify-end">
          <ul className="flex flex-wrap gap-1.5" aria-label={t("environments.title")}>
            {project.environments.map((environment) => (
              <li
                key={environment.id}
                className="rounded-control border border-border bg-surface-raised px-2 py-0.5 font-mono text-[12px] text-muted"
              >
                {environment.slug}
              </li>
            ))}
          </ul>
          <CaretRightIcon size={16} aria-hidden className="text-subtle group-hover:text-text" />
        </div>
      </Link>
    </li>
  );
}
