import { useLayoutEffect, useRef, useState } from "react";
import { ArrowDownIcon } from "@phosphor-icons/react";

import { useTranslation } from "@/shared/i18n";
import { cx } from "@/shared/lib/cx";

import { useContainerLogs } from "./useContainerLogs";

const STICK_THRESHOLD_PX = 24;

export function LogViewer({ applicationId, container }: { applicationId: string; container: string }) {
  const { t } = useTranslation("applications");
  const { lines, connection } = useContainerLogs(applicationId, container);
  const viewport = useRef<HTMLPreElement>(null);
  const [following, setFollowing] = useState(true);

  useLayoutEffect(() => {
    const element = viewport.current;
    if (element && following) {
      element.scrollTop = element.scrollHeight;
    }
  }, [lines, following]);

  const onScroll = (): void => {
    const element = viewport.current;
    if (element) {
      setFollowing(element.scrollHeight - element.scrollTop - element.clientHeight < STICK_THRESHOLD_PX);
    }
  };

  const placeholder = connection === "closed" ? t("logs.closed") : t("logs.waiting");

  return (
    <div className="relative">
      <pre
        ref={viewport}
        onScroll={onScroll}
        aria-live="off"
        className="h-96 overflow-auto rounded-control border border-border bg-bg p-3 font-mono text-[12px] leading-relaxed whitespace-pre-wrap text-muted"
      >
        {lines.map((line) => (
          <span key={line.id} className={cx(line.source === "stderr" && "text-danger")}>
            {line.text}
          </span>
        ))}
        {lines.length === 0 ? placeholder : null}
      </pre>
      {connection === "closed" && lines.length > 0 ? <p className="mt-2 text-[12px] text-subtle">{t("logs.closed")}</p> : null}
      {following ? null : (
        <button
          type="button"
          onClick={() => setFollowing(true)}
          className="press enter-settle absolute right-3 bottom-3 inline-flex items-center gap-1.5 rounded-control border border-border-strong bg-surface px-2.5 py-1 text-[12px] text-text"
        >
          <ArrowDownIcon size={14} aria-hidden />
          {t("logs.follow")}
        </button>
      )}
    </div>
  );
}
