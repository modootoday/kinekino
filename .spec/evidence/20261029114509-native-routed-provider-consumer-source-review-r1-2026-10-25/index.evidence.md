---
id: evidence:kinekino/20261029114509
title: "UI consumer r2: SDK/provider source review"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# UI consumer r2: SDK/provider source review

2026-10-25. UI requested review of four frozen leaves in `runtime/development/product-routed-consumer-r2`: `src/main.rs` and `src/assembly/meari/product_viewer_launch/routed.rs`, `routed/setup.rs`, `routed/provider_cleanup.rs`. Read the existing cleanup driver and the supplied dispatcher adapter to trace original ownership. Frozen sources unchanged; no Cargo, formatting, window, browser, media or network execution by this review.

Input manifest `source12.json`: SHA256 `2E4F2922A06616A5C359AEA24FC296EF1373B4A5CAE8B1EEFBA0B3077853D0D2`. Coherent108 manifest `coherent-supply.json`: SHA256 `1E82579A84E2A8B005592F733A0E6F351B2EA06256A6A0D01C84688107EA8459`. These exact manifest hashes were observed; no fresh whole-supply hash audit or build is claimed here.

| Requested contract | Source verdict and evidence | Unverified acceptance |
| --- | --- | --- |
| Same original provider/results/client | Source confirmed: routed.rs 62 starts one retaining provider; 68 calls `provider.files()`; 79 calls `prepare_provider_routed_client(base, &provider, dispatcher.handle())`; bootstrap `into_parts` supplies initial client and original activation. main 118 reads project through that client and 125 gives `client.clone()` to LocalHost. Live provider configuration skips the read-only injection. | Coherent type/strict checks; focused actual client/job/result identity and native activation checks. |
| Partial provider startup retains its original carrier | Source confirmed: setup.rs 39 invokes `failure.into_shutdown(provider_cleanup::original_deadline())`; any pending original owner enters `observe`, and the resulting cleanup outcome remains in the setup failure. Initial mpv verification precedes provider startup. | Inject partial provider start failure; observe original jobs/browser/profile guards and first error through Pending/Complete. |
| Provider deadline expiry preserves original owner | Source confirmed: provider_cleanup.rs 60 invokes `pending.poll(self.deadline)` with its stored original deadline. Pending is put back before budget-exhausted status; `into_parts` returns that same PendingProviderHost in the cleanup outcome. It does not fabricate a completion receipt. | Focused cutoff/error case asserting identity, deadline and pending ownership across repeated observation. |
| Normal cleanup keeps native/execution, provider and dispatcher completions distinct | Source confirmed: routed.rs 181 returns original provider/dispatcher on unsuccessful viewer cleanup. After successful viewer cleanup, line 192 closes provider, then drives dispatcher cleanup. Both cleanup outcomes remain owned; success requires provider Complete and dispatcher Joined. | Actual normal and faulted shutdown receipts on final coherent input. |
| Initial project/LocalHost failure retains configured runtime owners | Source confirmed: main.rs 123 and 131 route the original error through `abort_native_setup`; it consumes the original bootstrap via `abort_setup`, closes pending activation and retains routed/client/provider/dispatcher outcomes in the setup failure. `fatal_native_setup` holds that carrier to explicit abnormal process exit. No implicit successful Drop receipt. | Inject project selection and endpoint startup failures; inspect original cleanup/first-error evidence. |

No source-level change was requested for these scoped contracts. This is not type correctness, live browsing, media playback, OS cleanup or user quality acceptance. UI controls/catalog selection are explicitly outside this frozen12 supply and need their own successor and actual validation.

Applied previously read modoo.today `modootoday/software-qa/code-change-review-verification` body SHA256 `9f73865f96ec0cefcf3a4023b294400258473640df3b1d2add03dcca8192d64b` and required `references/verdict-table.md` SHA256 `9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9`: traced only the UI-requested contracts, separated source evidence from unrun acceptance, and avoided treating a plausible ownership assertion as runtime proof.
