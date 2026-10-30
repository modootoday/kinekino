---
id: adr:kinekino/20261029113512
title: "ADR: 공개 소스 디렉터리와 비공개 근거의 경계"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# ADR: 공개 소스 디렉터리와 비공개 근거의 경계

상태: 2026-10-20 사용자 지시 반영. 사용자는 맥락·단계별 Git 커밋, 최소 공개면, 파일별 예외가 장황해지지 않는 디렉터리 구조를 요구했다.

## 배경

기존 working tree는 검증된 소스도 대부분 미추적이었다. 기본 거부 ignore에 개별 파일 예외를 늘리면서 필수 UI 선언·plugin source/tests·자산이 공개 입력에서 빠졌고, private `.spec`의 문서/실제 자료를 컴파일 입력으로 사용했다. 로컬 검사 성공은 공개 소스 재현성을 보장하지 않았다.

## 결정

공개 전용 src/tests/계약·필수 자산은 책임별 디렉터리로 허용한다. runtime/model/cache/개인 미디어/키/운영 스크립트와 로컬 `.spec`/`.agents`는 분리해 제외한다. 기존 exact 목록은 인수 입력 inventory로 유지하며 신규 파일마다 사용자 승인을 요구하는 규칙으로 사용하지 않는다. 혼재 디렉터리는 소유 담당이 include/링크/배포 권한을 확인해 단계적으로 분리한다.

명시적으로 stage한 현재 기준선과 후속 변경을 맥락별로 커밋한다. 과거 미커밋 단계가 성공 커밋으로 존재했던 것처럼 복원하지 않는다. root index는 단일 담당에게 배정하며 독립 Meari index는 해당 담당이 소유한다. push는 별도 요청 전 하지 않는다.

공개 compile 계약은 한 책임의 Rust 원본에 둔다. proposal/conversion/document-session/intent-dispatch는 서로 다른 identity다. 진행 문서가 바뀔 때 실행 계약 identity가 임의로 변하지 않게 private 문서 include를 제거한다. 새로운 바이트 해시는 이전 문서 해시·서명·신뢰의 자동 별칭이 아니다.

일반 회귀는 공개 합성 입력을 사용한다. 실제 원문/모델 관측 인수는 명시 runtime 입력과 opt-in 게이트로 보존하며 합성 성공·기술 채택·사람 품질 승인과 구별한다. 자료 없는 기본 검사에서 실제 검증을 성공으로 가장하거나 원 assertions를 삭제하지 않는다.

## 검증과 영향

최종 공개 snapshot의 root와 pinned Meari source, 명시 수리 입력을 봉인해 private 자료 없이 검사했다. 기본435+추적 누락 복구6을 별도 인수하고 중복 재검증을 고유 시험 수에 더하지 않았다. 이 결과는 전체 제품/UI/실제 미디어/모델 품질·배포 승인이 아니다. submodule 새 커밋은 미push이므로 원격 가용성도 별도 게이트다.

구체 입력·실패·수리·범위는 [공개 재현성 계획](../../plans/player/git-reproducibility.md)과 [계약/테스트 분리](../../plans/player/public-contracts-split.md)에 연결한다. 현재 상태는 [PM 조율](../../plans/player/current-coordination.md)에서 관리한다.
