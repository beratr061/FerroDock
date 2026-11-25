import * as v from "valibot";

export const PASSWORD_MIN = 12;

export const setupSchema = v.object({
  name: v.pipe(v.string(), v.trim(), v.nonEmpty("validation.required"), v.maxLength(100, "validation.tooLong")),
  email: v.pipe(v.string(), v.trim(), v.email("validation.email")),
  organizationName: v.pipe(
    v.string(),
    v.trim(),
    v.nonEmpty("validation.required"),
    v.maxLength(100, "validation.tooLong"),
  ),
  password: v.pipe(
    v.string(),
    v.minLength(PASSWORD_MIN, "validation.passwordMin"),
    v.maxLength(128, "validation.tooLong"),
  ),
  setupToken: v.optional(v.pipe(v.string(), v.trim())),
});

export type SetupValues = v.InferOutput<typeof setupSchema>;
