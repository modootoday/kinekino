---
id: evidence:kinekino/20261029114520
title: 모델 검증의 실행 중 취소 전달 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 모델 검증의 실행 중 취소 전달 검증

2026-10-27 중앙 PM. [계약](../../player/application/analysis/model-verification-execution-control.md). 기존 모델 검증의 직접 checkpoint와 SDK execution의 원 operation 취소를 연결했다. user/owner·WIP·원본 증거·보류를 유지하며 source375는 private overlay다.

## 입력과 변경

SOURCE9F448BA39ACE5510FD35D5D50E30AF41C11F7D454DF2ED0EDF07C9AE2101D584, inventoryA4CAB53A09A8BFEAD4ADBE34F1AA53519EF083ABCC573F025C5EEB98E209D799/2240을682에서 `runtime/verification/source375-model-cancel-input-r1`에 고정했다. 원7파일은 source375-model-cancel-preimage-r1에 보존했다. 새 Rust 시험은 sdk_execution/tests/cooperative_model.rs다. 로컬 source/원 git index를 한꺼번에 흡수하지 않는다.

ApplicationClient는 execute_controlled의 기본 구현에서 기존 execute를 호출한다. PlayerSdk만 model verification에 control을 전달한다. worker는 원 Shared/sequence의 cancelling만 읽고 leaf가 실제 반환한 응답을 보존한다. model deadline은 외부 checkpoint와 기존 Instant 기한을 함께 사용한다. 별도 timer/observer thread·store·job/attempt·전역 취소를 추가하지 않았다. CLI 원격 cancel ACK나 강제 선점으로 확대하지 않는다.

681은 최종 capability/모듈 주석 변경 전 기본 strict다. 최종 입력의683 Native all-targets strict와684 기본 workspace all-targets strict는 통과했다. 로그는 runtime/verification/source375-datalab-b0-r1-logs/681~에 보존한다. 성공한 이전 입력의 새 실행으로 계상하지 않는다.

## R1 결과와 fmt 첫 실패

685 no-run exit0/2분49초 후686 sdk_execution19/19(새2 포함)·687 model_catalog_cli4/4(18.89초,실제CLI115)·688 실제9 historical 확장1/1(19.94초,CLI92)가 R1에서 통과했다. 686 stderr의 controlled worker join failure panic은 기존 주입 시험의 기대 관측이며 suite exit0다. 새 시험은 원 operation이 모델 verifier에 진입한 뒤 취소됨·다음 version 요청 정상·사전 취소의 snapshot/files0을 확인했다. 기존 nonparticipating running cancel→원 성공 보존도 통과했다.

모델 fixture 원4개는 runtime/build-cache/tmp/model-catalog-cli-36276-{1793071901354507200,1793071901356661900,1793071901474764100,1793071919301618800}에 유지하고 model-cancel-model-cli-r1/{invalid,cancel-timeout,catalog,matched}에 복사했다. 실제9는 runtime/verification/actual9-model-cancel-regression-r1, input-pins SHA E71C3F2CD5F39C12E5D66B6CCB44091CA697CAC22D19F6048908F0A722CC4E9A다. 원 실제9/후보/사용자 판정은 불변이며 복사본 deferred만 기록했다.

689~692에서 R1 선택190/190을 직렬 실행했고693 정상 bins2종 profile.test=false/fresh=true도 통과했다. player SHA8EFC85FD15B7FAC9F456A068DBE02DA9D6FB756888EA19CDF967FD3B707ACA41, CLI SHA4EB2B74B42A62F27C7EE87A0D99F89C08F6A21E8A85DA14779794F267C2819B1다.

694 fmt --check는 tests.rs의 모듈 선언 순서(initial/cooperative_model) 한 곳 때문에 실패했다. 첫 로그를 보존하고 r1-execution-tests.rs preimage를 추가한 뒤 선언 순서만 바꿨다. 695 fmt 재검사는 exit0다. R1의 실제 CLI 결과를 새 입력 실행으로 승계하지 않으며696에서 별도 R2를 고정했다.

## R2 최종 검증

