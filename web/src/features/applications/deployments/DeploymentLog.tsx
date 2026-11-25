import { useLayoutEffect, useRef, useState } from "react";
import { ArrowDownIcon } from "@phosphor-icons/react";

import { useTranslation } from "@/shared/i18n";
import { useDeploymentLog } from "./useDeploymentLog";

const STICK_THRESHOLD_PX = 24;

export function DeploymentLog({ deploymentId }: { deploymentId: string }) {
  const { t } = useTranslation("applications");
  const log = useDeploymentLog(deploymentId);
  const viewport = useRef<HTMLPreElement>(null);
  const [following, setFollowing] = useState(true);

  useLayoutEffect(() => {
    const element = viewport.current;
    if (element && following) {
      element.scrollTop = element.scrollHeight;
    }
  }, [log.text, following]);

  const onScroll = (): void => {
    const element = viewport.current;
    if (element) {
      setFollowing(element.scrollHeight - element.scrollTop - element.clientHeight < STICK_THRESHOLD_PX);
    }
  };

  return (
    <div className="relative">
      <pre
        ref={viewport}
        onScroll={onScroll}
        aria-live="off"
        className="h-80 overflow-auto rounded-control border border-border bg-bg p-3 font-mono text-[12px] leading-relaxed text-muted"
      >
        {log.text || (log.failed ? t("deployments.logUnavailable") : t("deployments.logWaiting"))}
      </pre>
      {following ? null : (
        <button
          type="button"
          onClick={() => setFollowing(true)}
          className="press enter-settle absolute right-3 bottom-3 inline-flex items-center gap-1.5 rounded-control border border-border-strong bg-surface px-2.5 py-1 text-[12px] text-text"
        >
          <ArrowDownIcon size={14} aria-hidden />
          {t("deployments.follow")}
        </button>
      )}
    </div>
  );
}
