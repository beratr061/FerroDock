import { defineConfig, devices } from "@playwright/test";

const ownerState = "e2e/.auth/owner.json";
const ci = Boolean(process.env["CI"]);

export default defineConfig({
  testDir: "./e2e",
  testMatch: "**/*.e2e.ts",
  fullyParallel: false,
  workers: 1,
  timeout: 240_000,
  expect: { timeout: 15_000 },
  retries: ci ? 1 : 0,
  reporter: ci ? [["github"], ["list"]] : "list",
  use: {
    baseURL: process.env["FD_E2E_URL"] ?? "http://127.0.0.1:3000",
    locale: "en-US",
    trace: "retain-on-failure",
  },
  projects: [
    {
      name: "flow",
      testMatch: "**/flow.e2e.ts",
      use: { ...devices["Desktop Chrome"], viewport: { width: 1440, height: 900 } },
    },
    {
      name: "desktop",
      testMatch: "**/layout.e2e.ts",
      dependencies: ["flow"],
      use: { ...devices["Desktop Chrome"], viewport: { width: 1440, height: 900 }, storageState: ownerState },
    },
    {
      name: "mobile",
      testMatch: "**/layout.e2e.ts",
      dependencies: ["flow"],
      use: { ...devices["Pixel 7"], viewport: { width: 375, height: 812 }, storageState: ownerState },
    },
  ],
});

