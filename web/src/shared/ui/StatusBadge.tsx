import { cx } from "../lib/cx";

export type StatusTone = "success" | "warning" | "danger" | "info" | "neutral";

const TONES: Record<StatusTone, string> = {
  success: "bg-success",
  warning: "bg-warning",
  danger: "bg-danger",
  info: "bg-info",
  neutral: "bg-subtle",
};

interface StatusBadgeProps {
  tone: StatusTone;
  label: string;
}

export function StatusBadge({ tone, label }: StatusBadgeProps) {
  return (
    <span
      className={cx(
        "inline-flex h-[22px] items-center gap-1.5 rounded-pill border border-border bg-surface-raised px-2",
        "text-xs whitespace-nowrap text-muted",
      )}
    >
      <span aria-hidden className={cx("size-1.5 rounded-full transition-colors duration-200 ease-out", TONES[tone])} />
      {label}
    </span>
  );
}
