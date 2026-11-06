import { expect, it } from "vitest";
import { validateSelection } from "./export";
it("accepts exactly the supported clip boundaries", () => {
  expect(validateSelection(0, 61, 61)).toBeNull();
  expect(validateSelection(10, 90, 100)).toBeNull();
});
it("rejects invalid selections before invoking native export", () => {
  for (const [start, duration, total] of [
    [-1, 61, 100],
    [NaN, 61, 100],
    [0, 60, 100],
    [0, 91, 100],
    [40, 61, 100],
    [0, 61, 3],
  ]) {
    expect(validateSelection(start, duration, total)).not.toBeNull();
  }
});
