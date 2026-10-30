---
id: evidence:kinekino/20261029113957
title: "Normal GUI mount261: ManualRepair entry failure"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Normal GUI mount261: ManualRepair entry failure

2026-10-25. This is an observed route failure, not visual acceptance or AI quality approval.

## Original execution

Central source manifest SHA256 `60DFB2E3B8926C0EEC5FE6B03FD5C91D4B8302DDD9CDF31D03CC6BEFADDCD2B9`. Normal player SHA256 `6D14B657310368662A36FE684D876ED9DA582EF019FAD8BF7FEC493412A80C00`, 161451520 bytes. PID58524, creation UTC 2026-10-25T02:49:03.7533246Z. Exact original argv and custody are preserved in [launch owner](../../../../../runtime/coordination/normal-gui-mount261-r1/launch-owner.json). Main monitor/default placement; no DPI, VM or device changes. One normal player, no extra player or fixture executable.

The proposal list was refreshed and actual9-review-r1-gemma270-correction / ManualRepair Correction explicitly selected. Open was clicked once at 02:50:12.033 UTC. No save, edit, adoption or quality verdict was performed. The repair component did not appear; the prior chooser remained visible. [Before Open](../../../../../runtime/coordination/normal-gui-mount261-r1/before-open-1792896612033.png), [failure screenshot](../../../../../runtime/coordination/normal-gui-mount261-r1/failed-inspected-1792896650439-0.png) and accompanying observed state JSON remain preserved.

## Observed boundary

[stderr](../../../../../runtime/coordination/normal-gui-mount261-r1/stderr.raw) records generation2: Draining → Starting → Inspecting → Inspected → Failed, first error Callback::Adapter. AwaitingArm was not reached. This locates failure after the original current SDK inspection and before successful live mount completion; the existing diagnostic does not distinguish prepare, session bind and commit. It does not prove a particular props error by itself.

[stdout](../../../../../runtime/coordination/normal-gui-mount261-r1/stdout.raw) records 18 frame callbacks/last scene18 with routed-frame first_functional_error=null. The prior FocusChanged error did not recur in this run. Route failures retained by LiveOpening are a separate error channel, so the null frame error and overall cleanup success do not establish successful proposal entry.

One normal Alt+F4 request at 02:50:50.441 UTC closed the original player. PID absence was verified. Normal OS exit, WorkersQuiescent(Closed), dispatcher joined/faults0, execution stopped/unacknowledged0, LocalHost pipe/SDK joined and cleanup_complete=true were recorded. No GUI replay was performed.

## Source-only follow-up and central handoff

[ManualRepair packet](../../../../../runtime/development/product-manual-repair-mount-r1/HANDOFF.md), source3 SHA256 `38EAA2A5AD4E6B9A1D4F5FB247A28A68FF97E11E2E3B87442A20B24BDA8816B7`, separates production props correction, original SDK synthetic regression and bounded private mount stage/static reason diagnostics. SDK independent read-only review confirmed five unconsumed props absent from the ManualRepair declaration. Strict NodeSlot guards remain unchanged. This is a source diagnosis awaiting centralized execution, not an observed error-message claim.

The synthetic regression uses real SDK CreateProposal to create NeedsRepair/no-draft documents for all three tasks in a process-time scratch root, then the original chooser prepared Open/current inspection/release and strict mount receipt/proof path. Actual9 namespaces and external models are not used. rustfmt/check exit0; Cargo/type/test/clippy and GUI successor execution remain central-owner work. No new implementation beyond this handed-off packet is authorized by the waiting state.

Applied previously read modoo.today software-qa/defect-triage-evidence and software-qa/test-fixture-and-mock-build: preserve original failure evidence, distinguish cleanup from product success and reproduce the actual panel flavor with original SDK ownership. Source and task selection review was coordinated with the SDK session; no claim of user quality approval.
