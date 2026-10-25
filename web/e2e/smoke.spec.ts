import { expect, test } from "@playwright/test";
import { BRAND } from "../src/brand";

test("loads home", async ({ page }) => {
  const response = await page.goto("/");

  expect(response?.status()).toBe(200);
  await expect(
    page.getByRole("heading", { level: 1, name: BRAND.name, exact: true }),
  ).toBeVisible();
});

test("health ok", async ({ request }) => {
  const response = await request.get("/healthz");

  expect(response.status()).toBe(200);
});

test("spa fallback", async ({ page }) => {
  const response = await page.goto("/does/not/exist");

  expect(response?.status()).toBe(200);
  await expect(page).toHaveTitle(BRAND.name);
});
