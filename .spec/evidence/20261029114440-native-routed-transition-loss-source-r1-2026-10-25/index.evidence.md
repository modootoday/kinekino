---
id: evidence:kinekino/20261029114440
title: Original parent-loss → same transition admission
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Original parent-loss → same transition admission

2026-10-25 SDK source-only successor. Manifest `runtime/coordination/native-routed-transition-loss-r1/source5.json`, SHA256 `8F12B8B8EFF9AA5D23014371905AE67F2CBC1684D819AD69AB14B4CF76D5081A`; detailed contract in adjacent `HANDOFF.md`. Source root `runtime/development/native-routed-transition-loss-r1`. Frozen transition11, source6/UI108/import repair unchanged. Exact private five-leaf rustfmt/check exit0; Cargo/type/tests/OS/browser/network/media0.

UI's forced-parent-loss gap is source-confirmed: can_close cannot be the only transition admission boundary. Existing EnginePart.parent_lost calls OriginalAdmission.revoke before State borrowing. New trusted factory/router composition fixes the same caller slot Arc in the original router and existing EpisodeTransitions; OriginalAdmission.revoke closes that slot atomically before route revocation or State borrowing. EnginePart.begin_cleanup also revokes before borrowing for direct cleanup paths. Original registration/type/API is preserved; no arbitrary slot injection into registration, new bool authority, Drop permission or pending drain.

UI still owns explicit can_close/abort hooks and original slot lifetime. Native/execution cleanup → slot.end (original admitted job/error evidence) → provider cleanup remains mandatory. The new hook only prevents new transition admission; it does not prove cancellation, drain or native exit. New pure original-admission test is authored but unrun; active State-borrow/forced-OS-loss/complete cleanup need coherent actual validation.

Applied previously read modoo.today code-change-review-verification body `9f73865f96ec0cefcf3a4023b294400258473640df3b1d2add03dcca8192d64b` and required verdict reference `9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9` to the UI-reported ordering gap and existing callback paths. No whole-project acceptance claim.
