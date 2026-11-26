import { expect, test, type Page } from "@playwright/test";

const OWNER = {
  name: "E2E Owner",
  email: "owner@e2e.test",
  organization: "E2E",
  password: "Correct-Horse-9-battery",
};
const DEPLOY_TIMEOUT = 180_000;
const RUN = Date.now().toString(36);
const PROJECT = `Storefront ${RUN}`;

test.describe.configure({ mode: "serial" });

function formWith(page: Page, submit: string) {
  return page.locator("form", { has: page.getByRole("button", { name: submit }) });
}

async function signIn(page: Page): Promise<void> {
  await page.goto("/sign-in");
  await page.getByLabel("Email").fill(OWNER.email);
  await page.getByLabel("Password").fill(OWNER.password);
  await page.getByRole("button", { name: "Sign in" }).click();
  await expect(page).toHaveURL(/\/projects$/);
}

test("the owner finishes setup", async ({ page }) => {
  const token = process.env["FD_SETUP_TOKEN"];
  test.skip(!token, "FD_SETUP_TOKEN is not set");
  await page.goto("/setup");
  await page.getByLabel("Full name").fill(OWNER.name);
  await page.getByLabel("Email").fill(OWNER.email);
  await page.getByLabel("Organization").fill(OWNER.organization);
  await page.getByLabel("Password").fill(OWNER.password);
  await page.getByLabel("Setup token").fill(token ?? "");
  await page.getByRole("button", { name: "Create account" }).click();
  await expect(page).toHaveURL(/\/projects$/);
});

test("an image application deploys, logs and opens a shell", async ({ page }) => {
  await signIn(page);
  await page.getByRole("button", { name: "New project" }).first().click();
  await page.getByLabel("Name").fill(PROJECT);
  await page.getByRole("button", { name: "Create project" }).click();
  await expect(page.getByRole("heading", { level: 1, name: PROJECT })).toBeVisible();

  await page.getByRole("button", { name: "New application" }).click();
  const application = formWith(page, "Create application");
  await application.getByLabel("Name", { exact: true }).fill("web");
  await application.getByText("Docker image", { exact: true }).click();
  await expect(application.getByLabel("Docker image")).toBeChecked();
  await application.getByLabel("Image", { exact: true }).fill("nginx:1.29-alpine");
  await application.getByRole("button", { name: "Create application" }).click();
  await expect(page.getByRole("heading", { level: 1, name: "web" })).toBeVisible();

  await page.getByRole("button", { name: "Deploy", exact: true }).click();
  await expect(page.getByText("Deployment finished")).toBeVisible({ timeout: DEPLOY_TIMEOUT });

  await page.getByRole("tab", { name: "Logs" }).click();
  await expect(page.locator("pre").first()).toContainText("nginx", { timeout: 30_000 });

  await page.getByRole("button", { name: "Terminal" }).click();
  await expect(page.getByText("Connected", { exact: true })).toBeVisible();
  await page.locator(".xterm").first().click();
  await page.keyboard.type("echo shell-$((2+3)) > /proc/1/fd/1");
  await page.keyboard.press("Enter");
  await expect(page.locator("pre").first()).toContainText("shell-5");
});

test("a managed database starts", async ({ page }) => {
  await signIn(page);
  await page.getByRole("link", { name: new RegExp(PROJECT) }).first().click();
  await page.getByRole("button", { name: "New database" }).click();
  const database = formWith(page, "Create database");
  await database.getByLabel("Name", { exact: true }).fill("cache");
  await database.getByRole("combobox", { name: "Engine" }).click();
  await database.getByRole("option", { name: "Redis", exact: true }).click();
  await database.getByRole("button", { name: "Create database" }).click();
  await expect(page.getByRole("heading", { level: 1, name: "cache" })).toBeVisible();
  await expect(page.getByText("Running", { exact: true }).first()).toBeVisible({ timeout: DEPLOY_TIMEOUT });
  await page.context().storageState({ path: "e2e/.auth/owner.json" });
});
