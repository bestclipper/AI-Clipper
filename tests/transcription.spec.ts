import { test, expect } from "@playwright/test";
test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    const w = window as unknown as Record<string, unknown>;
    w.isTauri = true;
    let ready = false;
    let cancelled = false;
    let fail = false;
    let release: (() => void) | undefined;
    let saved: unknown = null;
    const model = () => ({
      id: "base.en",
      ready,
      bytes: 147964211,
      sha256: "verified",
    });
    w.__manySegments = () => {
      saved = {
        schemaVersion: 1,
        model: "base.en",
        language: "en",
        duration: 120,
        wordsApproximate: true,
        alignmentStatus: "unreviewed",
        cpuFallbackUsed: false,
        segments: Array.from({ length: 101 }, (_, i) => ({
          start: i,
          end: i + 0.5,
          text: `Segment ${i}`,
          words: [],
        })),
      };
    };
    w.__releaseSpeech = () => release?.();
    w.__speechFailure = () => {
      fail = true;
    };
    w.__TAURI_INTERNALS__ = {
      convertFileSrc: () => "/tests/fixtures/sample.mp4",
      transformCallback: () => 1,
      unregisterCallback: () => {},
      invoke: async (
        command: string,
        args: { onProgress?: { onmessage: (value: unknown) => void } },
      ) => {
        if (command === "plugin:dialog|open") return "/Users/test/video.mp4";
        if (command === "probe_source")
          return { duration: 120, width: 320, height: 180, hasAudio: true };
        if (command === "asr_status")
          return { model: model(), componentReady: true, componentError: null };
        if (command === "load_analysis") return null;
        if (command === "analysis_status")
          return {
            model: {
              id: "Qwen3-1.7B-Q4_K_M",
              ready: false,
              bytes: 1282439264,
              sha256: "test",
            },
            componentReady: true,
            componentError: null,
          };
        if (command === "load_transcript") return saved;
        if (command === "cancel_operation") {
          cancelled = true;
          release?.();
          return;
        }
        if (
          command === "download_whisper_model" ||
          command === "transcribe_video"
        ) {
          cancelled = false;
          if (fail) throw "Die Audiodatei konnte nicht verarbeitet werden.";
          args.onProgress?.onmessage(
            command === "download_whisper_model"
              ? {
                  phase: "downloading",
                  percent: 55,
                  receivedBytes: 80000000,
                  totalBytes: 147964211,
                }
              : {
                  phase: "transcribing",
                  percent: 40,
                  chunk: 1,
                  totalChunks: 2,
                },
          );
          await new Promise<void>((resolve) => {
            release = resolve;
          });
          if (cancelled) throw "Vorgang abgebrochen.";
          if (command === "download_whisper_model") {
            ready = true;
            return model();
          }
          saved = {
            schemaVersion: 1,
            model: "base.en",
            language: "en",
            duration: 120,
            wordsApproximate: true,
            alignmentStatus: "unreviewed",
            cpuFallbackUsed: false,
            segments: [
              {
                start: 1,
                end: 3,
                text: "Hello country",
                words: [
                  { text: "Hello", start: 1, end: 1.7, approximate: true },
                ],
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
async function installModel(page: import("@playwright/test").Page) {
  await page
    .getByRole("button", { name: "base.en kostenlos herunterladen" })
    .click();
  await expect(
    page.getByRole("progressbar", { name: "Transkriptionsfortschritt" }),
  ).toHaveAttribute("value", "55");
  await page.evaluate(() =>
    (window as unknown as { __releaseSpeech: () => void }).__releaseSpeech(),
  );
  await expect(
    page.getByText("Modell lokal vorhanden und geprüft"),
  ).toBeVisible();
}
test("downloads, transcribes, seeks and restores the project", async ({
  page,
}) => {
  await expect(
    page.getByRole("button", { name: "Transkription starten" }),
  ).toBeDisabled();
  await installModel(page);
  await page.getByRole("button", { name: "Transkription starten" }).click();
  await expect(
    page.getByRole("button", { name: "Video entfernen" }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Abschnitt als MP4 exportieren" }),
  ).toBeDisabled();
  await expect(
    page.getByRole("progressbar", { name: "Transkriptionsfortschritt" }),
  ).toHaveAttribute("value", "40");
  await page.evaluate(() =>
    (window as unknown as { __releaseSpeech: () => void }).__releaseSpeech(),
  );
  await expect(page.getByText("Transkript lokal gespeichert")).toBeVisible();
  await page.getByRole("button", { name: "Zu 00:00:01.000 springen" }).click();
  await expect
    .poll(() =>
      page.locator("video").evaluate((v: HTMLVideoElement) => v.currentTime),
    )
    .toBeCloseTo(1, 1);
  await page.getByText("Ungefähre Wortzeiten").click();
  await expect(page.getByText("≈ 00:00:01.000")).toBeVisible();
  await page.getByRole("button", { name: "Ändern", exact: true }).click();
  await expect(
    page.getByText("Gespeichertes Transkript wiederhergestellt"),
  ).toBeVisible();
});
test("cancels a model download and allows retry", async ({ page }) => {
  await page
    .getByRole("button", { name: "base.en kostenlos herunterladen" })
    .click();
  await page
    .getByRole("button", { name: "Transkription / Download abbrechen" })
    .click();
  await expect(page.getByRole("alert")).toContainText("abgebrochen");
  await expect(
    page.getByRole("button", { name: "base.en kostenlos herunterladen" }),
  ).toBeEnabled();
});
test("cancels transcription and reports audio failures", async ({ page }) => {
  await installModel(page);
  await page.getByRole("button", { name: "Transkription starten" }).click();
  await page
    .getByRole("button", { name: "Transkription / Download abbrechen" })
    .click();
  await expect(page.getByRole("alert")).toContainText("abgebrochen");
  await expect(
    page.getByRole("button", { name: "Transkription starten" }),
  ).toBeEnabled();
  await page.evaluate(() =>
    (window as unknown as { __speechFailure: () => void }).__speechFailure(),
  );
  await page.getByRole("button", { name: "Transkription starten" }).click();
  await expect(page.getByRole("alert")).toContainText("Audiodatei");
});

test("reveals all long-transcript segments incrementally", async ({ page }) => {
  await page.evaluate(() =>
    (window as unknown as { __manySegments: () => void }).__manySegments(),
  );
  await page.getByRole("button", { name: "Ändern", exact: true }).click();
  await expect(page.locator(".transcript-list article")).toHaveCount(100);
  await page
    .getByRole("button", { name: "Weitere Textstellen anzeigen" })
    .click();
  await expect(page.locator(".transcript-list article")).toHaveCount(101);
  await expect(page.getByText("Segment 100", { exact: true })).toBeVisible();
});
