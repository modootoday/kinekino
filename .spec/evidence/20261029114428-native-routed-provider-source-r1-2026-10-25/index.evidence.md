---
id: evidence:kinekino/20261029114428
title: Live provider/routed SDK source successor r1
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Live provider/routed SDK source successor r1

2026-10-25. SDK owner source-only handoff; root/index/commit/push unchanged. No Cargo, rustfmt, browser, window, network, media or model execution. Existing SDK92 type/exact9 acceptance stays scoped to its frozen manifest and does not cover this successor.

## Sources

- Packet: `runtime/development/native-routed-provider-r1`.
- Exact six-leaf supply: `runtime/coordination/native-routed-provider-r1/source6.json`, SHA256 `35A3E4483DD6EF6F32F493957C7D61C52A28F4642A44E18E4A724D730D09FEDF`.
- Detailed constructor/caller/cleanup and typed job handoff: `runtime/coordination/native-routed-provider-r1/HANDOFF.md`.
- Existing three leaves: provider jobs host, catalog adapter, routed bootstrap registration. New three: provider/routed assembly, desktop browser-provider configuration, Rust parser tests.
- UI retains ownership of main, launch registration, callbacks/controls, and final lifecycle outcome. UI registers `pub mod browser_provider` in its own launch source. Media overlap needs reconciliation before coherent supply; no root source overlay here.

Media owner subsequently confirmed its repair23/raw21 and current minimal 131-error successor do not modify either infrastructure leaf (`browser_provider_jobs.rs`, `catalog.rs`). No ownership conflict for those two inputs; final coherent manifest reconciliation and UI caller acceptance are still pending. This is an owner report, not a new SDK execution receipt.

## Implemented source behavior

`prepare_provider_routed_client` consumes the original configured generic PlayerSdk and installs the original RetainingProviderJobsHost's JobsApi, CatalogArchive and exact results Arc before calling the existing routed bootstrap. Native activation still requires the original checked child. No second SDK, worker, endpoint, standalone engine or domain policy is created.

Provider capture now uses its original CatalogFiles Arc via `capture_service_with_artifacts`, instead of reopening a store through the root-based helper. Existing root-based capture API is retained. `ProviderJobsHost::files` exposes only the original Arc clone for routed resolution and GUI evidence adapters.

New application launch API reads its own explicit profile flag, requires absolute job/catalog/profile roots and NativeViewer mode, leaves unrelated flags to their original owners, and creates no default root. Existing provider constructor still owns canonical separation/profile lock validation. Parser tests are authored, not executed.

## Source-confirmed distinctions

Desktop read-only jobs injection does not supply live browser execution. Existing CLI provider constructor does register BrowserCatalogJobs and BrowserEpisodeResolveJobs. Its CatalogArchive is a stored result reader; category/series/episode navigation goes through the original jobs submission and terminal/readback sequence. Resolution inspection has no selected titles; catalog Entry carries id/title/url.

For a completed resolve, playback needs `output.receipt.resolution_artifact.artifact_id`, not `output.sha256` (body hash). Evidence inspection does not prove an unexpired address. A new explicit resolution action submits the selected original episode through the existing provider; automatic unbounded retries and guessed provider URLs are excluded.

## Required next acceptance

UI must consume the seam with the original provider retained on success/start failure/cleanup, skip its early read-only injection in the live configuration branch, and pass one original client to project/endpoint/execution owners. Existing ProviderStartFailure/PendingProviderHost and dispatcher carriers must remain owned until their explicit observations. Missing live configuration cannot silently fall back to archive-only and be counted as product completion.

After PM admits the final coherent successor: fmt, type/tests, strict clippy, focused provider/capture/config/lifecycle checks, then explicit existing CLI and GUI integration evidence. This source packet does not prove live navigation, playback, cleanup, visual acceptance or user AI quality approval.

## Applied domain skill

Read modoo.today `modootoday/software-qa/code-change-review-verification`, body SHA256 `9f73865f96ec0cefcf3a4023b294400258473640df3b1d2add03dcca8192d64b`, and required `references/verdict-table.md` SHA256 `9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9`. Applied by verifying PM/UI claims against exact constructors, dispatch/readback fields and ownership paths; findings and tests are tabulated in HANDOFF. Unrelated initial demo/scaffold/agent-permission search results were rejected.
