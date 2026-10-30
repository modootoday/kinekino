---
id: plan:kinekino/20261029113959
title: 공개 계약과 테스트 입력 분리
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공개 계약과 테스트 입력 분리

2026-10-20. 현재 구현과 커밋의 기록이며 Cargo 검증 완료를 뜻하지 않는다. 상위 진행은 [공개 재현성 계획](git-reproducibility.md)을 따른다.

## 계약 원본

기존 `media-contracts/src/plugin_contracts/`가 공개 UTF-8 바이트 원본을 소유한다. 새 Cargo 패키지나 제품 표시 이름 의존을 추가하지 않는다.

| 독립 버전 | 책임 | 소비 지점 |
|---|---|---|
| proposal_v1 | canonical SDK 제안 생성·조회·수정·명시 채택·historical binding | root proposal manifest |
| conversion_v1 | typed 자막 변환·호출별 context·SDK 오류 보존 | bridge library |
| document_session_v1 | 단일 registry 소유·원자 재투영·실패 시 이전 owner 보존·명시 cleanup | document_repair_sdk, native_document_sdk, original_review_sdk |
| intent_dispatch_v1 | bounded presentation queue·stale/중복 거부·dispatch 반환 후 drain | proposal_view, support/callback_fixture |

각 책임은 별도 identity다. private 진행 문서 바이트를 복사하거나 모든 계약을 같은 해시로 합치지 않았다. 새 바이트를 해시하므로 과거 문서의 contract hash와 다르며 이전 핀·서명의 자동 호환이나 인증을 주장하지 않는다. provider/request/reply/error 해시 계산과 기존 테스트 assertions는 유지했다.

## 테스트 자료

- actual 2곳: infrastructure historical_evidence 및 async_proposals policy의 기존 ignored 실제 원본 9개 검사. `HISTORICAL_COMPANION_PATH`의 명시 절대 경로를 읽고 기존 SHA·16566 bytes를 확인한다. 실제 assertions·verifier pin·이전 evidence를 보존한다.
- mock 2곳: application ai_provenance 및 ProposalFiles 테스트가 `cfg(test)` 공통 historical_fixture의 합성 request/candidate/reference를 사용한다. 고의 malformed candidate로 NeedsRepair 검사를 유지한다. 실제 모델 관측 성공으로 표시하지 않는다.
- review 1곳: 통합 테스트 전용 review_package가 typed 합성 ReviewPackage를 만든다. 실제 추론·사람 품질 승인과 별개다.

누락·상대 companion 경로를 I/O 전에 거부하는 Rust 회귀 1개를 추가했다. actual 원본·모델·키·영상은 공개 소스로 옮기지 않았다.

## 커밋과 근거

`2232b67`은 공개 계약 2종, `a40b718`은 root SDK/CLI와 필수 기존 자산, `6d9f666`은 bridge의 당시 소스 기준선이다. 후속 `1fd8ca1849eb0fd0012f478fa6acf98a43595476`은 추가 계약 2종과 테스트 5참조를 포함한 정확8파일 수리다. 새 SDK15파일과 후속8파일의 형식 검사는 통과했다. 후속8파일의 커밋 전후 해시 차이는 0이고 staged 잔여는 0으로 반환했다.

root src/tests 및 bridge src/tests의 제한된 multiline include 검색에서 private `.spec` 컴파일 입력 잔여는 0이었다. 이는 모든 동적 파일 의존이나 실행 성공을 입증하지 않는다. 원본 목록·핀·검색 결과는 비공개 `runtime/coordination/sdk-git-baseline-r1/`에 보존한다.

## 검증 근거와 잔여

공개 snapshot의 고정 입력·archive/overlay 핀·기본435와 복구6·첫 strict 실패와 수리·Native/bridge 집중 범위·종료 관측은 [SDK 공개 입력 검증](../../sots/verification/application/sdk-public-input-2026-10-20.md)에서 관리한다.

이 분리의 배정된 기본·집중 검사는 마감했다. 전체 bridge off/on, actual R3의 명시 입력 실행, OS·실사이트·새 AI 실행, 제품 전체 완성과 원격 재현 가용성은 이 checkpoint에서 인수하지 않았다. 후속 P3 source는 별도 입력·검증으로 추적한다.
