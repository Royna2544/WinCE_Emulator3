#!/usr/bin/env python3
"""Read-only, hash-pinned measurements from the historical E-38 run.

This does not launch the guest, inspect current mounts, or certify the current
source. Frame hashes are observations, not a claim about image contents.
"""

from __future__ import annotations

import hashlib
import json
from collections import defaultdict
from datetime import datetime
from pathlib import Path
from typing import Any, Iterable


EVIDENCE = Path(__file__).resolve().parents[1] / "evidence" / "e37-live-20261014T051228Z"
EXPECTED_SHA256 = {
    "app.stdout.raw.txt": "F386537C12C3ADFD117A8D1A52D5BB20EBE3A77FACED345AE7EC2437597095AA",
    "memory.jsonl": "627DA624A893F141EB93E72FFBD7C813ABC54547998296EC3B9470CC3067BD65",
    "observer-get.jsonl": "0174D0854A6EEF507A1EDC8E3AF24DE627B6656457CB84A828F843153B9305F9",
    "result.json": "C204B348DBDF6A958E7939A930D268BF817C30D36E9649B0B955AB02D13091C9",
}
BUSY_MIN_INSNS = 900_000
PROFILE_MARKER = "ce.slice_profile: slice profile "


def verified_bytes(evidence_dir: Path = EVIDENCE) -> dict[str, bytes]:
    """Read only four named files, retaining exactly the bytes that were hashed."""
    payloads = {}
    for name, expected in EXPECTED_SHA256.items():
        payload = (evidence_dir / name).read_bytes()
        actual = hashlib.sha256(payload).hexdigest().upper()
        if actual != expected:
            raise ValueError(f"historical input hash mismatch: {name}: {actual}")
        payloads[name] = payload
    return payloads


def jsonl(payload: bytes) -> list[dict[str, Any]]:
    return [json.loads(line) for line in payload.decode("utf-8").splitlines() if line.strip()]


def _utc(value: str) -> datetime:
    return datetime.fromisoformat(value.replace("Z", "+00:00"))


def frame_observations(records: Iterable[dict[str, Any]]) -> dict[str, Any]:
    records = list(records)
    frames = [row for row in records if row.get("name") == "frame" and row.get("status") == 200]
    if not frames:
        raise ValueError("no successful frame observations")

    segments: list[dict[str, Any]] = []
    previous = None
    for row in frames:
        started = row["startedUtc"]
        received = row["receivedUtc"]
        if _utc(started) > _utc(received):
            raise ValueError("frame poll ends before it starts")
        if previous is not None and _utc(started) < _utc(previous["startedUtc"]):
            raise ValueError("frame poll order is not monotone")

        if previous is None or row["sha256"] != previous["sha256"]:
            # The image was sampled at an unknown instant during each request.
            # Previous *request start* to new response is a conservative
            # bracket; previous response to new response would be too narrow.
            bracket = None if previous is None else {
                "earliestUtc": previous["startedUtc"],
                "latestUtc": received,
                "interpretation": "at least one unobserved frame change; intermediate frames unknown",
            }
            segments.append({
                "ordinal": len(segments) + 1,
                "sha256": row["sha256"],
                "firstObservedUtc": received,
                "lastObservedUtc": received,
                "successfulPolls": 0,
                "changeBracket": bracket,
            })
        segments[-1]["lastObservedUtc"] = received
        segments[-1]["successfulPolls"] += 1
        previous = row

    saved = [row["sha256"] for row in records if row.get("name") == "changed-frame-saved"]
    if saved != [segment["sha256"] for segment in segments]:
        raise ValueError("saved-frame ledger differs from successful frame-hash sequence")
    return {
        "successfulFramePolls": len(frames),
        "hashSegments": segments,
        "savedFrameEvents": len(saved),
        "firstImageBeforeObservation": "unknown",
        "visualContentAndMapReadiness": "unknown from frame hashes",
    }


def sampled_memory(records: Iterable[dict[str, Any]], host_pid: int) -> dict[str, Any]:
    rows = list(records)
    if not rows:
        raise ValueError("no host memory samples")
    if any(row["processId"] != host_pid for row in rows):
        raise ValueError("memory samples include another host process")
    if any(row["workingSetBytes"] < 0 or row["privateBytes"] < 0 for row in rows):
        raise ValueError("negative host memory sample")

    rss = max(rows, key=lambda row: row["workingSetBytes"])
    private = max(rows, key=lambda row: row["privateBytes"])
    return {
        "sampleCount": len(rows),
        "sampledPeakRssBytes": rss["workingSetBytes"],
        "sampledPeakRssMiB": round(rss["workingSetBytes"] / 2**20, 3),
        "sampledPeakRssElapsedSeconds": rss["elapsedSeconds"],
        "sampledPeakPrivateBytes": private["privateBytes"],
        "sampledPeakPrivateMiB": round(private["privateBytes"] / 2**20, 3),
        "sampledPeakPrivateElapsedSeconds": private["elapsedSeconds"],
        "unsampledTruePeak": "unknown",
    }