R2 SOURCE EAB91917F0452EAFADBD7A0AAC2E11EB08B007C39B834EA9D72A842D38035DC4, inventory804BB2079B332ED85FA2C832451A2B284B7AB9A84E5DDDA57B62D6486C87D42C/2240은 source375-model-cancel-input-r2다. 697 기본 strict/698 Native strict,699 no-run(18.00초),700 실행기19/19(0.04초)를 통과했다. R1과의 유일한 source 차이는 시험 모듈 선언 순서이며 제품 코드/실제 CLI bytes는 동일하다.

701 Product145/145,702 Core18/18,704 Native26/26,705 자동OS1/1을 직렬 실행해 최종 선택190/190을 통과했다. 703은 제가 Native에699 Product용 Cargo JSON을 지정해 missing expected original producer로 시험 실행 전에 차단됐다. 원703 로그/미실행을 보존하고704에서 정확한624 Native JSON을 사용했다. 새 Native 기능 실패로 계상하지 않는다. Product는699 새 no-run JSON, 변경 없는 독립 Meari는622/624 원 artifact를 재사용하여 실제 시험을 다시 실행했다.

706 정상 bins2종의 test=false/fresh=true와695 fmt 통과를 확인했다. R2의 CLI4EB2B74B42A62F27C7EE87A0D99F89C08F6A21E8A85DA14779794F267C2819B1/player8EFC85FD15B7FAC9F456A068DBE02DA9D6FB756888EA19CDF967FD3B707ACA41는 R1과 같다. R1의 실제CLI115/실제9 CLI92는 그 입력의 원 실행으로 유지하고 R2에서 반복했다고 주장하지 않는다. 이 두 실행과 R2 SDK19는 서로 다른 경계 근거이며 합쳐 전체 취소 지원으로 표시하지 않는다.

## 시험별 주장 경계

새 gated ModelCatalogPort는 verify_files 진입/원 execution Running을 관측한 뒤 원 handle.cancel을 호출하고 checkpoint의 Cancelled를 확인한다. 다음 request는 정상 실행되고 ACK/worker close까지 확인한다. 사전 취소 시험은 catalog snapshot/file verifier 호출0을 확인한다. RAII release 및 bounded wait를 두고 fixture 원 자원을 해제한다. 합성 port이며 실제 모델 bytes의 해시 읽기라고 주장하지 않는다.

실제 로컬 파일의64KiB/checkpoint·guard/slot 해제는 model_catalog_cli 시험의 별도 범위다. 두 근거를 구분하고 모델 엔진/품질·inference readiness로 승격하지 않는다. SDK execution 회귀의 nonparticipating client running cancel→실제 성공 유지·queued cancel·ACK/close도 함께 실행해 원 계약 보존을 확인했다. 별도 wrapper client나 endpoint가 controlled hook을 전달하지 않으면 기존 Requested/실제 response 경계를 유지한다.

modoo.today 검색의 PR-triage는 비적합으로 미채택했다. 앞서 읽은 test-fixture-and-mock-build 본문 SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100을 gated resource 획득/실패 시 release·관측 범위 구분에 적용했다. 성능 개선/실측을 주장하지 않으며 새 개발/시험은 Rust만 사용했다. GUI·추론·다운로드·WARP·push 보류는 유지한다.

최종707 verify-current는 exit0/40979ms로 R2 전체2240파일 content/identity 동일을 확인했다. root HEAD d13cbfca140e41863b4d0bd27930be3f57a1123f/Meari0844f6c24caf224617ca8abbb2dd75695a971b99, root lockB2E3D540665FFC8E68D24D0A6C531D2B983658FB19B5ABBA525659BA1ACC371D/Meari lock23792484F39356E19904DBBB34B1496979E87793CF035491EF71F7C4134CC1EE를 유지했다. cargo SHA AA5B8980BE3681899FDE765BA47E2ED361463AE9DD8EE7CD1963C18D23743A0F/rustc SHA BA03E3BA03A73D9FEF19AB3659140F94B43615EC1D587FE9BEFC9AED3BD5AA72도 기존 pin과 같다. 공유 index는 비어 있고 stage/commit/push·canonical 승격·추가 설치0이다. 모든 검증의 C NVMe 문맥·TMP/TEMP·jobs1/CPU2/BelowNormal·heavy1을 유지했다. 전체 CLI·원격 취소·실제 모델 IO/추론·GUI/사용자 품질은 미완료이므로 goal은 계속 진행 중이다.