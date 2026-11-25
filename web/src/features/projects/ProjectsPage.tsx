import { useState } from "react";
import { CubeIcon, PlusIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, EmptyState, InlineAlert } from "@/shared/ui";

import { projectsQuery, type Project } from "./api";
import { CreateProjectForm } from "./CreateProjectForm";
import { ProjectRow } from "./ProjectRow";

function ListSkeleton() {
  return (
    <div aria-hidden className="flex flex-col gap-2">
      <div className="skeleton h-16" />
      <div className="skeleton h-16" />
    </div>
  );
}

export function ProjectsPage() {
  const { t } = useTranslation("projects");
  const projects = useQuery(projectsQuery);
  const navigate = useNavigate();
  const [creating, setCreating] = useState(false);

  const onCreated = (project: Project): void => {
    void navigate({ to: "/projects/$projectId", params: { projectId: project.id } });
  };

  return (
    <div className="flex flex-col gap-6">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-col gap-1">
          <h1 className="text-xl text-text">{t("title")}</h1>
          <p className="text-muted">{t("subtitle")}</p>
        </div>
        {creating ? null : (
          <Button icon={<PlusIcon size={16} weight="bold" aria-hidden />} onClick={() => setCreating(true)}>
            {t("create")}
          </Button>
        )}
      </header>
      {creating ? <CreateProjectForm onCreated={onCreated} onCancel={() => setCreating(false)} /> : null}
      {projects.isPending ? <ListSkeleton /> : null}
      {projects.isError ? <InlineAlert tone="danger">{errorMessage(projects.error, t)}</InlineAlert> : null}
      {projects.data?.length === 0 && !creating ? (
        <EmptyState
          icon={<CubeIcon size={22} aria-hidden />}
          title={t("empty.title")}
          description={t("empty.description")}
        />
      ) : null}
      {projects.data && projects.data.length > 0 ? (
        <ul className="overflow-hidden rounded-panel border border-border bg-surface">
          {projects.data.map((project) => (
            <ProjectRow key={project.id} project={project} />
          ))}
        </ul>
      ) : null}
    </div>
  );
}
