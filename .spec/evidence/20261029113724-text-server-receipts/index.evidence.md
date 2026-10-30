---
id: evidence:kinekino/20261029113724
title: 텍스트 추론 실행·종료 receipt 수집
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 텍스트 추론 실행·종료 receipt 수집

2026-10-19. 모델 품질이나 엔진 내부 완결을 대신하지 않는 외부 실행 관측이다. [초기화 추적 구현](generation-gemma270-startup-lifecycle-implementation.md)과 [실제 설정 근거 조사](../../player/infrastructure/inference/research-followups/gemma270-runtime-control-evidence.md)를 연결한다.

## 책임과 출력

`tools/model-spike/src/infrastructure/local_text_server/launch_configuration.rs`는 실제 Command의 program/args와 허용된 명시적 환경 override/removal만 기록한다. 전체 상속 환경이나 엔진의 적용 설정 조회가 아니다. MCP 제거는 제거 키만 기록하고 비밀 값을 노출하지 않는다. 잘못된 UTF-16은 손실 변환 대신 거부한다.

같은 디렉터리의 `owned_process.rs`는 생성한 Child와 전용 Job의 수명만 소유한다. 명시적 shutdown은 소유 Job을 닫고 실제 Child.wait 결과를 반환한다. `server-shutdown.json`은 PID·종료 방법·닫기 전 실행 여부·JobClose 성공·실제 종료 코드/성공 여부를 기록한다. 별도 사용자 프로세스나 ChatGPT/Terminal을 종료하지 않는다. Drop은 소유 자원의 미정리 경로만 처리한다.

`application/generation_trace.rs`는 명시적 종료 전후 raw SHA와 종료 후 다시 읽은 parseable/complete 상태를 구분한다. raw가 양쪽 모두 없으면 안정된 해시로 처리하지 않는다. 동일 SHA와 parseable/complete는 마지막 fsync ACK의 증명이 아니며, 종료 전 마지막 관측 필드도 원자적 스냅샷으로 주장하지 않는다.

## 검증과 고정

Rust 테스트 child의 준비 신호 뒤 소유 Job 종료·실제 wait 상태 수신을 확인했다. 최종 r3는 fmt·68 tests·all-targets strict Clippy·build exit0이다. [검사 원문](../../../evidence/verification/text-server-receipts-2026-10-19-r3/)은 최초 r1의 JSON 타입 불일치 컴파일 실패와 수정 후 기록을 구분해 보존한다.

고정 worker SHA256은 `01e7b614242f21613879002b3ac9659d015d96d23cb98e906fa44514338014fd`다. [build manifest](../../../evidence/ai/model-spike-builds/01e7b614242f21613879002b3ac9659d015d96d23cb98e906fa44514338014fd/build-manifest.json)는 source206개와 실행본1개를 보존한다. 이 고정본에 아직 생성되지 않은 새 엔진 EXE의 Revision을 포함했다고 주장하지 않는다.

## 남은 검증

실제 새 모델 서버에서 receipt 생성과 외부 worker 종료를 대조하고, SDK의 RunReceipt에 engine child 종료와 outer worker 종료를 각각 연결해야 한다. runtime scheduler/prefix/speculative 등의 적용값, 최종 flush ACK, trace 비간섭 증명은 별도 미완료다. 테스트 child 성공을 실제 모델 종료·제품 품질 성공으로 확대하지 않는다.
