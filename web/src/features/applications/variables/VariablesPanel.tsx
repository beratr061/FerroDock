import { useState } from "react";
import { PlusIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { Button, InlineAlert, SegmentedControl } from "@/shared/ui";

import { useSaveVariables, variablesQuery, type Variable } from "./api";
import { useVariableDraft } from "./useVariableDraft";
import { VariableRow } from "./VariableRow";

type Mode = "list" | "file";

function Editor({ applicationId, saved }: { applicationId: string; saved: Variable[] }) {
  const { t } = useTranslation("applications");
  const draft = useVariableDraft(saved);
  const save = useSaveVariables(applicationId);
  const [mode, setMode] = useState<Mode>("list");
  const [text, setText] = useState("");
  const [invalid, setInvalid] = useState<number[]>([]);

  const switchMode = (next: Mode): void => {
    if (next === "file") {
      setText(draft.asText());
    } else {
      setInvalid(draft.applyText(text));
    }
    setMode(next);
  };

  const onSave = (): void => {
    const pending = mode === "file" ? draft.applyText(text) : [];
    setInvalid(pending);
    if (mode === "file") {
      setMode("list");
      return;
    }
    save.mutate(draft.inputs());
  };

  return (
    <div className="flex flex-col gap-3">
      <SegmentedControl<Mode>
        label={t("variables.mode")}
        value={mode}
        segments={[
          { value: "list", label: t("variables.modeList") },
          { value: "file", label: t("variables.modeFile") },
        ]}
        onChange={switchMode}
      />
      {invalid.length > 0 ? <InlineAlert tone="warning">{t("variables.invalidLines", { lines: invalid.join(", ") })}</InlineAlert> : null}
      {save.isError ? <InlineAlert tone="danger">{errorMessage(save.error, t)}</InlineAlert> : null}
      {mode === "file" ? (
        <div className="flex flex-col gap-2">
          <textarea
            aria-label={t("variables.modeFile")}
            value={text}
            onChange={(event) => setText(event.target.value)}
            spellCheck={false}
            rows={12}
            className="rounded-control border border-border-strong bg-surface p-3 font-mono text-[13px] text-text hover:border-text focus:border-text"
          />
          {draft.hiddenCount > 0 ? <p className="text-[13px] text-subtle">{t("variables.hiddenKept", { count: draft.hiddenCount })}</p> : null}
        </div>
      ) : (
        <div className="overflow-hidden rounded-panel border border-border bg-surface">
          {draft.rows.length === 0 ? <p className="px-4 py-6 text-muted">{t("variables.empty")}</p> : null}
          <ul>
            {draft.rows.map((row) => (
              <VariableRow
                key={row.id}
                applicationId={applicationId}
                row={row}
                onChange={(change) => draft.update(row.id, change)}
                onRemove={() => draft.remove(row.id)}
              />
            ))}
          </ul>
        </div>
      )}
      <div className="flex flex-wrap items-center justify-between gap-2">
        {mode === "list" ? (
          <Button variant="ghost" icon={<PlusIcon size={16} weight="bold" aria-hidden />} onClick={draft.add}>
            {t("variables.add")}
          </Button>
        ) : (
          <span />
        )}
        <Button busy={save.isPending} disabled={mode === "list" && !draft.dirty} onClick={onSave}>
          {mode === "file" ? t("variables.applyFile") : t("variables.save")}
        </Button>
      </div>
    </div>
  );
}

export function VariablesPanel({ applicationId }: { applicationId: string }) {
  const { t } = useTranslation("applications");
  const variables = useQuery(variablesQuery(applicationId));

  return (
    <section className="flex flex-col gap-3">
      <div className="flex flex-col gap-1">
        <h2 className="text-lg text-text">{t("variables.title")}</h2>
        <p className="max-w-2xl text-muted">{t("variables.description")}</p>
      </div>
      {variables.isError ? <InlineAlert tone="danger">{errorMessage(variables.error, t)}</InlineAlert> : null}
      {variables.isPending ? <div aria-hidden className="skeleton h-24" /> : null}
      {variables.data ? <Editor applicationId={applicationId} saved={variables.data} /> : null}
    </section>
  );
}
