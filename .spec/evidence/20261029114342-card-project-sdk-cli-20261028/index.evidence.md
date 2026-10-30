---
id: evidence:kinekino/20261029114342
title: 카드 프로젝트의 공통 SDK·정상 CLI 및 통합 회귀
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 카드 프로젝트의 공통 SDK·정상 CLI 및 통합 회귀

최종 1949 custody 종료0: current source content/identity unchanged files=2347. R104 고정 입력과 실제 시험/정상 bins 빌드 후 입력이 동일함을 확인했다.

2026-10-28 R102→R104 중앙 PM. [공통 계약](../../player/application/production/card-project-sdk-cli.md), [불변 저장](../../player/application/production/card-project-storage.md), [편집 문서](../../player/application/production/card-document.md). card_projects.create/inspect/save_revision을 PlayerSdk·object-safe ApplicationClient·schema/capabilities·정상 CLI의 명시 --card-project-root에 연결했다. 기본 카탈로그103, 별도 execution host10이다. 일반 production.export_cards는 계속 미지원이다.

## 입력과 첫 실패

R102 SOURCE.md 8EFF7643A490600F3B7BEA5A8A32C66508B0163D950A35ACC3B5BC2FC146ECBC, inventory1AC565A849050C2946EA2B2EDB3FF648959071554FAFF216CE91CA56E0CEAD56/2347. 1930 Native strict는 새 시험의 hash helper 누락(E0425)와 잘못된 receipt.exit_code(E0609)로 종료1이다. 원 preimage-r102-card-cli-test와 실패 로그를 보존했다. 수정은 child_exit 관측 및 공통 증거 helper의 요구 hash 함수 제공이다.

R103 SOURCE.md 3D75FF116BEE37248067560DA9A5145657FE6EA7D1803E6E1F4954A5AAF924C8, inventory FA60E30A2300A71400C3DD4FBD7B4A5DB8D4EF7F78A6857CBEEA74AF2D455561/2347. 1933 strict/1934 no-run 통과 후 1935 실제 시험은 두 번째 호출에서 CLI unsupported 종료3을 시험이1로 잘못 기대해 실패했다. 실제 CLI는 기존 계약대로 거절했다. preimage-r103-card-cli-exit·fixed-r103 exe/PDB·원 raw 증거를 보존했다. R104는 거절별 고정 exit contract(InvalidArgument2/Unsupported3/NotFound4/RevisionConflict5)로 시험만 수정했다. 성공만 검사하도록 약화하지 않았다.

R104 SOURCE.md SHA256 0883A7C1278B94926DB43F87AC8CBBA4AEC689A9EFE6F3D16EB54BD7D99832B6, inventory 14538F74EEB0A946478CC7597CB755CB675BD83FF675E8E96F6D7D7EC9A3BA47/2347. snapshot은 mirror/runtime/verification/input-r104다. AGENTS/lock/toolchain·C-only/cache/tmp·heavy1/CPU2/BelowNormal·offline/locked/-j1 유지. 도구의 canonical snapshot 일반 제목은 private 후보 채택이 아니다.

api mod/client/capabilities/runner 원 preimage는 preimage-r101-card-api에 있다. application/card_projects.rs 및 기존 발견/모델/역사 시험3개의 사본은 해당 단계 변경을 역적용해 재구성했으며 R101 source-current.tsv의 원 SHA와 각각 일치함을 확인했다(원 FileID 동일성은 주장하지 않는다). 세 시험의 변경은 기대 카탈로그 수100→103이며 명령별 schema/지원 상태 대조를 유지했다. 기존 root WIP/소유권을 덮어쓰거나 stage/commit/push하지 않았다.

## 현재 실제 실행

