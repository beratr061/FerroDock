import type { ButtonHTMLAttributes, ReactNode } from "react";

import { cx } from "../lib/cx";

type Variant = "primary" | "secondary" | "ghost" | "danger";

const VARIANTS: Record<Variant, string> = {
  primary: "bg-primary text-on-primary hover:bg-primary-hover",
  secondary: "bg-surface text-text border border-border-strong hover:bg-surface-hover",
  ghost: "text-muted hover:bg-surface-hover hover:text-text",
  danger: "bg-danger-tint text-danger border border-danger-border hover:bg-surface-hover",
};

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  busy?: boolean;
  icon?: ReactNode;
}

export function Button({
  variant = "primary",
  busy = false,
  icon,
  className,
  children,
  disabled,
  type = "button",
  ...rest
}: ButtonProps) {
  return (
    <button
      type={type}
      disabled={disabled === true || busy}
      aria-busy={busy}
      className={cx(
        "press inline-flex h-9 items-center justify-center gap-2 rounded-control px-3.5",
        "font-medium whitespace-nowrap select-none disabled:cursor-not-allowed",
        "disabled:opacity-100 disabled:bg-surface-raised disabled:text-subtle disabled:border-border",
        VARIANTS[variant],
        className,
      )}
      {...rest}
    >
      {icon}
      {children}
    </button>
  );
}
