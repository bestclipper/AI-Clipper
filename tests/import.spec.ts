import { test, expect } from "@playwright/test";
import path from "node:path";
test("imports, plays, replaces and removes a local video", async ({ page }) => {
  await page.goto("/");
  await expect(
    page.getByRole("button", { name: "5 Clips erstellen" }),
  ).toBeDisabled();
  await page
    .locator("input[type=file]")
    .setInputFiles(path.resolve("tests/fixtures/sample.mp4"));
  await expect(page.locator(".source-bar strong")).toHaveText("sample.mp4");
  await expect(page.locator(".source-bar small")).toContainText("320 × 180");
  const video = page.locator("video");
  await video.evaluate(async (v: HTMLVideoElement) => {
    v.muted = true;
    await v.play();
  });
  await expect
    .poll(() => video.evaluate((v: HTMLVideoElement) => v.currentTime))
    .toBeGreaterThan(0);
  await page
    .locator("input[type=file]")
    .setInputFiles(path.resolve("tests/fixtures/sample.mp4"));
  await expect(page.locator(".source-bar small")).toContainText("320 × 180");
  await page.getByRole("button", { name: "Video entfernen" }).click();
  await expect(
    page.getByRole("button", { name: "Video auswählen" }),
  ).toBeVisible();
  await expect(page.locator("video")).toHaveCount(0);
});
test("explains invalid and unplayable files", async ({ page }) => {
  await page.goto("/");
  await page.locator("input[type=file]").setInputFiles({
    name: "notes.txt",
    mimeType: "text/plain",
    buffer: Buffer.from("hello"),
  });
  await expect(page.getByRole("alert")).toContainText(
    "unterstützte Videodatei",
  );
  await page.locator("input[type=file]").setInputFiles({
    name: "broken.mp4",
    mimeType: "video/mp4",
    buffer: Buffer.from("not a video"),
  });
  await expect(page.getByRole("alert")).toContainText("nicht wiedergegeben");
});
