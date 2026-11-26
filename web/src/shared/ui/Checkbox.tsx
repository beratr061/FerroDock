import { forwardRef } from "react";
import type { InputHTMLAttributes, ReactNode } from "react";
import { CheckIcon } from "../icons";

import { cx } from "../lib/cx";

interface CheckboxProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "type"> {
  label: ReactNode;
}

export const Checkbox = forwardRef<HTMLInputElement, CheckboxProps>(function Checkbox(
  { label, className, ...rest },
  ref,
) {
  return (
    <label className={cx("inline-flex cursor-pointer items-center gap-2.5 text-text select-none", className)}>
      <span className="relative grid size-4 shrink-0 place-items-center">
        <input
          ref={ref}
          type="checkbox"
          className={cx(
            "peer absolute inset-0 m-0 cursor-pointer appearance-none rounded-[4px] border border-border-strong bg-surface",
            "transition-colors duration-150 ease-out hover:border-subtle checked:border-primary checked:bg-primary",
            "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus disabled:cursor-not-allowed",
          )}
          {...rest}
        />
        <CheckIcon
          aria-hidden
          size={11}
          weight="bold"
          className="pointer-events-none relative scale-0 text-on-primary transition-transform duration-150 ease-out peer-checked:scale-100"
        />
      </span>
      {label}
    </label>
  );
});
