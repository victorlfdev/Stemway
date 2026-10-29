import { test } from "node:test";
import assert from "node:assert/strict";
import {
  gains,
  presetMix,
  restoreQueue,
  movePending,
  stemId,
} from "./studio.js";
test("solo and mute preserve gain and export the audible mix", () => {
  assert.deepEqual(
    gains({ vocals: { volume: 0.4, soloed: true }, drums: { volume: 0.7 } }, [
      "vocals",
      "drums",
    ]),
    [0.4, 0],
  );
  assert.deepEqual(
    gains({ vocals: { volume: 0.4, soloed: true, muted: true } }, ["vocals"]),
    [0],
  );
  assert.equal(presetMix("karaoke", ["vocals", "drums"]).vocals.muted, true);
});
test("interrupted tasks become pending without repeating completed work", () => {
  const queue = restoreQueue([
    { id: "1", path: "a", model: "demucs", status: "running" },
    { id: "2", path: "b", model: "demucs", status: "done" },
  ]);
  assert.deepEqual(
    queue.map((q) => q.status),
    ["pending", "done"],
  );
  assert.equal(movePending(queue, "1", 1), queue);
  assert.equal(stemId("C:\\music\\vocals.wav"), "vocals");
});
