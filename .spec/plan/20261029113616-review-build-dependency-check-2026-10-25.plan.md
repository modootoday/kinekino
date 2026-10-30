---
id: plan:kinekino/20261029113616
title: 12시간 검토 빌드 의존성 점검
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 12시간 검토 빌드 의존성 점검

목표·범위·현재 담당은 [통합 계획](user-review-build-12h-2026-10-25.md), 실행 권한은 [실행 배정](execution-leases.md)이 소유한다. 이 문서는 산출물 누락과 공유 자원 의존성만 점검한다.

## 기준과 산정 한계

사용자 지정 기간은 12시간이다. PM 기록의 시작 01:59 KST와 목표 13:59 KST를 따른다. 개별 남은 구현·실제 실행의 확정 소요 시간은 없다. 따라서 임계 경로 시간·완료 시각·여유 시간을 임의 수치로 채우거나 12시간 내 성공을 확정하지 않는다. 아래 관계는 작업 순서이며 예상 시간이 아니다.

## 산출물별 작업 배선

| 사용자 산출물 | 남은 직접 작업 | 담당 | 선행 조건 |
|---|---|---|---|
| 원 단일 창의 네이티브 영상 | 생성 style·focus·원 owner 정리의 실제 관측, SDK 늦은 연결, 실제 미디어 통합 | UI·SDK | 창 CPU 검사 → 원 창 OS → 동일 SDK/route 통합 |
| 탐색·회차·재생 컨트롤 | 기존 CLI 기능을 단일 client에 연결하고 실제 제품 상태 표시 확인 | UI·SDK | 원 ready route·동일 client |
| 네이티브 탐색·음량 슬라이더 | 기존 Native 엔진의 range 미지원에 대한 최소 Rust 입력·표시 어댑터와 제품 연결 | Meari·UI | 원 callback 계약, pointer/keyboard/disabled/focus·정리 검증 |
| 성공 후 상태 자동 갱신·전체화면 idle | 원 window/document에 귀속된 monotonic 예약·취소·stale 거부와 제품 상태 갱신 연결 | Meari·UI | 기존 WaitUntil 재사용, failed-turn retry와 분리, 새 thread·busy loop 없음 |
| 실제 AI 원본/후보 수정·저장 | NeedsRepair 명시 진입·typed 수정 UI23 검사와 제품 연결 | Media·UI | 기존 SDK 계약, 후보 타입·회귀 |
| 원시 AI 후보 품질 판정·이력 | reviews4의 단일 Pending 연결·원 identity·늦은 응답 정리·GUI 표시 | Media, Meari 소스 지원 | 기존 단일 execution attachment, 원 package 입력 |
| 실제 하이라이트 구간 재생 | 원 영상·검증된 자막/PTS 표본 확보 및 원 구간 연결 | Media·UI·SDK | range=null인 현재9와 구별한 실제 근거 |
| 검토용 실행본과 안내 | 같은 소스 조합의 빌드·실행·오류/종료 확인, 핀·표본·저장 위치·미완료 안내 | PM·각 담당 | 위 결과 통합, CLI/GUI 검증 구별 |

모든 우선 산출물에 작업과 담당이 있다. 다운로드·나머지 제작·AI 범위는 [전체 게이트](full-product-integration-gates.md)에 남는다. 현재 미완료인 기능을 제외한 채 전체 완성으로 재정의하지 않는다.

## 자원에 따른 경로

창 경로 = 생성 수정 CPU + 실제 원 창 OS + SDK/원 client 통합 + 실제 미디어 + 통합 빌드 확인.

AI 경로 = repair·raw 판정 소스 구현 + CPU/CLI 검증 + 동일 제품 연결 + 실제 표본 화면 확인.

두 소스 구현은 병렬이다. CPU·미디어·AI 실행은 같은 중앙 슬롯을 사용하므로 동시에 실행 가능한 독립 경로로 계산하지 않는다. 각 경로 시간 미확정으로 합계 및 float는 산출 불가다. 실패 조사·수정·재검증 시간도 완료한 시간으로 차감하지 않는다. 현재 명시된 별도 수치 버퍼는 없다. 새 순환 의존성은 두지 않는다: raw 품질 판정은 proposal 채택을, repair UI는 새 추론을 선행 조건으로 요구하지 않는다.

실제 소스 확인에서 기본 GUI의 회차 선택/playback.open이 아직 연결되지 않았고, 현재 Native 엔진은 HTML range의 실제 입력·표시를 지원하지 않았다. UI는 typed open·회차·원 engine observation/controls를, SDK는 기존 CLI의 실제 provider 갱신 경로를, Meari는 private range 후계를 맡는다. archive 조회·수동 resolution 주입·텍스트 컨트롤 임시 연결만으로 최종 흐름을 충족했다고 표시하지 않는다. 이들은 통합 구현의 미완료 사항이며 사용자 입력을 기다리는 차단 상태가 아니다.

## 적용 스킬

modoo.today `modootoday/product-planning/project-plan-scope-and-schedule-check` 본문을 실제 읽었다. 산출물→작업 coverage, 공유 자원 직렬 의존성, 확정되지 않은 duration/float 구분을 적용했다. 사용자에게 없는 기간·비용·수치 버퍼를 만들지 않았다. 범위 변경·계약 변경 승인을 요청하는 문서가 아니며 새 승인 대기로 구현을 중단하지 않는다.

2026-10-25 후속 소스 점검: 이전·다음화 서비스는 초기108 GUI 조합에 없었다. SDK의 transition11+parent-loss5는 같은 provider/files/jobs/router/TransitionSlot을 연결하는 별도 미타입 후보이며 UI가 소비한다. 부모 소실 시 transition admission을 State borrow 전에 닫고 native/execution 정리 후 slot.end, 이후 provider 정리를 수행한다. 실제 회차 교체·취소·종료 검증은 아직 없다. 성공 turn 이후 새 Completion 예약 공개 API도 없어 Meari가 별도 소유형 timer 후보를 작성한다. 수동 새로고침은 임시 연결이며 최종 자동 상태 갱신 완료 근거가 아니다.

2026-10-25 병렬 분담 후속: UI는 catalog/controls/Ready emitter를 유지하고 SDK는 frozen UI12의 setup/routed/provider_cleanup에서 원 TransitionSlot 생성·서비스 주입·입장 차단·native/execution 뒤 slot.end·provider 정리 순서만 별도 private 후보로 작성한다. SDK는 UI 작업본·canonical을 수정하지 않고 exact 선행핀/diff/소유권을 UI에 인계한다. UI가 후계에서 겹침을 명시 합성하며 Ready emitter는 중복 구현하지 않는다. 현재115 회귀 입력은 그대로 유지한다.
