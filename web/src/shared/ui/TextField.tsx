import { forwardRef, useId } from "react";
import type { InputHTMLAttributes } from "react";

import { cx } from "../lib/cx";

interface TextFieldProps extends InputHTMLAttributes<HTMLInputElement> {
  label: string;
  hint?: string;
  error?: string | undefined;
}

export const TextField = forwardRef<HTMLInputElement, TextFieldProps>(function TextField(
  { label, hint, error, className, id, ...rest },
  ref,
) {
  const generatedId = useId();
  const inputId = id ?? generatedId;
  const messageId = `${inputId}-message`;
  const message = error ?? hint;
  return (
    <div className={cx("flex flex-col gap-1.5", className)}>
      <label htmlFor={inputId} className="font-medium text-text">
        {label}
      </label>
      <input
        ref={ref}
        id={inputId}
        aria-invalid={error ? true : undefined}
        aria-describedby={message ? messageId : undefined}
        className={cx(
          "h-10 rounded-control border bg-surface px-3 text-text placeholder:text-subtle",
          "transition-colors duration-200 ease-out",
          error ? "border-danger" : "border-border-strong hover:border-text focus:border-text",
        )}
        {...rest}
      />
      {message ? (
        <p id={messageId} className={cx("text-[13px]", error ? "text-danger" : "text-subtle")}>
          {message}
        </p>
      ) : null}
    </div>
  );
});
