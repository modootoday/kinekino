---
id: evidence:kinekino/20261029114625
title: SOURCE DFB0 통합 회귀 — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SOURCE DFB0 통합 회귀 — 2026-10-27

여러 시험 전용 후속 변경을 반영한 현재 입력에서 fmt/전체 strict/no-run·등록190·정상bins를 직렬 재검증했다. source375는 private overlay이며 snapshot 도구의 generic canonical 제목은 적용 승인이나 canonical 검증이라는 뜻이 아니다. 전체 제품/CLI·AI 품질·GUI 시각 인수와 구분한다.

## 고정 입력·도구·환경

- SOURCE `runtime/verification/source375-episode-archive-input-r1/SOURCE.md`: SHA256 DFB0EB9021A8ED965D2E9FCBF43DF08B63C0C0DC74BE04EC369458C9171F150D. inventory ABD3A6013D739B0AB5109E446433785CAA8B2176BAD4146017B833EF4C436A6D,2244파일. 각 preimage·원 입력/실패는 앞 기능별 정본이 소유한다.
- root lock B2E3D540665FFC8E68D24D0A6C531D2B983658FB19B5ABBA525659BA1ACC371D, Meari lock23792484F39356E19904DBBB34B1496979E87793CF035491EF71F7C4134CC1EE.
- stable MSVC cargo SHA AA5B8980BE3681899FDE765BA47E2ED361463AE9DD8EE7CD1963C18D23743A0F, rustc BA03E3BA03A73D9FEF19AB3659140F94B43615EC1D587FE9BEFC9AED3BD5AA72. custody78DD39F4DBB8C47CD0242B43A7E766CC92CD1DB17087F640E2BB64B6D94672D9.
- Pretendard Regular6D0AF5258997AEC7354A6E340FC2325BA321C410CA48B3AF858C8C3D6E92A324, SemiBold5E1C548732AF70873103066C16E1369B9A8A871F0B38C321A1D5BC73E43CEA2D, JP RegularB14698ED6D82737F45E918D079BCA6FC498A107BCB094B56C5229DBDDF3246D3. font/license 원본 보존.
- C NVMe의 native/meari-native/resource-runner/tmp 문맥 분리, 프로젝트 Cargo jobs1, 실행별 TMP/TEMP/CARGO_TARGET_DIR 고정. global Cargo/Rustup 변경 없음. resource-runner conservative2/CPU2/BelowNormal, 무거운 작업1.

## 실제 gate

800 검증은 exit0/39386ms로 종료했다. 전체2244 입력 content/identity 동일을 확인했으며 아래 진행 중은 종료 전 이력이다.

원 stdout/stderr/receipt는 `runtime/verification/source375-datalab-b0-r1-logs/791~800-integrated-dfb0-*`다.

| gate | 결과 |
|---|---|
| 791 cargo fmt --check | exit0 |
| 792 기본 clippy --all-targets -- -D warnings | exit0 |
| 793 Native clippy --all-targets -- -D warnings | exit0; Meari paint의 기존 dependency deprecated 경고 보존 |
| 794 Native lib/jobs no-run | exit0, 원 Cargo JSON producer 사용 |
| 795 Product | 145/145 |
| 796 Meari Core | 18/18 |
| 797 Meari Native | 26/26 |
| 798 Meari OS | 1/1, 기존 자동 OS 시험이며 수동 GUI 아님 |
| 799 정상bins | player/player-cli profile.test=false·fresh=true,exit0 |

등록 계획은 `runtime/verification/product-meari-os-joint375-d-r5-gate/00-exact190.json` SHA082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464다. exact 증거는 `runtime/verification/integrated-dfb0-exact190-r1/{product,core,native,os}`에 있다. 실제 등록/ignored 상태, producer 해시·파일 식별자 전후와 각 one-pass 결과를 custody 도구가 확인했다. 각 group-return execution_result/producer_integrity=Ok, SOURCE DFB0 귀속이다. 이것은 지정190 전체이며 Cargo 모든 등록 시험의 실행을 뜻하지 않는다. 특히 durable jobs 실제종단·range·archive 시험은 별도 기능 근거로 유지한다.

Meari는 원622 Core/624 Native Cargo JSON의 실제 producer를 재해시·식별 검증해 재사용했고 각 시험은 이번에 다시 실행했다. 새로 컴파일했다는 주장이나 과거 성공 재사용이 아니다. 정상player SHA82DF7DC5B0C403C86D06297D3F2787FD0CAC53B2CEEFE366492CA61865A49269, CLI1DF8ADD0F6210B4E7C38CEC46943D0EA6647530086902DCB752752D4C735378D.

800 전체 입력 전후 동일 검증은 진행 중이다. 이번 gate의 첫 실패는 없으며 과거 실패743/751/765와 원source375의 실패/미실행은 덮어쓰지 않는다. 원본/WIP·소유권·AI 품질0/9·전체CLI/GUI 미완료·WARP/추론/다운로드/수동GUI/push 보류 유지.

기존 읽은 modoo.today code-change-review-verification 본문9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B와 verdict-table9D9619A829AE24D73645EF5F7C8126C8257CAEDDAC80800632866257008AD2A9의 주장/원 실행/미실행 분리를 적용했다. 전체 제품 완료로 goal을 닫지 않는다.
