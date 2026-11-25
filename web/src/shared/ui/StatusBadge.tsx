import { cx } from "../lib/cx";

export type StatusTone = "success" | "warning" | "danger" | "info" | "neutral";

const TONES: Record<StatusTone, string> = {
  success: "bg-success-tint text-success border-success-border",
  warning: "bg-warning-tint text-warning border-warning-border",
  danger: "bg-danger-tint text-danger border-danger-border",
  info: "bg-info-tint text-info border-info-border",
  neutral: "bg-surface-raised text-muted border-border",
};

interface StatusBadgeProps {
  tone: StatusTone;
  label: string;
  live?: boolean;
}

export function StatusBadge({ tone, label, live = false }: StatusBadgeProps) {
  return (
    <span
      className={cx(
        "inline-flex h-6 items-center gap-1.5 rounded-control border px-2 text-[12px] font-medium whitespace-nowrap",
        "transition-colors duration-200 ease-out",
        TONES[tone],
      )}
    >
      {live ? <span aria-hidden className="size-1.5 rounded-full bg-current" /> : null}
      {label}
    </span>
  );
}
