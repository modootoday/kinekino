# kinekino

An AI-assisted video player and clip workbench for creators, driven by one JSON SDK shared by the GUI, the CLI and agents.

![status: pre-alpha](https://img.shields.io/badge/status-pre--alpha-orange)
![rust 1.99.1](https://img.shields.io/badge/rust-1.99.1-blue)
![platforms: Windows | Linux](https://img.shields.io/badge/platforms-Windows%20%7C%20Linux-lightgrey)

## Why

- Watch, detect and skip segments, transcribe and edit subtitles, and cut clips and cards in one tool.
- Every action is a typed SDK command, so the GUI (`player`), the CLI (`kinekino`) and AI agents behave the same.
- Models run locally (ONNX / Hugging Face pipeline); nothing is uploaded unless you ask.
- One core with OS policy behind ports (`kinekino-platform`), so Windows and Linux share the same code.

Status legend: **Available** = built and tested in this repo; **Partial** = works with limits; **Planned** = designed, not built.

## Contents

- [Install](#install) · [Quick start](#quick-start) · [Concepts](#concepts) · [Commands](#commands)
- [Using with AI agents](#using-with-ai-agents) · [Guides](#guides) · [Exit codes and schemas](#exit-codes-and-schemas)
- [Contributing](#contributing) · [License](#license)

## Install

| Method | Status |
| --- | --- |
| Build from source (`cargo build --locked`) | Available |
| Prebuilt Windows / Linux binaries | Planned |
| Package managers (winget, cargo install) | Planned |

Requirements: Rust 1.99.1 (pinned in `rust-toolchain.toml`). On Windows also the MSVC Build Tools and Windows SDK.
Media tools (`mpv`, ffmpeg) and models are not bundled; install them only for features that need them.

```console
$ git clone <this repository> kinekino && cd kinekino
$ cargo build --locked -p kinekino-cli
```

## Quick start

These commands run without media, models or a GUI. Output is a JSON envelope: `{"ok": …, "data": …, "error": …}`.

```console
$ cargo run -q -p kinekino-cli -- version
{"api_version":"1","request_id":"cli","ok":true,"data":{"api_version":"1",…,"sdk_version":"0.1.0"},"error":null}
$ cargo run -q -p kinekino-cli -- capabilities
{"api_version":"1","request_id":"cli","ok":true,"data":{…,"commands":[{"command":"asset_artifacts.inspect","status":"unsupported"},…]},"error":null}
$ cargo run -q -p kinekino-cli -- schema playback status
{"api_version":"1","request_id":"cli","ok":true,"data":{"command":"playback.status",…},"error":null}
$ cargo run -q -p kinekino-cli -- --help
$ cargo test --workspace --features kinekino-vad-worker/test-control-process
$ cargo xtask lint-structure
```

## Concepts

| Concept | What it is | Status |
| --- | --- | --- |
| SDK command | `group.action` request with a JSON schema; one envelope for every client | Available |
| Hosts | Local, playback, job and execution hosts that own long-running work | Partial |
| Playback | WebView2 / CEF engines with segment skip and audio handoff | Partial (Windows) |
| Subtitles | Transcribe, diarize, convert and edit SRT/VTT | Partial |
| Proposals and projects | Clip, card and asset projects with revisioned saves | Partial |
| Jobs | Batched background work with explicit retry and cancel | Partial |
| OS ports | `process`, `ipc`, `clock`, `window` policies in `kinekino_core::platform` | Available (Windows runtime unverified) |
| Linux GUI | Native window and playback on X11/Wayland | Planned |

Design records live in [`.spec/adr`](.spec/adr) (for example [ADR 0009, native window port](.spec/adr/20261030190000-adr-0009-native-window-port.adr.md)).

## Commands

Run `kinekino schema <group> <action>` for the full request schema of any command.
The table is generated from `capabilities`; a test fails when it drifts.

<!-- kinekino:commands:begin -->
| Group | Commands |
| --- | --- |
| `asset_artifacts` | `inspect` |
| `asset_sources` | `export`, `inspect` |
| `asset_projects` | `create`, `inspect`, `save_revision` |
| `clip_projects` | `create`, `inspect`, `save_revision` |
| `clip_sources` | `inspect` |
| `job_batches` | `create`, `inspect`, `submit`, `refresh`, `retry_failed` |
| `card_projects` | `create`, `inspect`, `save_revision` |
| `reviews` | `media.prepare`, `media.inspect`, `playback.open`, `playback.status`, `playback.close`, `playback.control`, `association.inspect`, `association.create`, `association.revise`, `range_draft.read`, `range_draft.save`, `inspect`, `record_decision`, `decisions`, `read_text_asset` |
| `production` | `export_card_review_png`, `render_short`, `download_media`, `capture_frame`, `export_snapshot`, `export_cards`, `segment_subject`, `build_assets`, `package_projection` |
| `version` | `version` |
| `capabilities` | `capabilities` |
| `schema` | `schema` |
| `catalog` | `search`, `read_raw` |
| `episodes` | `list`, `inspect_resolution`, `resolve` |
| `playback` | `open`, `status`, `audio_handoff`, `play`, `pause`, `seek`, `engine`, `set_volume`, `set_muted`, `set_rate`, `select_track`, `previous`, `next`, `transition.observe`, `transition.commit`, `transition.release`, `close` |
| `segments` | `detect`, `open`, `inspect`, `evaluate`, `skip`, `skip_export`, `export_list`, `export_peek`, `export_consume`, `override`, `close`, `edit`, `accept_skip`, `set_skip_policy` |
| `subtitles` | `transcribe`, `diarize`, `convert`, `export_range`, `from_transcript`, `speakers`, `align_speakers`, `edit_cues`, `edit_speakers` |
| `models` | `list`, `inspect`, `install`, `verify`, `remove` |
| `jobs` | `get`, `submit`, `list`, `watch`, `cancel`, `retry` |
| `projects` | `create`, `inspect`, `save_revision`, `reconnect_source`, `export` |
| `proposals` | `create`, `list`, `import_historical`, `inspect`, `inspect_source_context`, `edit`, `accept`, `export_subtitles`, `inspect_output_review_target`, `record_output_decision`, `output_decisions` |
<!-- kinekino:commands:end -->

Per-command availability depends on the host and installed models; `capabilities` reports `supported` or `unsupported` at run time.

## Using with AI agents

| Interface | Status |
| --- | --- |
| CLI with JSON output (`kinekino <command>`, `schema`, `capabilities`) | Available |
| MCP server exposing the same commands | Planned |
| [`llms.txt`](llms.txt) index for agents | Available |

Agents working **on** this repository follow [`AGENTS.md`](AGENTS.md); this README is for people **using** the tool.

## Guides

- [Design and ADRs](.spec/adr) · [Guides](.spec/guide) · [Plans](.spec/plan)
- [Build cache guide](.spec/guide/build-cache.guide.md)
- [Third-party sources](THIRD_PARTY.md)

## Exit codes and schemas

| Code | Meaning |
| --- | --- |
| 0 | success |
| 1 | internal |
| 2 | invalid_argument |
| 3 | unsupported / needs_input |
| 4 | not_found |
| 5 | revision_conflict |
| 6 | permission_denied |
| 7 | resource_exhausted |
| 8 | source_expired / engine_failed |
| 9 | cancelled |
| 10 | timeout |

Request and response schemas: `kinekino schema <group> <action>`.

## Contributing

Read [`AGENTS.md`](AGENTS.md) first. Before a change lands: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
workspace tests, `cargo xtask lint-structure` (no new violations) and the spec check must all pass.

## License

Licensed under the [MIT License](LICENSE). Copyright (c) 2026 modootoday. Third-party notices are in [`THIRD_PARTY.md`](THIRD_PARTY.md).
