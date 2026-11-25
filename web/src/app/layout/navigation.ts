import { CubeIcon, GearSixIcon, HardDrivesIcon } from "@phosphor-icons/react";

export const NAV_ITEMS = [
  { to: "/projects", key: "projects", Icon: CubeIcon },
  { to: "/servers", key: "servers", Icon: HardDrivesIcon },
  { to: "/settings", key: "settings", Icon: GearSixIcon },
] as const;
