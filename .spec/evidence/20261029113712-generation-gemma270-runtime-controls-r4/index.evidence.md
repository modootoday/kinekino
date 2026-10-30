---
id: evidence:kinekino/20261029113712
title: Gemma R4 runtime 관측·flush 계약
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Gemma R4 runtime 관측·flush 계약

2026-10-19. [R3 실제 관측](generation-gemma270-startup-lifecycle-actual.md)에 없는 runtime controls와 마지막 flush 성공 순서를 별도 producer source로 추가한다. [진행 계획](../../../plans/gemma-runtime-controls-r4.md)의 구현·검증·pin 인계가 범위다. 새 모델 추론·다운로드는 실행하지 않는다.

후속 사용자 우선순위 변경에 따라 API/SDK/UI의 사용자 실측 준비가 끝나기 전에는 새 추론·다운로드·실측 예약과 cold worker compile을 연기한다. 이미 실행 중인 건강한 vendor 검사 묶음만 자연 완료·원문 보존한다. 아래 worker/assembler는 source·fmt·lock 단계의 정적 준비이며 실행·시험·pin 통과로 취급하지 않는다. 제품의 기존 실제9표본 수입·수정·채택을 이 후속 실험 때문에 대기시키지 않는다.

## 입력과 구현 범위

독립 source는 `runtime/ai-spike/source/mistral-runtime-controls-source-r4`다. [전체 source snapshot](../../../evidence/ai/engine-context-cap/gemma270-runtime-controls-source-r4/manifest.json) SHA256은 `2d91345c6767bc1070d2cbb939953e580434b816180dfaaca4075bbbb694fbe4`다. base commit과 고정 FEA overlay로 만든1,891파일을 R3와 대조한 뒤 기존10경로 변경·신규2경로를 허용해1,893파일을 보존했다. R3 source/엔진/worker/actual은 변경하지 않았다. 별도 target은 `runtime/ai-spike/targets/gemma-runtime-controls-r4`다.

raw journal schema3 본체의 필드·event는 유지한다. 별도 schema1 controls/commit/ACK 파일과 source pin을 추가하므로 SDK의 기존 FEA producer admission을 자동으로 통과하지 않는다. SDK 소유 세션과 새 source profile admission을 별도 검토한다.

## 실제 관측 지점

| 값 | 관측 근거 |
|---|---|
| scheduler variant/max sequences | 완성된 Default/Paged Scheduler의 실제 보유 method/config getter. 미지원 구현은None |
| prefix capacity/disabled/paged/acceptance | PrefixCacheManagerV2의 n_on_device/no_prefix_cache/has_paged_attention와 두 accepts getter 각각 보존. population0으로 capacity0을 추정하지 않음 |
| speculative | MultimodalPipeline의 실제 model.has_speculative_proposer getter. 다른 pipeline은None으로 미관측 |
| flavor/native capacity | 성공한 GGUF native Gemma3 text loader branch, 실제 Gemma3Config Text/WithVision 판별, 반환 pipeline 주소 귀속과 native config capacity |
| CPU/runtime capacity/kind/category | 실제 pipeline device·GeneralMetadata·category. prefix/source profile 이름으로 대신하지 않음 |
| offline | child 내 is_hf_hub_offline 실제 해석 getter. OS 네트워크 차단으로 확대하지 않음 |
| configured threads4 | 별도 Command 명시 env override. 실제 pool/OS 전체 thread cap으로 표현하지 않음 |

engine 생성 후와 실제 NormalRequest dispatch 전·후에 pipeline/scheduler/prefix lock 안에서는 owned 값만 복사한다. lock을 놓은 뒤 controls 파일을 create_new하고 sync한다. startup1+cohort3의 고정 실행은9개 snapshot을 예상하며 collector는 순서·session·설정 불변을 대조한다. 실제 pool getter는 초기화를 유발하지 않는 관측이 확보되지 않아null/unavailable로 남긴다.

## flush commit과 ACK

raw write/truncate/sync_all이 반환된 완결 snapshot에 한해 flush_revision/raw SHA·bytes/last ordinal·run/session/engine/model/source와 마지막 controls 파일 SHA를 immutable commit에 쓴다. commit sync_all 성공 후 별도 create_new ACK를 작성·sync한다. revision/SHA별 파일이므로 이전 commit/ACK를 덮지 않는다. 더 오래된 snapshot을 건너뛴 Sink는 그 revision의 ACK를 만들지 않는다.

ACK에는 commit 파일과 SHA, raw SHA·bytes·revision·ordinal, commit_sync_returned만 기록한다. ACK 자신의 fsync 성공을 ACK 자기 필드로 증명하지 않는다. raw의 성공 자기선언과 구별되는 성공 후 순서 증거이며 독립 OS·전원 손실 내구성 보증은 아니다. worker의 실제 readback·원본 bytes hash·owned JobClose+wait 및 outer worker의 실제 종료 측정과 결합한다. 실패·누락·다른 run/hash·필수값null은 거부하고 보존한 원문을 임의 완성하지 않는다.

