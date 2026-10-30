---
id: sot:kinekino/vad-host-sdk-cli
title: 제품 VAD host의 공통 SDK·CLI 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 제품 VAD host의 공통 SDK·CLI 계약

기존 `--job-root`는 caller가 준비한 ordinary absolute directory를 연다. `jobs::start_retaining_host`→`JobFiles::open`→`NativePaths::root`→기존 `immutable_storage_paths::root`가 각 ancestor directory와 canonical path를 확인한다. 없는 root를 자동 생성하는 계약은 아니다. 이번 기본 EOF 시험의 NotFound는 해당 준비 누락이며 제품 동작은 변경하지 않는다. 기본 긍정 fixture에만 root를 만들고, 주소/설정 거부 시험은 ready·root 생성이 없음을 검사한다. 현재 gate 진행 근거는 `runtime/verification/vad-product-host-r1`에 기록하며 아래 source-only 표기는 동결 시점의 이력이다.

2026-10-23 source-only 후보다. 현재 채택 root b03975bc와 실제190은 아래 새 정상 module 경로의 실행 근거가 아니다. 후보/전체 preimage는 runtime/development/vad-product-host-r1/HANDOFF.md를 따른다.

후속 관측에서 root는 5ef988b6로 전진했다. 원 parent b039 대비 SDK 변경6과 후보21의 교집합0·기존 preimage13 일치·신규8 부재를 별도 root-advance.json에 기록했다. 원 parent와 후보를 덮거나 새 root를 이 후보의 타입/실행 근거로 사용하지 않는다.

공통 Rust SDK의 ProductVadHostOptions/Plan과 `infrastructure::vad_product_host::start_product_vad_jobs`를 통해 기존 retaining jobs host를 조립한다. CLI는 `--serve-jobs --job-root ABS --vad-root ABS_RELEASE --vad-workspace ABS_DATA`의 주소만 전달한다. 두 VAD 주소는 한 번씩 함께 지정하며 상대/중복/누락 주소를 거절한다. 기본 `--serve-jobs`는 기존 subtitle 경로를 유지한다. 기존 ready/endpoint/stdin EOF/원 dependency 종료 carrier를 재사용한다.

모델 권한은 주소와 분리한다. private Rust PRODUCT_RELEASE와 native record에서 고른 profile/license/worker/full manifest/dependency/notice/Windows 기대값을 실제 immutable file bytes와 대조한 뒤에만 ExecutorRegistry에 audio.vad를 등록한다. caller JSON·flag·자기 hash는 권한을 만들지 않는다. 현재 승인 record가 없어 NeedsInput으로 ready 전에 닫는다. record·설치 artifact 부족의 NeedsInput, 다른 실제 SHA/profile/artifact의 SourceExpired/RevisionConflict, 비 Windows transport/비 owned 실행의 Unsupported를 구별한다. 오류만 반환하는 상태를 최종 제품 지원 완료로 삼지 않는다.

workspace는 ordinary audio/outputs/cache/evidence 디렉터리를 요구한다. jobs.submit의 기존 opaque audio_artifact_id는 `audio/audio-ID.wav`, destination_id는 `outputs/result-ID.json`에 매핑한다. 원 source SHA는 그 실제 WAV bytes의 SHA다. 준비된 PCM16 mono16k WAV만 첫 수직 범위로 받으며 기존 공통 wav_pcm body를 그대로 재사용해 PCM/sample pin을 계산한다. 데이터 hash는 무결성 근거이지 모델/배포 승인이 아니다.

매 요청·재개 시 새 factory binding으로 다른 audio/destination을 공급한다. 이전 FixedVadInputs/GuardedReleasedVadInputs의 single mapping은 요청 내부 어댑터로만 사용하고 제품 전체를 단일 샘플로 제한하지 않는다. 실제 WAV 원 guard→기존 descriptor/CAS/readback→원 scope/input fingerprint 확인→기존 readmission→configured22 assembly로 이어진다. 제출 시에는 bounded 입력 준비만 하고 새 scope/deadline을 만들지 않는다. 실행 시 원 control/cutoff, checked JobId의 evidence, 원 owner/session/P0/결과 publication과 no-overwrite/cancel/retry 정책을 사용한다. RootJobExecutor::new3 및 legacy16 호출 형태는 그대로 두고, 정상 제품은 표준 TryFrom의 concrete Rust issuer/selection 결속을 통해 명시 선택한다.

조건부 정상 의존성·module 선언·issuer3의 실제 편입과 CLI help/typed adapter는 코드 후보에 있지만 Cargo/타입/실행은0이다. source21 `5EE51BAD305E9018746BAC5FCD692A9328FAB1EC57DD0D23D1FD9EBFB087399F`, before13 `65B3EDFD2D5DE645EDE033A916E5047CC2E85194B188148FE5BE3AA28D29D86D`, 직접fmt20·신규authored8이며 canonical root/S/T/index를 수정하지 않았다. SDK configured source6과 파일 교집합0이다.

modoo.today `modootoday/software-qa/code-change-review-verification` 본문9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B/참조9D9619A829AE24D73645EF5F7C8126C8257CAEDDAC80800632866257008AD2A9를 미마운트·single mapping의 원 callsite와 새 conditional caller 경로 판정에 적용했다. 실제 승인 worker/model의 CLI 처리와 [후속 활성화 계획](../../../../plans/player/inference/vad-product-host-activation.md)이 완료 기준이며 전체 AI/미디어 품질 범위는 유지한다.
