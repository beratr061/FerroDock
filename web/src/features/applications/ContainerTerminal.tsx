import { TerminalWindowIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";

import { useTranslation } from "@/shared/i18n";
import { openTerminal } from "@/shared/terminal";
import { Button } from "@/shared/ui";

import { containersQuery } from "./api";

export function ContainerTerminal({ applicationId }: { applicationId: string }) {
  const { t } = useTranslation("applications");
  const containers = useQuery(containersQuery(applicationId));
  const running = containers.data?.find((container) => container.running);

  return (
    <Button
      variant="secondary"
      icon={<TerminalWindowIcon size={16} aria-hidden />}
      disabled={!running}
      title={running ? undefined : t("terminal.unavailable")}
      onClick={() => running && openTerminal(running.id, running.name)}
    >
      {t("terminal.open")}
    </Button>
  );
}
