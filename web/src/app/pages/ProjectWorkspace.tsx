import { ApplicationsSection } from "@/features/applications";
import { DatabasesSection } from "@/features/databases";
import { ProjectDetailPage } from "@/features/projects";

export function ProjectWorkspace() {
  return (
    <ProjectDetailPage
      renderWorkloads={(project) => (
        <>
          <ApplicationsSection projectId={project.id} environments={project.environments} />
          <DatabasesSection projectId={project.id} environments={project.environments} />
        </>
      )}
    />
  );
}