## 별도 worker와 SDK 인계

`tools/runtime-controls-spike`는 R3 worker를 수정하지 않는 별도 Rust 도구다. R3 f763 snapshot의 owned process/launch/process measurement 어댑터를 복사해 사용하며 launch allowlist에는 R4 source SHA 설정을 명시한다. 자체 compiler input·EXE를 별도 보존한다. 엔진 pin은 최종 CLI build와 archive 이후에만 frozen으로 전환하며 pending 상태는 실행을 거부한다.

collector는 최종 raw에 일치하는 commit/ACK와 controls9개, raw에서 실제 해석된 cohort n_choices를 대조한다. assembler는 실제 launch overrides/removal·child exit와 supervisor의 실제 outer exit를 받은 뒤 RunReceipt v1·정확한6 evidence role·외부 고정 fixture의 CasePin·TrustedPins·입력 경로 매핑을 새 디렉토리에 작성한다. configured4와 실제 pool 미관측은 구별한다. SDK의 새 source/환경 필드 admission과 full join 본체 실행은 별도 인수다. capture_authenticated/noninterference/full compiler/품질은 자동 승격하지 않는다.

## 현재 관측 상태

기존 Rust engine-lab bundle은 자연terminal exit0이다. core+CLI all-targets locked offline check는exit0(1075.5961865초), core test는exit0(컴파일 포함3911.7665369초)/1388passed/0failed/5ignored(실제 test7.98초), core+CLI all-target strict Clippy는exit0(269.0070896초), 최종 CLI build는exit0(3694.8416091초)이다. 네 단계 뒤 source1,893파일·외부 fork53파일과 source manifest pin 일치를 재검증했다. 컴파일 소요 시간은 모델 추론 성능이 아니다.

[최종 엔진 archive](../../../evidence/ai/engine-cli-builds/gemma270-runtime-controls-cli-r4/manifest.json)의 EXE는99,183,104B/SHA256 `ec3d9371632dc7b134c3655432a6fe585463dd2b84b768e7a59b8412af815156`이며 원본/복사본의 native SHA가 Rust gate result와 같다. gate/ 아래 원문21receipt, outer log와 copied-file hash inventory를 보존했다. static archive manifest SHA는 `b92b49feb9e8228f170fe0ef9e4e82802d5513a7e39a8832839c4f53aa7fc630`이다. 새 cold worker build를 피하기 위해 native Copy-Item/Get-FileHash와 정적 메타데이터로 보존했으며 deferred worker의 archive-engine 명령을 실행한 결과가 아니다. 해당 명령의 flat files manifest와 동일 profile로 가정하지 않고 후속 consumer admission 시 보존 형태를 명시 검토한다. 기존 archive 경로를 덮어쓰거나 명령을 재실행하지 않는다.

worker pin은 pending 그대로이고 worker test/Clippy/build·실제 R4 모델 실행·shared strictjoin은 아직 없다. capture/noninterference/품질 승인도 없다. 새worker·모델실측은 CLI 코어 준비와 중앙 자원 배정 뒤로 유보한다. 모델 기능/품질은 실제 Rust CLI/SDK 고정 입출력·근거로 판정하고 GUI는 표현/비교/접근성 UI/UX 검토로 구분한다. R3 full CLI의 기존 실패는 입력artifact 미공급에 따른 artifact-open 실패이며 strictjoin 본체의 실제 거부 검증이 아니다.

## 보존 source의 문서 링크 검사

[검증 원문](../../../evidence/verification/gemma-runtime-controls-links-2026-10-19-r4/)에서 spec-maintenance fmt·13tests·strict Clippy·build·실제 check-links가 모두exit0이다. 검사746문서/canonical broken0/preserved upstream4이며 source 전체를 예외 처리하지 않았다. Python fenced code에 있는 두 함수 호출 표현은 링크가 아니므로 fence 종류/길이/닫힘을 고려해 오탐을 제거했다. R4 source의 MCP README 한 건은 정확한 경로·target·원문SHA `8f70a8c365fdbb31a3742e0eef18b37af2921066f704568228c78c6856329394`와 source snapshot SHA를 대조해 보존된 upstream broken link로 분류한다. 고정 Git base의 `docs/MCP/README.md` 부재도 확인했으며 원본은 수정하지 않았다.

이 링크 검사 EXE는5,069,312 bytes/SHA256 `6a2fd4e9f6e028959a3d0bd538bb00abe5f09cd57573a593c8b2fcc080d2e391`이다. 모델·UI·schema3 strictjoin·문서 내용 품질의 통과로 확대하지 않는다.