로그 루트 runtime/verification/source375-datalab-b0-r1-logs. 1938 Native all-target strict0, 1939 lib/jobs/card_project_cli/cli_discovery_inventory no-run0, 1940 카드 실제 선택시험1통과/8filtered/12.92초, 1941 기본 발견/schema 선택1통과/5filtered/22.34초, 1942 default all-target strict0, 1943 fmtcheck0. 이전1934는 실제 compile3분30초였고 R104 no-run은 동일 제품 소스/갱신 시험을 확인했다. transitive blitz-paint deprecated 경고는 별도 보존했다.

1940 증거는 runtime/verification/source-sdk-r104-card-project-cli-r1이다. summary는 실제CLI24호출과 typed_sdk_cli_equal=true를 기록했다. 기본 저장소 부재·root 구성별 schema/지원 상태·create·duplicate conflict·필수 expected 누락·save·stale save·과거/최신 재열기·미지원 context·외부 source·경로 이탈/추가 field/없는 revision 거절·마지막 head 불변을 확인했다. 별도 SDK root에서 Arc<dyn ApplicationClient>의 typed create/save/inspect가 CLI와 동일한 발행/원본/문서 데이터를 반환했다. 프로세스는 기존 owned_process_evidence의 원 child-family supervision·deadline/receipt 경계를 사용했으며 raw request/response/receipt와 저장 파일을 보존했다. 첫 R103 실패 증거는 별도 디렉터리에 그대로 남겼다.

고정 산출물은 runtime/build-cache/native/source-sdk-integration-r1/fixed-r104다. CLI SHA256 144E0C0C20E73A6DFFE33C50A6831ACD6934DA4C9B7ED1F40E7E95A7AC0221A3, card_project_cli-3349fa5f0507471b.exe 47C983DFFB9E2DA9ECA788140944E929E3565FBB9A1F9983873B3D403278BB5A, discovery exe 055FB3DDEF483A635F56D4348B754060F1ADCF349955F4D404AF08C201104037. PDB도 보존했다.

## 같은 입력의 지정 gate

1944 Product145/1945 Core18/1946 Native26/1947 OS1 모두 실제 실행·종료0이다. runtime/verification/source-sdk-r104-exact190-r1 각 group-return.txt의 execution_result=Ok(())와 producer_integrity=Ok(())를 확인했다. 각 source-receipt는 R104 SOURCE를 보존한다. Product producer는1939 JSON manifest, Meari는 기존622 Core/624 Native·OS producer를 재사용해 이번에 시험을 실행했다. Meari를 새로 컴파일했다고 주장하지 않는다. 원190 plan과 역사D명은 보존하며 실제 실행/cache는 C 경로다.

1948 정상 native-proposal-window player/player-cli 명시 build 종료0(캐시 재사용0.47초). fixed-r104/native-bins exe/PDB를 보존했다. GUI SHA256 5DCFD123DC3EE2F19082ED9308ACAB91A67457A70393CFBE72EB7C2BF1CC5A30, CLI는 위144E...와 같다. 빌드는 GUI 수동 실행/시각 인수가 아니다. 지정190·정상2bins의 R92 근거를 승계한 것이 아니라 이번 R104에서 실행한 근거다.

## 남은 범위

전체 CardDocument/Jobs/renderer/묶음 출력 인수는 미완료다. 공개API에서 저장은 지원하지만 영속 undo·취소/timeout·idempotent ACK 복구·disk/permission/crash fault 주입·독립 JSON Schema 엔진의 전수 동등성·실제 미디어 provenance 인증·사용자 AI 품질은 미완료/미지원이다. 현재 저장은 cue snapshot 초안을 저장하며 실제9를 승인/변경하지 않는다. 원9 NeedsRepair·사용자 품질0/9, 기존 WIP/소유권/private overlay·WARP/추론/다운로드/수동 GUI/push 보류 유지. 다음은 카드 묶음의 명시 template/frame/asset preflight와 기존 renderer/Jobs의 실제 산출물 연결이다.

적용 스킬: 기존에 읽은 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100), code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B). 첫 실패·raw 증거를 보존하고 발견/파일/실제CLI/전체 gate/사용자 품질의 범위를 구분했다.
