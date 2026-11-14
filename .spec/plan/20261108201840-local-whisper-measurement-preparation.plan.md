---
id: plan:kinekino/20261108201840
title: Local whisper measurement preparation
status: active
created: 2026-11-08T20:18:40Z
updated: 2026-11-08T20:19:04Z
rel:
  governs: [crates/kinekino-inference-local/src/measurement_prep.rs]
---

# Local whisper measurement preparation

## Goal

Prep for a later local whisper.cpp measurement exists, and nobody can mistake that prep for a completed measurement.

`kinekino-inference-local` already wraps `whisper-cli` (`WhisperCliBackend`) and already generates 16 kHz mono PCM16 WAV segments (`wav_segment`). This slice adds a format fixture built from that helper, a fixed list of measurement items, a reproduction procedure, and an API that reports prep readiness without spawning the CLI or downloading a model. `start_measurement` returns `Unsupported` until there is an explicit approval. A blank record is always `NotRun` with empty values.

This prep is not a measured result, not a quality approval, and not an install of whisper.cpp.

## Authority

| Concern | Who decides |
|---|---|
| Whether a real whisper.cpp run may start | User approval (P7). This crate refuses until then. |
| Whether automatic numbers count as quality | The user, in the real GUI. Tests never stand in for that. |
| WAV format of the format fixture | Existing `wav_segment` contract (16 kHz, mono, PCM16). |
| How the pack is obtained | The operator installs `whisper-cli` and the ggml file locally. The crate does not download them. |
| Which speech recording is used for character error rate | User approval plus a licensed fixture supplied later. This slice does not add one. |

## Evidence

Observed on 2026-11-09 (Asia/Seoul) against main `a1171f11` and this branch:

- `WhisperCliBackend` already parses `whisper-cli -oj` JSON, records provenance `local-pack/whisper.cpp` and cost `0.0`, and checks that the program and model paths are files. Contract tests use `KK_FAKE_WHISPER`; they do not run a real model.
- The crate docs already say the pack is user-installed and the crate never downloads anything.
- `wav_segment` writes a 44-byte PCM WAV header. The new format fixture is two seconds of silence at 16 kHz produced through that helper.
- Six new tests (`whisper_measurement_prep`) first failed on `unimplemented!` stubs (and an empty procedure). After the implementation they passed: the fixture is 16 kHz mono PCM16 of 2.0 s, required items are present and CER is marked as needing a speech reference, the procedure text forbids download and quality-approval substitution, a missing model file is not created, and a probe `whisper-cli` that would write a sentinel is never spawned by `assess_prep` or `start_measurement`.

No wall-clock, memory, transcript, or CER numbers were collected. No model or dataset was downloaded.

## Unverified

- How real `whisper-cli` behaves on this silent format fixture (it may emit no chunks). That is why the fixture is format-only.
- How peak memory will be sampled on Linux (process cgroup, `/proc`, or something else) once measurement is approved.
- Which ggml filename the operator will actually install. The adapter default pack name remains `ggml-large-v3-turbo.bin`; this slice does not fetch it.
- Whether a later licensed speech fixture will be user-supplied audio or a separately obtained corpus. FLEURS and other datasets are not fetched here.
- Host, CPU, and `whisper-cli` version for any future numbers. Those belong in the `conditions` item of a later record.

## Steps

1. **Prep artifacts (this slice).** Done when the format fixture, item list, and procedure are present, `assess_prep` reports `artifacts_ready` without setting `measured`, and `start_measurement` returns `Unsupported` mentioning approval and prep. Completion is the new tests passing, not a measurement record with values.
2. **Approved measurement (later).** After explicit approval, run three transcriptions of the chosen fixture with a user-installed pack, fill every required item (CER only if a licensed speech fixture was supplied), and write an evidence record. Completion is that record with `measured` true plus the run conditions. This step is not started here.

## Rollback

Phase 1 rolls back by removing `measurement_prep.rs`, its re-exports, and `tests/whisper_measurement_prep.rs`. No model, dataset, or `whisper-cli` install is created, so there is nothing to uninstall. Phase 2 does not land in this slice; a later measurement record can be deleted without changing this prep.

## Record

- Recommendation: reuse the existing local adapter and `wav_segment`; prepare the reproduction procedure, format fixture, and measurement items only; refuse unapproved measurement; do not treat prep as a completed measurement.
- skill: no matching domain skill from `skill_find` (queries: local whisper measurement prep without download or inference; measurement protocol with fixtures and required metrics). Local skills used: plan-authoring, existing-capability-before-build.
