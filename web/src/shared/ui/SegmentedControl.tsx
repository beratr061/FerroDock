import { useId } from "react";

import { cx } from "../lib/cx";

export interface Segment<T extends string> {
  value: T;
  label: string;
}

interface SegmentedControlProps<T extends string> {
  label: string;
  value: T;
  segments: Segment<T>[];
  onChange: (value: T) => void;
}

export function SegmentedControl<T extends string>({ label, value, segments, onChange }: SegmentedControlProps<T>) {
  const name = useId();
  return (
    <fieldset className="flex flex-col gap-1.5">
      <legend className="mb-1.5 font-medium text-text">{label}</legend>
      <div className="inline-flex w-fit flex-wrap gap-1 rounded-control border border-border bg-surface-raised p-1">
        {segments.map((segment) => (
          <label
            key={segment.value}
            className={cx(
              "press cursor-pointer rounded-control px-3 py-1.5 text-[13px] transition-colors duration-150",
              "has-focus-visible:outline-2 has-focus-visible:outline-focus",
              segment.value === value ? "bg-surface text-text" : "text-muted hover:text-text",
            )}
          >
            <input
              type="radio"
              name={name}
              value={segment.value}
              checked={segment.value === value}
              onChange={() => onChange(segment.value)}
              className="sr-only"
            />
            {segment.label}
          </label>
        ))}
      </div>
    </fieldset>
  );
}
