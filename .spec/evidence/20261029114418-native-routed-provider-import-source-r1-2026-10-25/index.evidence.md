---
id: evidence:kinekino/20261029114418
title: Provider capture constructor import fix
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Provider capture constructor import fix

2026-10-25. UI owner reported the first coherent108 type attempt stopped with exit101 and E0425/E0433 at `src/infrastructure/catalog.rs` lines44/45: new helper used `CatalogCaptureService` without importing it. SDK source review did not establish type correctness; no UI/main/CLI type success follows from that stopped build.

The SDK owner copied the frozen provider-r1 catalog leaf to a separate packet and added `CatalogCaptureService` to its existing use list only (plus line wrapping). Function bodies, policy and frozen source6/coherent108 are unchanged. No execution or root/index mutation by SDK.

- Source1: `runtime/coordination/native-routed-provider-import-r1/source1.json`, SHA256 `03C0BB43CF0F4E53BFB6F0DB30BF3A03E75C989A83059F2173435D132E2695BF`.
- Leaf: `runtime/development/native-routed-provider-import-r1/src/infrastructure/catalog.rs`, SHA256 `A4630199374FAE29867A2D374CFC29F3C9097D0CB29AA4A2F8B88F9B5ACDE5FA`.
- Replaces the existing catalog leaf in a separately authorized coherent successor, adding no new supply path. Transition work is not included.
- Prepared coherent108 successor: `runtime/coordination/native-routed-provider-import-r1/coherent108-supply.json`, SHA256 `D4602107D4AFDECCC99227FB5F24FABB14FF9E7F8799E3CC1856D71149BCBCC5`; original UI12 manifest remains `2E4F2922A06616A5C359AEA24FC296EF1373B4A5CAE8B1EEFBA0B3077853D0D2`. Replaces only the catalog source entry in original coherent108 `1E82579A84E2A8B005592F733A0E6F351B2EA06256A6A0D01C84688107EA8459`.
- Exact original/fixed diff: `runtime/coordination/native-routed-provider-import-r1/import-only.diff`, SHA256 `EB5CABBD49A8C688716F05BE8E8D3E359BCBB586624F9D08BF5E39C495DE69BD`. Observed diff is the import addition and wrapping only; no function changes.

UI owns the failed build receipt/source restoration. PM controls re-entry; SDK did not retry or infer compilation success. Required fmt/type/test/strict acceptance remains pending.
