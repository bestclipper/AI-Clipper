import { describe, it, expect } from "vitest";
import { playlistStep, validPlaylist, type ClipSegment } from "./analysis";
const part = (start: number, end: number): ClipSegment => ({
  start,
  end,
  duration: end - start,
  firstUnit: 0,
  lastUnit: 0,
});
describe("source playlist", () => {
  it("jumps across source gaps and stops at final boundary", () => {
    const parts = [part(18, 45), part(170, 220)];
    expect(validPlaylist(parts, 240)).toBe(true);
    expect(playlistStep(parts, 0, 44)).toEqual({ index: 0, finished: false });
    expect(playlistStep(parts, 0, 45)).toEqual({
      index: 1,
      seek: 170,
      finished: false,
    });
    expect(playlistStep(parts, 1, 220).finished).toBe(true);
  });
  it("handles three or more segments without rendering a file", () => {
    const p = [part(0, 20), part(40, 65), part(100, 130)];
    expect(playlistStep(p, 1, 65)).toEqual({
      index: 2,
      seek: 100,
      finished: false,
    });
    expect(validPlaylist(p, 130)).toBe(true);
  });
  it("rejects out of bounds, overlap, backwards and nonfinite values", () => {
    for (const parts of [
      [],
      [part(20, 10)],
      [part(-1, 50)],
      [part(0, 80), part(50, 100)],
      [part(0, Infinity)],
    ])
      expect(validPlaylist(parts, 100)).toBe(false);
  });
});
