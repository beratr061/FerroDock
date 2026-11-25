import { useState } from "react";

import type { Variable, VariableInput } from "./api";
import { parseEnvFile, serializeEnvFile } from "./envFile";

export interface DraftRow {
  id: number;
  key: string;
  value: string | null;
  secret: boolean;
}

let nextId = 0;

function toRow(variable: Pick<Variable, "key" | "value" | "secret">): DraftRow {
  nextId += 1;
  return { id: nextId, key: variable.key, value: variable.value ?? null, secret: variable.secret };
}

function fingerprint(rows: Pick<DraftRow, "key" | "value" | "secret">[]): string {
  return JSON.stringify(rows.map(({ key, value, secret }) => [key, value ?? null, secret]));
}

export function useVariableDraft(saved: Variable[]) {
  const [base, setBase] = useState(saved);
  const [rows, setRows] = useState<DraftRow[]>(() => saved.map(toRow));
  if (base !== saved) {
    setBase(saved);
    setRows(saved.map(toRow));
  }

  const update = (id: number, change: Partial<DraftRow>): void =>
    setRows((current) => current.map((row) => (row.id === id ? { ...row, ...change } : row)));
  const remove = (id: number): void => setRows((current) => current.filter((row) => row.id !== id));
  const add = (): void => setRows((current) => [...current, toRow({ key: "", value: "", secret: false })]);

  const hidden = rows.filter((row) => row.value === null);
  const asText = (): string =>
    serializeEnvFile(rows.flatMap((row) => (row.value === null ? [] : [{ key: row.key, value: row.value }])));

  const applyText = (text: string): number[] => {
    const parsed = parseEnvFile(text);
    const secretKeys = new Set(rows.filter((row) => row.secret).map((row) => row.key));
    const keptHidden = hidden.filter((row) => !parsed.variables.some((variable) => variable.key === row.key));
    const fromText = parsed.variables.map((variable) => toRow({ ...variable, secret: secretKeys.has(variable.key) }));
    setRows([...fromText, ...keptHidden]);
    return parsed.invalidLines;
  };

  const inputs = (): VariableInput[] =>
    rows.map((row) => ({ key: row.key.trim(), value: row.value, secret: row.secret }));

  return {
    rows,
    hiddenCount: hidden.length,
    dirty: fingerprint(rows) !== fingerprint(saved.map((variable) => ({ ...variable, value: variable.value ?? null }))),
    update,
    remove,
    add,
    asText,
    applyText,
    inputs,
  };
}
