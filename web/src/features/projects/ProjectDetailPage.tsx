import type { ReactNode } from "react";
import { ArrowLeftIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { Link, useParams } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";

import { errorMessage } from "@/shared/lib/errors";
import { InlineAlert } from "@/shared/ui";

import { projectQuery, type Project } from "./api";
import { EnvironmentList } from "./EnvironmentList";
import { ProjectSettings } from "./ProjectSettings";

export function ProjectDetailPage({ renderWorkloads }: { renderWorkloads?: (project: Project) => ReactNode }) {
  const { t } = useTranslation("projects");
  const { projectId } = useParams({ from: "/app/projects/$projectId" });
  const project = useQuery(projectQuery(projectId));

  return (
    <div className="flex flex-col gap-8">
      <Link
        to="/projects"
        className="press inline-flex w-fit items-center gap-1.5 rounded-control text-muted hover:text-text"
      >
        <ArrowLeftIcon size={16} aria-hidden />
        {t("back")}
      </Link>
      {project.isPending ? <div aria-hidden className="skeleton h-24" /> : null}
      {project.isError ? <InlineAlert tone="danger">{errorMessage(project.error, t)}</InlineAlert> : null}
      {project.data ? (
        <>
          <header className="flex flex-col gap-1">
            <h1 className="text-xl text-text">{project.data.name}</h1>
            <p className="font-mono text-[12px] text-subtle">{project.data.slug}</p>
            {project.data.description ? <p className="max-w-2xl text-muted">{project.data.description}</p> : null}
          </header>
          {renderWorkloads?.(project.data)}
          <EnvironmentList project={project.data} />
          <ProjectSettings project={project.data} />
        </>
      ) : null}
    </div>
  );
}
