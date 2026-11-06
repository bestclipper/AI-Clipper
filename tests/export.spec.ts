import { test, expect } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  // UI contract test only: native encoding is exercised by the Rust FFmpeg tests.
  await page.addInitScript(() => {
    const w = window as unknown as Record<string, unknown>;
    w.isTauri = true;
    let mode = "success";
    let cancelled = false;
    let release: (() => void) | undefined;
    w.__setExportMode = (value: string) => {
      mode = value;
    };
    w.__releaseExport = () => release?.();
    w.__TAURI_INTERNALS__ = {
      convertFileSrc: () => "/tests/fixtures/sample.mp4",
      transformCallback: () => 1,
      unregisterCallback: () => {},
      invoke: async (
        command: string,
        args: { onProgress?: { onmessage: (data: unknown) => void } },
      ) => {
        if (command === "plugin:dialog|open") return "/Users/test/source.mp4";
        if (command === "probe_source")
          return { duration: 120, width: 320, height: 180, hasAudio: true };
        if (command === "cancel_export") {
          cancelled = true;
          release?.();
          return;
        }
        if (command === "export_clip") {
          cancelled = false;
          if (mode === "folder-cancel") return null;
          if (mode === "error") throw "Exportprüfung fehlgeschlagen";
          args.onProgress?.onmessage({ phase: "encoding", percent: 42 });
          await new Promise<void>((resolve) => {
            release = resolve;
          });
          if (cancelled) throw "Export abgebrochen.";
          args.onProgress?.onmessage({ phase: "verifying", percent: 90 });
          return {
            outputPath: "/Users/test/Clips/clip.mp4",
            duration: 61,
            width: 1080,
            height: 1920,
            hasAudio: true,
          };
        }
        throw new Error(`Unexpected native command: ${command}`);
      },
    };
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Video auswählen" }).click();
});
test("validates a range, shows progress, locks the source and reports verified output", async ({
  page,
}) => {
  const exportButton = page.getByRole("button", {
    name: "Abschnitt als MP4 exportieren",
  });
  await expect(exportButton).toBeEnabled();
  await page.getByLabel("Länge (Sekunden)").fill("60");
  await expect(exportButton).toBeDisabled();
  await page.getByLabel("Länge (Sekunden)").fill("61");
  await exportButton.click();
  await expect(
    page.getByRole("progressbar", { name: "Exportfortschritt" }),
  ).toHaveAttribute("value", "42");
  await expect(
    page.getByRole("button", { name: "Video entfernen" }),
  ).toBeDisabled();
  await expect(page.getByLabel("Start (Sekunden)")).toBeDisabled();
  await page.evaluate(() =>
    (window as unknown as { __releaseExport: () => void }).__releaseExport(),
  );
  await expect(page.getByText("MP4 exportiert und geprüft")).toBeVisible();
  await expect(page.getByText("/Users/test/Clips/clip.mp4")).toBeVisible();
  await expect(exportButton).toBeEnabled();
});
test("restores controls after folder cancellation and native errors", async ({
  page,
}) => {
  const exportButton = page.getByRole("button", {
    name: "Abschnitt als MP4 exportieren",
  });
  await page.evaluate(() =>
    (
      window as unknown as { __setExportMode: (mode: string) => void }
    ).__setExportMode("folder-cancel"),
  );
  await exportButton.click();
  await expect(exportButton).toBeEnabled();
  await expect(page.getByText("MP4 exportiert und geprüft")).toHaveCount(0);
  await page.evaluate(() =>
    (
      window as unknown as { __setExportMode: (mode: string) => void }
    ).__setExportMode("error"),
  );
  await exportButton.click();
  await expect(page.getByRole("alert")).toContainText(
    "Exportprüfung fehlgeschlagen",
  );
  await expect(exportButton).toBeEnabled();
});

test("cancels an active export and restores controls", async ({ page }) => {
  const button = page.getByRole("button", {
    name: "Abschnitt als MP4 exportieren",
  });
  await button.click();
  await page.getByRole("button", { name: "Abbrechen", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("Export abgebrochen");
  await expect(button).toBeEnabled();
});
