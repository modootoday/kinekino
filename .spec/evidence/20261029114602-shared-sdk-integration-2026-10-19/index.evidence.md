---
id: evidence:kinekino/20261029114602
title: 공통 SDK와 실제 CLI 통합 재검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공통 SDK와 실제 CLI 통합 재검증

2026-10-19 현재 작업 소스로 전체 회귀94개와 CLI 실행 파일을 먼저 고정한 capability 전수 대조2개가 통과했다. GUI 네이티브 상태 provider 차이는 계약에 맞으며, AI 스파이크 결과를 제품 SDK 지원으로 승격하지 않는다. 실제 GUI 재조작·미디어 출력·새 모델 추론은 이번 시험에 포함하지 않았다.

## 실제 검증 범위

| 구성 | 지원 수 | 근거와 제한 |
|---|---:|---|
| 기본 SDK | 3 | version·capabilities·schema, 기본 구성 회귀 |
| 파일·FFmpeg adapter를 가진 실제 CLI와 동일 SDK | 5 | 위3개 + subtitles.convert·production.capture_frame. 이 호스트의 dependency 가용 조건 |
| GUI와 같은 상태 reader를 추가한 SDK | 6 | 같은 adapter 구성에 playback.status만 추가. 실제 GUI 화면을 통한 capability 조회로 표현하지 않음 |
| CLI의 알려진 나머지 명령 | 47 | 52개 전체 schema와47개 unsupported 응답을 실제 subprocess/SDK 사이에 대조 |

현재 main은 native_status_channel의 reader를 공통 PlayerSdk에, publisher를 presentation lifecycle에 조립한다. presentation의 update_native는 관측값을 publish하고 native 교체/종료 경로는 clear를 호출한다. 상태 DTO 변환·미관측 처리 정책은 application에 있으며, CLI는 별도 GUI host IPC가 없어 playback.status를 unsupported로 반환한다. GUI provider가 있는 SDK와 CLI 구성의 capability 차이가 이 명령 하나뿐임을 회귀로 확인했다. source에 clear가 있다는 사실을 정상 stop·source 교체 경합의 새 실기 증거로 사용하지 않는다. 실제 paused/엔진 종료 clear 관측은 [기존 r53 검토를 연결한 조립 정본](native-status-assembly-2026-10-18.md)을 따른다.

## 명령과 결과

`cargo test --locked --offline --target-dir runtime/parallel-build/sdk -j 1`은 exit0,94passed/0failed/8ignored였다. library71·CLI2·CLI contract4·요청 identity1·frame2·playback status3·capability parity2·subtitle9다. 외부 frame/media6개·원본 자막1개는 이번에 실행하지 않았고, 나머지 ignored1개는 deadline 시험이 내부 호출하는 subprocess fixture다. build3분36초를 시험 수행 시간과 합치지 않는다.

원시 요청을 보존하기 위해 `SDK_CAPABILITY_EVIDENCE_DIR`을 새 절대 경로로 지정하고 `cargo test --locked --offline --target-dir runtime/parallel-build/sdk --test sdk_capability_parity -j 1 -- --test-threads=1`만 다시 실행했다.2passed/0failed,20.05초다. 시험은 player-cli.exe를 폴더에 복사하고 원본과 byte 일치를 확인한 뒤 복사본으로 discovery·52schema·47unsupported 요청을 수행했다. GUI provider 차분 시험도 함께 통과했다.94개와2개는 같은 capability 시험의 재실행이므로96개의 서로 다른 시험으로 합산하지 않는다.

root `cargo fmt --check`와 같은 SDK target의 `cargo clippy --locked --offline --all-targets -j 1 -- -D warnings`도 exit0였다. clippy1분00초다. GUI를 띄우거나 VM·DPI·Pages·새 장시간 시험을 수행하지 않았다.

## 증거와 재현 한계

[봉인 manifest](../../../evidence/player/sdk-capability-parity-2026-10-19-r1/seal-manifest.json) SHA256은 `876a5757e8a2d97c35b27f4a526ff7401c51f93050fb228ab0a70899cd641235`, 요청 전 고정 CLI SHA256은 `aced852b962b874b79e318d9fa8536cfb9ac3b5f0b8ba9ea6984f91b6eeea0f8`이다. 같은 폴더에 discovery·각 schema/unsupported 요청·양쪽 응답·종료 코드·요약·원문 시험/fmt/clippy 로그와 검토한 소스/Cargo 사본을 보존했다. 전체 회귀 stdout SHA256은 `589dfb593e5663a1cf4728cc026193042536f12fb8b630c64db0d151eaceb590`다.

Rust model-spike 도구에 별도 cohort를 봉인하는 명령을 추가했다. 이전 고정 봉인과 frame 이미지는 재사용/덮어쓰기하지 않았다. 도구는 fmt·61tests·strict clippy·locked offline build를 통과했다. 실제 봉인 전에 [컴파일 입력198개와 EXE1개](../../../evidence/ai/model-spike-builds/2b4bea52d58cc75d4ad60c678a9cff6840bf8a93eef38581cf3ee0fc281318bd/build-manifest.json)를 선보존했다. 이 source/EXE 대응은 봉인 도구의 것이며 제품 CLI 전체 compiler input attestation이 아니다. 제품 소스 사본은 시험 후 검토 snapshot이며 해당 CLI의 전체 컴파일 입력으로 소급 주장하지 않는다.

## 남은 통합

중앙 Rust 검사기는 위 고정 seal과117개 저장 파일의 크기·SHA·경로 범위·중복을 실제 검사했다.9tests·fmt·strict clippy·release build도 통과했다. [중앙 감사 보고서](../../../evidence/verification/shared-sdk-integration-2026-10-19-r1/build-evidence.json) SHA256은 `e4f5a7657271386a1cc76893e6093da092687c401187fac5392f855e65a1abd5`, 검사EXE SHA256은 `97e473bc80155420022140c2553274ec4043bfd21da04f74bfc108245652ec57`이다.369문서의 정본 broken0·별도 보존상위참조3개를 확인했다. 같은 감사 폴더의 검사기 소스는 감사 시점 사본이며 compiler attestation이 아니다.

GUI 현재 frame 캡처 연결과 공유 playback 제어 포트, CLI host IPC, AI jobs·모델 registry·프로젝트 저장·취소/재시작은 미완료다. 정상 stop·source 변경 경합과 playing DTO의 실제 관측도 별도다. CLI와 GUI의 capability가 모두 같아야 한다는 이유로 미제공 adapter를 지원으로 표시하지 않는다. AI 품질은 [별도 게이트](../../../plans/player/ai-quality-gates.md), UI 정밀 검토는 [시각 후속 계획](../../../plans/player/presentation/visual-fidelity-follow-up.md)을 유지한다.
