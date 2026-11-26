import type { QueryClient } from "@tanstack/react-query";
import {
  createRootRouteWithContext,
  createRoute,
  createRouter,
  lazyRouteComponent,
  Outlet,
  redirect,
} from "@tanstack/react-router";

import { meQuery, setupStatusQuery } from "@/features/auth";

import { AppShell } from "./layout/AppShell";

interface RouterContext {
  queryClient: QueryClient;
}

const rootRoute = createRootRouteWithContext<RouterContext>()({ component: Outlet });

const setupRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/setup",
  beforeLoad: async ({ context }) => {
    const status = await context.queryClient.ensureQueryData(setupStatusQuery);
    if (!status.required) {
      throw redirect({ to: "/sign-in" });
    }
  },
  component: lazyRouteComponent(() => import("@/features/auth/pages"), "SetupPage"),
});

const signInRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/sign-in",
  beforeLoad: async ({ context }) => {
    const status = await context.queryClient.ensureQueryData(setupStatusQuery);
    if (status.required) {
      throw redirect({ to: "/setup" });
    }
  },
  component: lazyRouteComponent(() => import("@/features/auth/pages"), "SignInPage"),
});

const appRoute = createRoute({
  getParentRoute: () => rootRoute,
  id: "app",
  beforeLoad: async ({ context }) => {
    const status = await context.queryClient.ensureQueryData(setupStatusQuery);
    if (status.required) {
      throw redirect({ to: "/setup" });
    }
    try {
      await context.queryClient.ensureQueryData(meQuery);
    } catch {
      throw redirect({ to: "/sign-in" });
    }
  },
  component: AppShell,
});

const indexRoute = createRoute({
  getParentRoute: () => appRoute,
  path: "/",
  beforeLoad: () => {
    throw redirect({ to: "/projects" });
  },
});

const projectsRoute = createRoute({
  getParentRoute: () => appRoute,
  path: "/projects",
  component: lazyRouteComponent(() => import("@/features/projects"), "ProjectsPage"),
});

const projectDetailRoute = createRoute({
  getParentRoute: () => appRoute,
  path: "/projects/$projectId",
  component: lazyRouteComponent(() => import("./pages/ProjectWorkspace"), "ProjectWorkspace"),
});

const applicationRoute = createRoute({
  getParentRoute: () => appRoute,
  path: "/applications/$applicationId",
  component: lazyRouteComponent(() => import("@/features/applications"), "ApplicationPage"),
});

const databaseRoute = createRoute({
  getParentRoute: () => appRoute,
  path: "/databases/$databaseId",
  component: lazyRouteComponent(() => import("@/features/databases"), "DatabasePage"),
});

const serversRoute = createRoute({
  getParentRoute: () => appRoute,
  path: "/servers",
  component: lazyRouteComponent(() => import("@/features/servers"), "ServersPage"),
});

const settingsRoute = createRoute({
  getParentRoute: () => appRoute,
  path: "/settings",
  component: lazyRouteComponent(() => import("@/features/settings"), "SettingsLayout"),
});

const settingsIndexRoute = createRoute({
  getParentRoute: () => settingsRoute,
  path: "/",
  beforeLoad: () => {
    throw redirect({ to: "/settings/api-keys" });
  },
});

const apiKeysRoute = createRoute({
  getParentRoute: () => settingsRoute,
  path: "/api-keys",
  component: lazyRouteComponent(() => import("@/features/settings"), "ApiKeysPage"),
});

const appearanceRoute = createRoute({
  getParentRoute: () => settingsRoute,
  path: "/appearance",
  component: lazyRouteComponent(() => import("@/features/settings"), "AppearancePage"),
});

const backupsRoute = createRoute({
  getParentRoute: () => settingsRoute,
  path: "/backups",
  component: lazyRouteComponent(() => import("@/features/settings"), "BackupsPage"),
});

const storageRoute = createRoute({
  getParentRoute: () => settingsRoute,
  path: "/storage",
  component: lazyRouteComponent(() => import("@/features/settings"), "StoragePage"),
});

const updatesRoute = createRoute({
  getParentRoute: () => settingsRoute,
  path: "/updates",
  component: lazyRouteComponent(() => import("@/features/settings"), "UpdatesPage"),
});

const routeTree = rootRoute.addChildren([
  setupRoute,
  signInRoute,
  appRoute.addChildren([
    indexRoute,
    projectsRoute,
    projectDetailRoute,
    applicationRoute,
    databaseRoute,
    serversRoute,
    settingsRoute.addChildren([settingsIndexRoute, apiKeysRoute, appearanceRoute, backupsRoute, storageRoute, updatesRoute]),
  ]),
]);

export function buildRouter(queryClient: QueryClient) {
  return createRouter({
    routeTree,
    context: { queryClient },
    defaultPreload: "intent",
    defaultViewTransition: true,
    scrollRestoration: true,
  });
}

declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof buildRouter>;
  }
}
