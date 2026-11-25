import { SignOutIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";

import { meQuery, useSignOut } from "@/features/auth";

function initials(name: string): string {
  return name
    .split(/\s+/)
    .slice(0, 2)
    .map((part) => part.charAt(0).toLocaleUpperCase())
    .join("");
}

export function UserMenu() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const me = useQuery(meQuery);
  const signOut = useSignOut();
  const organization = me.data?.memberships.find(
    (membership) => membership.organization_id === me.data?.active_organization_id,
  );

  const onSignOut = (): void => {
    signOut.mutate(undefined, { onSettled: () => void navigate({ to: "/sign-in" }) });
  };

  return (
    <div className="flex items-center gap-2.5">
      <span className="grid size-9 shrink-0 place-items-center rounded-full border border-border bg-surface-raised text-[12px] font-semibold text-text">
        {me.data ? initials(me.data.name) : ""}
      </span>
      <div className="hidden min-w-0 flex-1 lg:block">
        <p className="truncate font-medium text-text">{me.data?.name}</p>
        <p className="truncate text-[12px] text-subtle">{organization?.organization_name}</p>
      </div>
      <button
        type="button"
        onClick={onSignOut}
        title={t("actions.signOut")}
        className="press grid size-9 shrink-0 place-items-center rounded-control text-muted hover:bg-surface-hover hover:text-text md:hidden lg:grid"
      >
        <SignOutIcon aria-hidden size={18} />
        <span className="sr-only">{t("actions.signOut")}</span>
      </button>
    </div>
  );
}
