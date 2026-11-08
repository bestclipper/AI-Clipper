import { test, expect } from "@playwright/test";
test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    const w = window as unknown as Record<string, unknown>;
    w.isTauri = true;
    let ready = false,
      cancelled = false;
    let release: (() => void) | undefined;
    let saved: unknown = null;
    const model = () => ({
      id: "Qwen3-1.7B-Q4_K_M",
      ready,
      bytes: 1282439264,
      sha256: "verified",
    });
    w.__releaseAnalysis = () => release?.();
    w.__TAURI_INTERNALS__ = {
      convertFileSrc: () => "/tests/fixtures/sample.mp4",
      transformCallback: () => 1,
      unregisterCallback: () => {},
      invoke: async (
        command: string,
        args: { onProgress?: { onmessage: (data: unknown) => void } },
      ) => {
        if (command === "plugin:dialog|open") return "/Users/test/video.mp4";
        if (command === "probe_source")
          return { duration: 120, width: 320, height: 180, hasAudio: true };
        if (command === "asr_status")
          return {
            model: { ...model(), id: "base.en", ready: true },
            componentReady: true,
            componentError: null,
          };
        if (command === "analysis_status")
          return { model: model(), componentReady: true, componentError: null };
        if (command === "load_transcript") return null;
        if (command === "load_analysis") return saved;
        if (command === "cancel_operation") {
          cancelled = true;
          release?.();
          return;
        }
        if (command === "download_qwen_model" || command === "analyze_video") {
          cancelled = false;
          args.onProgress?.onmessage({
            phase: command === "download_qwen_model" ? "downloading" : "events",
            percent: 42,
            message: "Testfortschritt",
          });
          await new Promise<void>((resolve) => {
            release = resolve;
          });
          if (cancelled)
            throw "Analyse abgebrochen. Zwischenergebnisse sind gespeichert.";
          if (command === "download_qwen_model") {
            ready = true;
            return model();
          }
          // Short preview fixture only; production candidates are separately validated at 61–90s in Rust.
          saved = {
            complete: true,
            model: model().id,
            nextWindow: 3,
            totalWindows: 3,
            evaluated: 2,
            totalPlans: 2,
            cpuFallbackUsed: false,
            candidates: [
              {
                id: "preview",
                title: "A linked story",
                mode: "smart_story",
                segments: [
                  {
                    firstUnit: 0,
                    lastUnit: 0,
                    start: 0.2,
                    end: 0.7,
                    duration: 0.5,
                  },
                  {
                    firstUnit: 1,
                    lastUnit: 1,
                    start: 1.4,
                    end: 1.9,
                    duration: 0.5,
                  },
                ],
                totalDuration: 1,
                score: 8.2,
                category: "Test fixture",
                reason: "An actual source playlist with two distinct parts.",
                relation: "setup_payoff",
                connectionScore: 9,
                ratings: {
                  entertainment: 8,
                  surprise: 8,
                  hook: 9,
                  clarity: 8,
                  tension: 7,
                  ending: 9,
                },
                evidence: ["First part", "Second part"],
                timestampsApproximate: true,
              },
            ],
          };
          return saved;
        }
        throw new Error(`Unexpected command: ${command}`);
      },
    };
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Video auswählen" }).click();
});
async function download(page: import("@playwright/test").Page) {
  await page
    .getByRole("button", { name: /Qwen kostenlos herunterladen/ })
    .click();
  await expect(
    page.getByRole("progressbar", { name: "KI-Analysefortschritt" }),
  ).toHaveAttribute("value", "42");
  await page.evaluate(() =>
    (
      window as unknown as { __releaseAnalysis: () => void }
    ).__releaseAnalysis(),
  );
  await expect(
    page.getByRole("button", { name: "KI-Analyse starten", exact: true }),
  ).toBeEnabled();
}
test("model download, serial job lock, cancel and retry", async ({ page }) => {
  await download(page);
  await page
    .getByRole("button", { name: "KI-Analyse starten", exact: true })
    .click();
  await expect(
    page.getByRole("button", { name: "Video entfernen" }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Transkription starten", exact: true }),
  ).toBeDisabled();
  await page
    .getByRole("button", { name: "KI-Analyse / Download abbrechen" })
    .click();
  await expect(page.getByRole("alert")).toContainText("abgebrochen");
  await expect(
    page.getByRole("button", { name: "KI-Analyse starten", exact: true }),
  ).toBeEnabled();
});
test("restores candidates and plays all source parts skipping the gap", async ({
  page,
}) => {
  await download(page);
  await page
    .getByRole("button", { name: "KI-Analyse starten", exact: true })
    .click();
  await page.evaluate(() =>
    (
      window as unknown as { __releaseAnalysis: () => void }
    ).__releaseAnalysis(),
  );
  await expect(
    page.getByRole("heading", { name: "A linked story" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Ändern", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "A linked story" }),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "Smart Story als Playlist ansehen" })
    .click();
  await expect
    .poll(() =>
      page.locator("video").evaluate((v: HTMLVideoElement) => v.currentTime),
    )
    .toBeGreaterThanOrEqual(1.4);
  await expect
    .poll(() =>
      page.locator("video").evaluate((v: HTMLVideoElement) => v.paused),
    )
    .toBe(true);
  await expect(
    page.getByRole("button", { name: "Vorschau beenden" }),
  ).toHaveCount(0);
});
