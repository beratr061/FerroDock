import type { ReactNode } from "react";
import { CheckCircleIcon, InfoIcon, WarningCircleIcon, WarningIcon } from "../icons";

import { cx } from "../lib/cx";

type Tone = "success" | "warning" | "danger" | "info";

const TONES: Record<Tone, string> = {
  success: "bg-success-tint border-success-border text-success",
  warning: "bg-warning-tint border-warning-border text-warning",
  danger: "bg-danger-tint border-danger-border text-danger",
  info: "bg-info-tint border-info-border text-info",
};

const ICONS = {
  success: CheckCircleIcon,
  warning: WarningIcon,
  danger: WarningCircleIcon,
  info: InfoIcon,
} as const;

interface InlineAlertProps {
  tone: Tone;
  children: ReactNode;
  className?: string;
}

export function InlineAlert({ tone, children, className }: InlineAlertProps) {
  const Icon = ICONS[tone];
  return (
    <div
      role={tone === "danger" ? "alert" : "status"}
      className={cx(
        "enter-settle flex items-start gap-2.5 rounded-control border px-3 py-2.5",
        TONES[tone],
        className,
      )}
    >
      <Icon aria-hidden size={18} weight="bold" className="mt-px shrink-0" />
      <div className="min-w-0">{children}</div>
    </div>
  );
}
