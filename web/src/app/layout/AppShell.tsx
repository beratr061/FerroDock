import { Outlet } from "@tanstack/react-router";

import { TerminalHost } from "@/shared/terminal";
import { Logo } from "@/shared/ui";

import { PaletteHost } from "../palette/PaletteHost";
import { BottomNav } from "./BottomNav";
import { RealtimeBridge } from "./RealtimeBridge";
import { Sidebar } from "./Sidebar";
import { UserMenu } from "./UserMenu";

export function AppShell() {
  return (
    <div className="flex min-h-dvh">
      <RealtimeBridge />
      <Sidebar />
      <main className="min-w-0 flex-1 px-4 pt-4 pb-24 sm:px-8 md:pt-6 md:pb-10 lg:px-10">
        <header className="mb-6 flex items-center justify-between md:hidden">
          <Logo compact />
          <UserMenu />
        </header>
        <div className="mx-auto w-full max-w-6xl">
          <Outlet />
        </div>
      </main>
      <BottomNav />
      <TerminalHost />
      <PaletteHost />
    </div>
  );
}
