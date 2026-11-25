import { cx } from "../lib/cx";

interface LogoProps {
  compact?: boolean;
  className?: string;
}

export function Logo({ compact = false, className }: LogoProps) {
  return (
    <span className={cx("inline-flex items-center gap-2.5 text-text", className)}>
      <svg aria-hidden viewBox="0 0 32 32" className="size-7 shrink-0">
        <rect width="32" height="32" rx="7" className="fill-primary" />
        <path d="M9 8h14v4H13v3h8v4h-8v5H9z" className="fill-on-primary" />
      </svg>
      {compact ? null : <span className="text-[15px] font-semibold tracking-tight">FerroDock</span>}
    </span>
  );
}
