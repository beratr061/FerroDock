import { forwardRef, useId } from "react";
import type { SelectHTMLAttributes } from "react";
import { CaretDownIcon } from "@phosphor-icons/react";

import { cx } from "../lib/cx";

export interface SelectOption {
  value: string;
  label: string;
}

interface SelectFieldProps extends SelectHTMLAttributes<HTMLSelectElement> {
  label: string;
  options: SelectOption[];
  hideLabel?: boolean;
}

export const SelectField = forwardRef<HTMLSelectElement, SelectFieldProps>(function SelectField(
  { label, options, hideLabel = false, className, id, ...rest },
  ref,
) {
  const generatedId = useId();
  const selectId = id ?? generatedId;
  return (
    <div className={cx("flex flex-col gap-1.5", className)}>
      <label htmlFor={selectId} className={cx("font-medium text-text", hideLabel && "sr-only")}>
        {label}
      </label>
      <div className="relative">
        <select
          ref={ref}
          id={selectId}
          className={cx(
            "h-10 w-full appearance-none rounded-control border border-border-strong",
            "bg-surface pr-9 pl-3 text-text hover:border-text focus:border-text",
          )}
          {...rest}
        >
          {options.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
        <CaretDownIcon
          aria-hidden
          size={14}
          weight="bold"
          className="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-muted"
        />
      </div>
    </div>
  );
});
