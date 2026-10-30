---
id: evidence:kinekino/20261029114303
title: 실행 파일 고정 GUI CLI 관측의 중앙 검사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실행 파일 고정 GUI CLI 관측의 중앙 검사

2026-10-19 05시 KST. [r145 실제 통합 시험](native-gui-pinned-execution-2026-10-19.md)의 고정 V2 봉인을 중앙 Rust 검사기로 독립 재검사했다. `check-native-pinned`는 exit0이며 파일96개, 실행 파일 고정 기록4개, 실제 copied CLI 자식 이미지 기록21개와 저장된 복원 결과를 확인했다. 새 GUI 실행이나 모델 추론은 수행하지 않았다.

봉인 manifest SHA256은 `89205291bc7f6609f223ef806792fc6fa8913474da9cd179f1bc183ce6915cc7`다. 검사 결과 stdout SHA256은 `27f3c364690419e2d98fef080889a107c54a8ae09df23232fa8dade7e07953e7`, 사용한 검사 EXE SHA256은 `a1fc7989b08a4af27a05e912cd7d9911b104b9807f61560108b0f8f50adcef46`다.

[검사 결과·EXE·소스 스냅샷·게이트 로그](../../../evidence/verification/native-pinned-central-2026-10-19-r1/)를 보존했다. 도구의 fmt 검사, 테스트9개, all-targets Clippy `-D warnings`, locked offline debug build가 모두 통과했다. 소스 스냅샷은 감사 시점의 소스이며 전체 전이 의존성과 native compiler 입력에 대한 attestation은 아니다.

## 검사 범위와 잔여 작업

이 명령은 전체 검사와 같은 r145 검증 함수를 사용하지만 전체 문서 링크·나머지 AI 증거 검사를 대체하지 않는다. GUI/harness의 loaded memory와 compiler input은 증명하지 않는다. 과거 r137의 실행 파일 해시 미기록 한계는 그대로 유지한다.

기존 release 검사기의 전체 검사 시도 `gemma-internal-trace-integrity-r2`는 04시59분 무렵 exit1과 `지정된 경로를 찾을 수 없습니다. (os error 3)`로 종료했다. 해당 오류에는 대상 경로가 없어 원인 경로는 아직 확인하지 못했다. 이 전체 검사 시도를 성공으로 기록하지 않는다. 정확한 경로 진단과 전체 링크·해시 재검사는 남아 있다.

```text
tools/spec-maintenance/target/debug/player-spec-maintenance.exe check-native-pinned
```