def _profile_fields(line: str) -> dict[str, str] | None:
    if PROFILE_MARKER not in line:
        return None
    fields = {}
    for part in line.split(PROFILE_MARKER, 1)[1].split():
        key, separator, value = part.partition("=")
        if separator:
            fields[key] = value
    for required in (
        "setup_ms", "hooks_ms", "run_ms", "snapshot_ms", "sync_out_ms",
        "guest_insns", "process_id", "native_result_err", "native_capture_result",
        "snapshot_context_error", "mapped_blob_sync_error", "immediate_process_sync_error",
    ):
        if required not in fields:
            raise ValueError(f"profile row missing {required}")
    return fields


def _weighted_mips(insns: int, run_ms: int) -> float | None:
    return round(insns / (1000 * run_ms), 3) if run_ms > 0 else None


def slice_profiles(lines: Iterable[str]) -> dict[str, Any]:
    emitted = completed = error_rows = busy_count = mismatch_count = unattributed = 0
    total = defaultdict(int)
    by_pid: dict[int, dict[str, int]] = defaultdict(lambda: defaultdict(int))

    for line in lines:
        fields = _profile_fields(line)
        if fields is None:
            continue
        emitted += 1
        successful = (
            fields["native_result_err"] == "None"
            and fields["native_capture_result"] == "Ok(())"
            and fields["snapshot_context_error"] == "None"
            and fields["mapped_blob_sync_error"] == "None"
            and fields["immediate_process_sync_error"] == "None"
        )
        if not successful:
            error_rows += 1
            continue
        completed += 1
        guest_insns = int(fields["guest_insns"])
        if guest_insns < BUSY_MIN_INSNS:
            continue
        busy_count += 1
        pid_text = fields.get("native_start_pid")
        native_pid = int(pid_text) if pid_text is not None else None
        if native_pid is None:
            unattributed += 1
        elif native_pid != int(fields["process_id"]):
            mismatch_count += 1

        values = {
            "guestInstructions": guest_insns,
            "runMilliseconds": int(fields["run_ms"]),
            "setupMilliseconds": int(fields["setup_ms"]),
            "hooksMilliseconds": int(fields["hooks_ms"]),
            "snapshotMilliseconds": int(fields["snapshot_ms"]),
            "syncOutMilliseconds": int(fields["sync_out_ms"]),
        }
        for key, value in values.items():
            total[key] += value
            if native_pid is not None:
                by_pid[native_pid][key] += value
        if native_pid is not None:
            by_pid[native_pid]["busySlices"] += 1

    def presented(stats: dict[str, int], count: int) -> dict[str, Any]:
        run_ms = stats["runMilliseconds"]
        return {
            "busySlices": count,
            **{key: stats[key] for key in (
                "guestInstructions", "runMilliseconds", "setupMilliseconds",
                "hooksMilliseconds", "snapshotMilliseconds", "syncOutMilliseconds",
            )},
            "runOnlyWeightedMips": _weighted_mips(stats["guestInstructions"], run_ms),
            "meanSetupPlusHooksMilliseconds": round(
                (stats["setupMilliseconds"] + stats["hooksMilliseconds"]) / count, 3
            ) if count else None,
        }

    return {
        "emittedProfileRows": emitted,
        "completedProfileRows": completed,
        "errorProfileRows": error_rows,
        "busyThresholdGuestInstructions": BUSY_MIN_INSNS,
        "completedBusy": presented(total, busy_count),
        "byNativeStartPid": {
            str(pid): presented(stats, stats["busySlices"])
            for pid, stats in sorted(by_pid.items())
        },
        "busyRowsWithProcessIdDifferentFromNativeStartPid": mismatch_count,
        "busyRowsWithoutNativeStartPid": unattributed,
        "unprofiledSliceAndSetupTime": "unknown",
    }


def validate_historical_run(evidence_dir: Path = EVIDENCE) -> dict[str, Any]:
    payloads = verified_bytes(evidence_dir)
    result = json.loads(payloads["result.json"])
    if result["stdoutSha256"] != EXPECTED_SHA256["app.stdout.raw.txt"]:
        raise ValueError("run result does not pin the same stdout")
    return {
        "run": "historical E-38 / E-37 live capture 20261014T051228Z",
        "inputSha256": EXPECTED_SHA256,
        "hostProcessId": result["processId"],
        "frameObservations": frame_observations(jsonl(payloads["observer-get.jsonl"])),
        "hostMemory": sampled_memory(jsonl(payloads["memory.jsonl"]), result["processId"]),
        "sliceProfile": slice_profiles(payloads["app.stdout.raw.txt"].decode("utf-8").splitlines()),
        "limits": [
            "Historical evidence only; current source and live readiness are unmeasured.",
            "Frame hashes do not establish image contents or map readiness; changes between polls are unknown.",
            "Memory maxima are sampled peaks, not the true continuous peak.",
            "Weighted MIPS uses completed busy rows' logged run_ms only, excluding setup and unprofiled time.",
        ],
    }


if __name__ == "__main__":
    print(json.dumps(validate_historical_run(), indent=2))
