import type { ReactNode } from "react";

interface EmptyStateProps {
  icon: ReactNode;
  title: string;
  description: string;
  action?: ReactNode;
}

export function EmptyState({ icon, title, description, action }: EmptyStateProps) {
  return (
    <div className="enter-settle flex max-w-md flex-col items-start gap-4 py-16">
      <span className="grid size-11 place-items-center rounded-panel border border-border bg-surface text-muted">
        {icon}
      </span>
      <div className="flex flex-col gap-1.5">
        <h2 className="text-lg text-text">{title}</h2>
        <p className="text-muted">{description}</p>
      </div>
      {action}
    </div>
  );
}
