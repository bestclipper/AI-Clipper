import { describe, it, expect } from "vitest";
import { formatDuration, isVideoName, OUTPUT_PROFILE } from "./pipeline";
describe("video import", () => {
  it("accepts uppercase extensions and paths containing dots", () => {
    expect(isVideoName("/streams/day.1/VIDEO.MP4")).toBe(true);
  });
  it("rejects disguised files and unsupported formats", () => {
    expect(isVideoName("video.mp4.exe")).toBe(false);
    expect(isVideoName("video.mkv")).toBe(false);
  });
});
describe("duration display", () => {
  it("supports long streams", () =>
    expect(formatDuration(36610.8)).toBe("10:10:10"));
  it("handles invalid metadata", () => {
    for (const n of [NaN, Infinity, -1]) expect(formatDuration(n)).toBe("–");
  });
});
it("keeps the future output contract explicit", () => {
  expect(OUTPUT_PROFILE).toMatchObject({
    count: 5,
    minSeconds: 61,
    maxSeconds: 90,
    width: 1080,
    height: 1920,
    subtitleLanguage: "en",
    format: "mp4",
  });
});
