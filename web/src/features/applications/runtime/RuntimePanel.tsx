import { HardDrivesIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";

import { useTranslation } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/errors";
import { InlineAlert } from "@/shared/ui";

import type { Application } from "../api";
import { isCompose } from "../sourceForm";
import { runtimeQuery } from "./api";
import { RuntimeEditor } from "./RuntimeEditor";

export function RuntimePanel({ application }: { application: Application }) {
  const { t } = useTranslation("applications");
  const compose = isCompose(application);
  const runtime = useQuery({ ...runtimeQuery(application.id), enabled: !compose });

  return (
    <section className="flex flex-col gap-3">
      <header className="flex flex-col gap-1">
        <h2 className="flex items-center gap-2 text-lg text-text">
          <HardDrivesIcon size={18} aria-hidden />
          {t("runtime.title")}
        </h2>
        <p className="max-w-2xl text-muted">{t(compose ? "runtime.composeNote" : "runtime.description")}</p>
      </header>
      {runtime.isError ? <InlineAlert tone="danger">{errorMessage(runtime.error, t)}</InlineAlert> : null}
      {runtime.data ? (
        <RuntimeEditor
          key={runtime.dataUpdatedAt}
          applicationId={application.id}
          saved={runtime.data}
          blueGreen={application.strategy === "blue_green"}
        />
      ) : null}
    </section>
  );
}
