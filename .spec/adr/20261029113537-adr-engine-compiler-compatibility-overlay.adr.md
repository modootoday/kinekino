---
id: adr:kinekino/20261029113537
title: 고정 추론 엔진의 컴파일러 호환 overlay
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# 고정 추론 엔진의 컴파일러 호환 overlay

## 배경

Gemma270 runtime2048을 유지하는 Rust 엔진 수정은 고정 mistral.rs v0.9.4 소스에서 검증한다. Windows Rust1.99 strict clippy가 기존 macro 생성 attribute·플랫폼 조건·생성자에서 실패했다.

## 결정

고정 archive에서 별도 async-trait0.1.89 fork를 추출하고 must_use 진단 이유만 명시한다. Cargo.toml path patch와 lock을 보존한다. Unix 전용 상수의 조건과 private XLora 생성자를 같은 동작의 Rust 코드로 정리한다. 모델 context 제한·시스템 cache·기존 설치 EXE는 유지한다. 소스 취득·검증·보존은 Rust 도구와 명시적 외부 Git/Cargo 어댑터가 담당한다.

원래 clippy all-targets -D warnings를 다시 실행한다. no-deps·allow를 대체 통과 근거로 사용하지 않는다. 이전 실패·cap-only patch·최종 호환 patch·fork metadata/license·성공 로그를 구별한다. 준비 manifest 상태를 후속 성공으로 다시 쓰지 않는다.

## 결과와 한계

[호환 검증 정본](../../sots/verification/ai/generation-gemma270-compiler-compatibility.md)에 strict clippy와 최종 check 성공을 기록했다. 실험 엔진에만 적용하며 제품 기본 엔진·배포·모델 품질 승인이 아니다. 다른 CPU 호환성, 매크로 전체 upstream UI test, 전체 transitive/native attestation은 별도다.

상위 버전이 수정하면 동일 명령과 실제 입력으로 검증한 뒤 overlay를 제거한다. 제거 전 fork·lock·EXE 증거를 보존하고 미검증 버전으로 자동 교체하지 않는다. 실패 시 기존 설치 엔진·모델 파일은 유지된다.
