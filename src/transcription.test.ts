import { expect, it } from "vitest";
import { timestamp } from "./transcription";
it("formats exact millisecond timestamps, including long videos", () => {
  expect(timestamp(3661.234)).toBe("01:01:01.234");
  expect(timestamp(0)).toBe("00:00:00.000");
  expect(timestamp(59.9996)).toBe("00:01:00.000");
});
it("does not display invalid timestamps", () => {
  expect(timestamp(NaN)).toBe("–");
  expect(timestamp(-1)).toBe("–");
});
