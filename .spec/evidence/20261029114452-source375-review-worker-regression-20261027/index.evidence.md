---
id: evidence:kinekino/20261029114452
title: 로컬 review worker 후속 선택190·정상 bins
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 로컬 review worker 후속 선택190·정상 bins

최종 확정: 268은 terminal=true/exit0/43914ms로 전후2172파일 content/identity 동일성을 통과했다. 같은 SOURCE F7A32EFE…에서 선택190·정상 bins2·정상 CLI3시험과 입력 전후 보호를 완료했다. 전체 제품 인수와 goal 완료는 아니다.

2026-10-27 중앙 PM. 현재 고정 입력의 실제 회귀를 수행한다. 원 source375 private overlay·첫 실패·역사 canonical 검증을 새 호스트 성공으로 복사하지 않는다.

입력은 runtime/verification/source375-review-worker-input-r3/SOURCE.md SHA F7A32EFEE0AA82341B3A63687F4A3AAD58F32C6A2ABD34539D024C7300442C54, source-current.tsv SHA A7FE68409362842B83ECCBFD3127B1ECFCC305D49789059F4490E643C0F7847E /2172파일이다. 직전258에서 전체 content/FileID/bytes/creation/mtime 동일성을 확인했고 이후 소스 변경을 중단했다. [실제 worker 검증](review-local-worker-20261027.md)의 첫 실패247/252·후속257 성공과 구별한다.

원 exact190 계획 SHA082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464, custody 도구78DD39F4… 유지. source-current는 기존 원375 manifest를 대체하지 않는다. Root HEADd13cbfca…/Meari0844f6c2…, root lockB2E3D540…/Meari lock23792484…이며 WIP·소유권을 유지한다.

| 단계 | 현재 실제 결과 |
|---|---|
|259 제품 lib/jobs no-run|native-proposal-window, 종료0|
|260 Product exact|145/145, 등록·ignored·producer 무결성 Ok, SOURCE F7A32EFE…|
|261 독립 Meari Core no-run|meari lib/range_commit, 종료0|
|262 Core exact|18/18, 등록·ignored·producer 무결성 Ok, SOURCE F7A32EFE…|
|263 Native no-run|window/lib/range_input/source_callbacks/window_user_close/window_user_close_os 종료0|
|264 Native exact|26/26, 등록·ignored·producer 무결성 Ok,9224ms|
|265 OS exact|1/1, 등록·ignored·producer 무결성 Ok,2221ms|
|266 정상 player/player-cli|두 bin profile.test=false/fresh=true, 종료0|
|267 정상 CLI|저장/replay/CAS·재열기·readonly·combined3시험 통과, actual23.44초/outer35224ms|
|268 전체 입력 후속 감사|2172파일 content/FileID/bytes/creation/mtime 동일, 종료0/43914ms|

네 exact 그룹 모두 source_receipt_sha256=F7A32EFEE0AA82341B3A63687F4A3AAD58F32C6A2ABD34539D024C7300442C54다. Product260은19975ms,Core262는2446ms로 종료0이며 모두 원 producer 및 정확 등록/ignored 검사를 통과했다. 전체 선택190은 현재 입력으로 실제 실행한 결과다.

정상 player SHA C3B05223E0CEF336217F7C8EC5614D236E501F76BBEF4485380A0F831C64E5DE, player-cli SHA 0B527048F1A03822A6F8F674FA07353F1C7ED8FFFD47F7917C13964DF07858BA다. CLI는 직전 실제 headless seek257의 원 witness와 같고267 후에도 동일했다. GUI는 실행하지 않았다. 기존 dependency blitz-paint deprecated warning은 별도 담당 WIP로 유지했다.

모든 실행은 runtime/verification/source375-datalab-b0-r1-logs의 새 번호별 stdout/stderr/원 receipt를 보유한다. 네 exact 그룹은 runtime/verification/source375-review-worker-{product,core,native,os}-exact-r1에 별도로 발행한다. Rust 검증기는 Cargo JSON의 원 EXE/PDB를 선택해 실제 등록과 ignored 상태를 검사하고 한 프로세스에서 하나의 정확 시험만 실행하며 원 producer를 전후 비교한다.

product/native, 독립 Meari/meari-native, TMP/TEMP/tmp C cache·locked/offline/jobs1/CPU2/BelowNormal·heavy1/producer 독점을 유지한다. 전역 Cargo/Rustup·원 D 역사 정책/manifest/FileID를 변경하지 않았다. 수동 GUI·추론·다운로드·WARP 변경·push는0이며 OS 명명 시험도 기존 명시된 자동 검증 범위다.

기존 modoo.today build-cache-correctness-audit/shared-checkout/test-fixture-and-mock-build/code-change-review-verification의 원 producer·실제 등록·부분 실행/전체 인수 분리를 적용했다. 정확 출처는 [이전 통합 정본](source375-timeline-canonical-20261026.md) 및 연결 근거를 유지한다. 선택190은 전체 제품·CLI93·AI9·GUI 인수의 분모가 아니며 goal은 남은 public SDK/CLI/Verified/executor/GUI를 위해 계속 활성 상태다.
