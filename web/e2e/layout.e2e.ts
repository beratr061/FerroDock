import { expect, test, type Page } from "@playwright/test";

const APPLICATION_TABS = ["deployments", "logs", "environment", "domains", "storage", "settings"];

interface Named {
  id: string;
}

async function ids(page: Page, path: string): Promise<string[]> {
  const response = await page.request.get(path);
  expect(response.ok()).toBeTruthy();
  return ((await response.json()) as Named[]).map((item) => item.id);
}

async function populatedProject(page: Page): Promise<{ project: string; application: string; database: string }> {
  for (const project of await ids(page, "/api/v1/projects")) {
    const [application] = await ids(page, `/api/v1/projects/${project}/applications`);
    const [database] = await ids(page, `/api/v1/projects/${project}/databases`);
    if (application && database) {
      return { project, application, database };
    }
  }
  throw new Error("no project with an application and a database; run the flow tests first");
}

async function expectNoSidewaysScroll(page: Page, path: string): Promise<void> {
  await page.goto(path);
  await page.waitForLoadState("networkidle");
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  expect(overflow, `${path} is wider than the screen`).toBeLessThanOrEqual(0);
}

test("pages fit the screen", async ({ page }) => {
  const { project: projectId, application: applicationId, database: databaseId } = await populatedProject(page);
  const paths = [
    "/projects",
    `/projects/${projectId}`,
    `/databases/${databaseId}`,
    "/servers",
    "/settings/storage",
    ...APPLICATION_TABS.map((tab) => `/applications/${applicationId}?tab=${tab}`),
  ];
  for (const path of paths) {
    await expectNoSidewaysScroll(page, path);
  }
});
