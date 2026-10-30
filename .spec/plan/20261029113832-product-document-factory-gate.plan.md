---
id: plan:kinekino/20261029113832
title: 제품 document factory 검증 완료와 후속
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 document factory 검증 완료와 후속

상태: 독립 consumer 기능 gate 완료. 실제 [검증 정본](../../../sots/verification/presentation/product-document-factory-2026-10-19.md)의 최초 선택9/전체36pass·0fail·0ignored, 최소 lint 정리 뒤 영향9·strict/fmt0을 확인했다. 제품 root/창 등록 완료는 아니다.

## 고정 입력과 결과

| 항목 | 실제 근거 |
|---|---|
| 소유 입력 | proposals/{panel_model,document_factory,document_session,mod}.rs 및 tests/{panel_preparation,document_factory}.rs. 준비 owned6 차이0 |
| 외부 입력 | SDK API732cec…4567, Meari inventory978122…a422. 정확 SHA와 lock은 검증 정본 참조 |
| 실행 | 선택9 131586ms0, 전체36 --no-fail-fast121788ms0 |
| 최소 수리 | 최초 strict23288ms101의 collapsible_if/type_complexity2개만 let-chain·동일 type alias로 정리. 원 source와 diff 보존 |
| 최종 검사 | 영향9 16254ms0, strict11637ms0, fmt310ms0. 전체36은 수리 전이며 수리 후 다시 실행했다고 주장하지 않음 |
| 최종 pins | gate-r1/final-source6·final-owned-pins.json, inventory129c202…564c1. 외부 SDK/Meari delta0·manifest0, 소유 프로세스[]·heavy/freeze 반환 |

실행 디렉토리는 runtime/quality-review/player-ui-product-factory-r1/{gate-r1,retry-clippy}다. 준비 snapshot은 runtime/coordination/ui-product-factory-prepared-r1이며 최종 lint 정리 source와 구별한다. 모든 실제 수치는 검증 정본과 원 receipt로 대조한다.

## 검증한 책임

명시 proposal/source/index·원본 inventory/ModuleProgram·초기 props의 실제 세션 투영 대조, 실제 root scope·단일 registry/context·외부 parent lease·문서 생성, 부분 owner/원오류 보존과 정리 재시도, 다중 항목·refs 순서·stale/foreign 거부를 확인했다. 제품 factory는 new_retaining을 사용하며 error만 반환하는 호환 new의 partial Drop 한계가 남는다. callback 정리 증거와 실제 Native close 관측을 구별하고 cancel Requested를 terminal 완료로 취급하지 않는다.

SDK parent lease register/handle/close의 실제품 wrapper·assembly 등록은 SDK 소유다. 시험 Lease·고정 기술 데이터는 제품 기본값으로 이관하지 않는다. 기존 공통 SDK/저장소/정책을 사용한다.

## 다음 통합

이후 작성한 NativeWindowHost bridge·새 시험 소스는 [별도 bridge gate 준비](native-window-bridge-gate.md)로 추적한다. 해당 변경은 위 완료 입력을 상속하지 않는다. 실제 제품 창·root 등록·fresh CLI/project/provenance·커서/오디오/seek stall 원인·청취·전체13화면 시각 비교는 미완료다.

누적된 소스 준비·미실행 단계의 원문은 [과거 준비 이력](history/product-document-factory-preparation-2026-10-19.md)에 보존했다. 이 문서는 현재 상태와 완료 인수·다음 통합만 관리한다.
