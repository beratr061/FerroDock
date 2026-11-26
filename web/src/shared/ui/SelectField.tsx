import { forwardRef, useEffect, useId, useRef } from "react";
import type { RefObject, SelectHTMLAttributes } from "react";
import { CaretDownIcon, CheckIcon } from "../icons";

import { cx } from "../lib/cx";
import { useListbox } from "./select/useListbox";
import { useNativeSelect } from "./select/useNativeSelect";

export interface SelectOption {
  value: string;
  label: string;
}

interface SelectFieldProps extends SelectHTMLAttributes<HTMLSelectElement> {
  label: string;
  options: SelectOption[];
  hideLabel?: boolean;
}

function useOutsideClose(open: boolean, root: RefObject<HTMLDivElement | null>, close: () => void): void {
  useEffect(() => {
    if (!open) {
      return undefined;
    }
    const onPointer = (event: PointerEvent): void => {
      if (!root.current?.contains(event.target as Node)) {
        close();
      }
    };
    document.addEventListener("pointerdown", onPointer);
    return () => document.removeEventListener("pointerdown", onPointer);
  }, [open, root, close]);
}

function Options({ id, options, value, active, onHover, onPick }: {
  id: string;
  options: SelectOption[];
  value: string;
  active: number;
  onHover: (index: number) => void;
  onPick: (index: number) => void;
}) {
  return (
    <div
      id={`${id}-listbox`}
      role="listbox"
      className="palette-enter absolute top-full left-0 z-30 mt-1 max-h-64 w-max min-w-full max-w-[min(28rem,calc(100vw-2rem))] overflow-y-auto rounded-control border border-border bg-surface-raised p-1 shadow-overlay"
    >
      {options.map((option, index) => (
        <div
          key={option.value}
          id={`${id}-option-${index}`}
          role="option"
          aria-selected={option.value === value}
          onPointerMove={() => onHover(index)}
          onPointerDown={(event) => event.preventDefault()}
          onClick={() => onPick(index)}
          className={cx(
            "flex h-8 cursor-pointer items-center justify-between gap-3 rounded-[6px] px-2.5 text-text",
            index === active && "bg-surface-active",
          )}
        >
          <span className="truncate">{option.label}</span>
          {option.value === value ? <CheckIcon aria-hidden size={14} weight="bold" className="shrink-0 text-primary" /> : null}
        </div>
      ))}
    </div>
  );
}

export const SelectField = forwardRef<HTMLSelectElement, SelectFieldProps>(function SelectField(
  { label, options, hideLabel = false, className, id, disabled, ...native },
  ref,
) {
  const generatedId = useId();
  const fieldId = id ?? generatedId;
  const root = useRef<HTMLDivElement>(null);
  const { attach, value, choose } = useNativeSelect(ref, options.map((option) => option.value).join("\u0000"));
  const shown = native.value === undefined ? value : String(native.value);
  const selected = Math.max(options.findIndex((option) => option.value === shown), 0);
  const listbox = useListbox({
    labels: options.map((option) => option.label),
    selected,
    onCommit: (index) => choose(options[index]?.value ?? ""),
  });
  useOutsideClose(listbox.open, root, listbox.close);

  return (
    <div ref={root} className={cx("relative flex flex-col gap-1.5", className)}>
      <span id={`${fieldId}-label`} className={cx("text-[13px] font-medium text-muted", hideLabel && "sr-only")}>
        {label}
      </span>
      <select ref={attach} tabIndex={-1} aria-hidden className="sr-only" disabled={disabled} {...native}>
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
      <button
        type="button"
        id={fieldId}
        role="combobox"
        aria-labelledby={`${fieldId}-label`}
        aria-controls={`${fieldId}-listbox`}
        aria-expanded={listbox.open}
        aria-activedescendant={listbox.open ? `${fieldId}-option-${listbox.active}` : undefined}
        disabled={disabled}
        onClick={() => (listbox.open ? listbox.close() : listbox.show(selected))}
        onKeyDown={listbox.onKeyDown}
        onBlur={listbox.close}
        className={cx(
          "flex h-9 w-full items-center justify-between gap-2 rounded-control border bg-surface px-3 text-left text-base text-text sm:text-sm",
          "transition-colors duration-150 ease-out focus-visible:outline-none disabled:cursor-not-allowed disabled:text-subtle",
          listbox.open ? "border-focus" : "border-border-strong hover:border-subtle focus-visible:border-focus",
        )}
      >
        <span className="truncate">{options[selected]?.label ?? ""}</span>
        <CaretDownIcon
          aria-hidden
          size={12}
          weight="bold"
          className={cx("shrink-0 text-subtle transition-transform duration-150 ease-out", listbox.open && "rotate-180")}
        />
      </button>
      {listbox.open ? (
        <Options id={fieldId} options={options} value={shown} active={listbox.active} onHover={listbox.setActive} onPick={listbox.commit} />
      ) : null}
    </div>
  );
});
