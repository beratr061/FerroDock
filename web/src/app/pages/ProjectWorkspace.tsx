import { ApplicationsSection } from "@/features/applications";
import { ProjectDetailPage } from "@/features/projects";

export function ProjectWorkspace() {
  return (
    <ProjectDetailPage
      renderWorkloads={(project) => (
        <ApplicationsSection projectId={project.id} environments={project.environments} />
      )}
    />
  );
}
