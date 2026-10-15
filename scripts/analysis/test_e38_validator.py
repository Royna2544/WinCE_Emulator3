"""Bounded checks for the historical, read-only E-38 validator."""

import tempfile
import unittest
from pathlib import Path

from e38_validator import frame_observations, slice_profiles, validate_historical_run, verified_bytes


def profile_line(*, process_id=99, native_start_pid=7, error="None"):
    return (
        "2026-10-14T00:00:00Z DEBUG ce.slice_profile: slice profile "
        "setup_ms=2 hooks_ms=3 run_ms=30 snapshot_ms=1 sync_out_ms=1 "
        f"guest_insns=900000 process_id={process_id} native_start_pid={native_start_pid} "
        f"native_result_err={error} native_capture_result=Ok(()) "
        "snapshot_context_error=None mapped_blob_sync_error=None "
        "immediate_process_sync_error=None"
    )


class HistoricalE38ValidatorTests(unittest.TestCase):
    def test_preserved_run_matches_independent_receipt_counts(self):
        report = validate_historical_run()
        frames = report["frameObservations"]
        memory = report["hostMemory"]
        profile = report["sliceProfile"]

        self.assertEqual((frames["successfulFramePolls"], frames["savedFrameEvents"]), (166, 4))
        self.assertEqual(len(frames["hashSegments"]), 4)
        self.assertEqual(frames["hashSegments"][-1]["successfulPolls"], 151)
        self.assertEqual(memory["sampleCount"], 160)
        self.assertEqual(memory["sampledPeakRssBytes"], 451_596_288)
        self.assertEqual(memory["sampledPeakPrivateBytes"], 466_567_168)
        self.assertEqual((profile["emittedProfileRows"], profile["completedProfileRows"], profile["errorProfileRows"]), (6241, 6240, 1))
        combined = profile["completedBusy"]
        self.assertEqual((combined["busySlices"], combined["guestInstructions"], combined["runMilliseconds"]), (1714, 1_713_585_486, 100_095))
        self.assertEqual(combined["runOnlyWeightedMips"], 17.120)
        self.assertEqual((profile["byNativeStartPid"]["1"]["busySlices"], profile["byNativeStartPid"]["1"]["runOnlyWeightedMips"]), (481, 26.359))
        self.assertEqual(profile["busyRowsWithoutNativeStartPid"], 0)

    def test_synthetic_process_id_is_not_attribution_authority(self):
        profile = slice_profiles([profile_line(process_id=99, native_start_pid=7)])
        self.assertEqual(profile["completedBusy"]["busySlices"], 1)
        self.assertEqual(list(profile["byNativeStartPid"]), ["7"])
        self.assertEqual(profile["byNativeStartPid"]["7"]["runOnlyWeightedMips"], 30.0)
        self.assertEqual(profile["busyRowsWithProcessIdDifferentFromNativeStartPid"], 1)

    def test_native_error_is_not_completed_busy_work(self):
        profile = slice_profiles([profile_line(error="Some(READ_UNMAPPED)")])
        self.assertEqual((profile["emittedProfileRows"], profile["errorProfileRows"]), (1, 1))
        self.assertEqual(profile["completedBusy"]["busySlices"], 0)

    def test_frame_change_bracket_starts_at_previous_request_start(self):
        old = {"name": "frame", "status": 200, "sha256": "old", "startedUtc": "2026-01-01T00:00:00Z", "receivedUtc": "2026-01-01T00:00:01Z"}
        new = {"name": "frame", "status": 200, "sha256": "new", "startedUtc": "2026-01-01T00:00:02Z", "receivedUtc": "2026-01-01T00:00:03Z"}
        records = [old, {"name": "changed-frame-saved", "sha256": "old"}, new, {"name": "changed-frame-saved", "sha256": "new"}]
        report = frame_observations(records)
        bracket = report["hashSegments"][1]["changeBracket"]
        self.assertEqual(bracket["earliestUtc"], old["startedUtc"])
        self.assertEqual(bracket["latestUtc"], new["receivedUtc"])

    def test_mismatched_historical_input_fails_closed(self):
        with tempfile.TemporaryDirectory() as scratch:
            (Path(scratch) / "app.stdout.raw.txt").write_bytes(b"tampered")
            with self.assertRaisesRegex(ValueError, "historical input hash mismatch"):
                verified_bytes(Path(scratch))


if __name__ == "__main__":
    unittest.main()
