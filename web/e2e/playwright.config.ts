import { resolve } from "node:path";
import { defineConfig, devices } from "@playwright/test";

const configDirectory = resolve(__dirname);

const playwrightConfig = defineConfig({
  testDir: configDirectory,
  outputDir: resolve(configDirectory, "../test-results"),
  reporter: [
    ["html", { outputFolder: resolve(configDirectory, "../playwright-report"), open: "never" }],
  ],
  use: {
    baseURL: process.env.E2E_BASE_URL ?? "http://localhost:8080",
    trace: "retain-on-failure",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
    {
      name: "webkit-mobile",
      use: { ...devices["iPhone 13"] },
    },
  ],
});

module.exports = playwrightConfig;
