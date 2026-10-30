---
id: evidence:kinekino/20261029114623
title: 봉인 Verified 후계의 선택190·정상 bins 회귀 — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 봉인 Verified 후계의 선택190·정상 bins 회귀 — 2026-10-27

중앙 PM. 원 source375 private overlay와 역사 성공·실패를 보존하고 현재 canonical WIP 입력의 실행 근거를 별도로 발행한다. 전체 제품·AI 품질·GUI 완료가 아니다.

## 고정 입력

runtime/verification/source375-sealed-verified-input-r1/SOURCE.md SHA256=6381DE6FB1AFE12E613D5129184BC2E53DDDDA0CE527D0F432B6E2E5BF7A5633, inventory256C8CC0C128B10C3CFD372C016F98EDC1F55B42F3C6F8CD12DA50A7CF2FC8CE/2176파일. 원 exact190 계획082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464와 custody 도구78DD39F4…를 유지한다. root HEADd13cbfca…/Meari0844f6c2…, root lockB2E3D540…/Meari lock23792484…를 재확인했다.

[실제 봉인 Verified](review-sealed-verified-20261027.md)의296 no-run/297 strict·fmt/298 actual1과 같은 입력이다. snapshot 이후 소스 편집 없이 다음 순서를 진행했다. product/native·독립 Meari/meari-native·tmp C 캐시를 구분하고 heavy1/jobs1/CPU2/BelowNormal/locked/offline을 유지했다.

| 단계 | 실제 결과 |
|---|---|
|299 Product lib/jobs no-run|native-proposal-window, 종료0|
|300 Product exact|145/145, 등록·ignored 확인, 종료0|
|301 독립 Meari Core no-run|lib/range_commit, 종료0|
|302 Core exact|18/18, 등록·ignored 확인, 종료0|
|303 Native no-run|window/lib/range_input/source_callbacks/window_user_close/window_user_close_os, 종료0|
|304 Native exact|26/26, 등록·ignored 확인, 종료0|
|305 OS exact|1/1, 등록·ignored 확인, 종료0|
|306 정상 bins|player/player-cli, native-proposal-window, 두 bin profile.test=false/fresh=true, 종료0|
|307 정상 CLI|저장/replay/CAS·readonly·combined·evidence 미주입 거절3시험 통과, 종료0/33860ms|
|308 전체 입력 후속 감사|2176파일 content/identity 동일, 종료0/40107ms|

정확 시험은 runtime/verification/source375-sealed-verified-{product,core,native,os}-exact-r1에 보존한다. 원 Cargo JSON에서 선택한 EXE/PDB의 전후 무결성과 실제 등록/ignored를 검증하고 각 시험은 원 producer로 별도 process에서 실행했다. 모든 그룹에 같은 SOURCE.md를 제공했다. stdout/stderr/원 launcher receipt는 source375-datalab-b0-r1-logs의 각 번호가 소유한다.

네 group-return.txt에서 execution_result=Ok(())·producer_integrity=Ok(()) 및 source_receipt_sha256=6381DE6F…를 직접 확인했다. 307 실제 normal CLI 근거는 runtime/build-cache/tmp/review-source-cli-32012-1이다. 전체 입력 감사까지 모두 완료했으며 이후 소스 변경은 없다. 정상 CLI3시험은22.78초였다.

정상 player SHA256=EB5C130A89E64AD6BECB0972CD6D8E1E2F200078DF87DB1697EED8826CBDCE1C, player-cli SHA256=D4ADAC58FABE1D24488332A272BC879ADFD9CDCFE6CBB69B80436FEF0C9D959B. GUI는 실행하지 않았다. dependency blitz-paint의 기존 deprecated 경고는 소유권을 유지해 별도로 남긴다.

modoo.today build-cache-correctness-audit/shared-checkout/test-fixture-and-mock-build/code-change-review-verification의 producer·실제 등록·부분/전체 인수 구분을 적용한다. 원 실패287/288 및 과거 원375 실패·소스/FileID를 소급 교체하지 않는다. 수동 GUI·추론·다운로드·WARP·push 보류를 유지한다. 남은 범위는 public source-bound 재생 open/close·frame 실제 관측·Verified 정상CLI 성공 경로·잔여 executor/CLI 인수·AI/GUI다.
