import { useQuery } from "@tanstack/react-query";

import { BackupsPanel } from "@/shared/backups";
import { useTranslation } from "@/shared/i18n";

import type { Application } from "../api";
import { isCompose } from "../sourceForm";
import { runtimeQuery } from "./api";

export function VolumeBackups({ application }: { application: Application }) {
  const { t } = useTranslation("applications");
  const compose = isCompose(application);
  const runtime = useQuery({ ...runtimeQuery(application.id), enabled: !compose });
  const volumes = (runtime.data?.mounts ?? []).filter((mount) => mount.kind === "volume");

  return volumes.map((mount) => (
    <BackupsPanel
      key={mount.source}
      target={{ kind: "volume", applicationId: application.id, volume: mount.source }}
      title={t("runtime.volumeBackups.title", { volume: mount.source })}
      description={t("runtime.volumeBackups.description")}
    />
  ));
}
