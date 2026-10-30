---
id: plan:kinekino/20261029113938
title: Historical import UI 연결 잔여
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Historical import UI 연결 잔여

2026-10-20 읽기 감사. 실행 입력·SDK/Native/기존 UI source는 변경하지 않았다. cargo/GUI 실행 0. SDK 담당의 실제 CLI 교정·Card·HL edit/accept/reopen 및 전체396/strict/fmt 통과 인계는 SDK 기능 근거이며 아래 GUI 연결이나 bootstrap 검증 근거가 아니다.

후속 bootstrap checkpoint에서는 [별도 consumer 61개·strict/fmt 검증](../../../sots/verification/presentation/bootstrap-preparation-2026-10-20.md)을 완료했다. 아래는 당시 읽기 관측이다. document3/binding1·legacyNone·잘못된 version/binding·원본/수정 output 품질 분리의 순수 회귀는 현재 통과했으며 실제 package reader/GUI capability·원본 decision 표시의 잔여는 유지한다.

## 관측 사실

- `src/domain/proposals/stored.rs`의 `StoredProposal.historical`은 canonical `Option<HistoricalBinding>`이며 serde default/None 생략을 사용한다. `historical.rs`는 document 3/binding 1, legacy document 1·2를 구별한다. `StoredProposal.validate()`는 binding 유무에 따라 document version을 판정한다. SourceSnapshot 자체의 version과 document version은 같다고 가정할 수 없다.
- 새 `bootstrap.rs`는 source version으로 document를 재작성하지 않고 canonical validate 뒤 전체 StoredProposal을 borrow한다. 따라서 읽기상 document 3/binding 1을 버리거나 거부하는 자체 version 분기는 없다. 이 검토는 컴파일·실행 호환성 확인이 아니다. `bootstrap/tests.rs`의 저장 JSON fixture는 historical 필드 없는 legacy 경로만 다룬다. document 3/binding 1 및 잘못된 version/binding 조합의 실제 회귀는 후속 배정 대상이다. 현재 fixture를 실제 provenance 검증 표본으로 취급하지 않는다.
- bootstrap의 Card/HL `qualityLabel`은 caller가 전달한 `PanelLabels.quality_unknown`을 그대로 사용한다. correction/repair에는 독립 provenance/quality 표시 props가 없다. 전체 canonical binding을 보존하는 것과 화면에 정확히 표시하는 것은 별개다. 원본 companion 결합이 있다는 사실만으로 품질 승인·human gold·수정 output의 verdict를 만들지 않는다. 실제 reader 검증은 SDK store 책임이며 순수 bootstrap의 구조 검증으로 대체하지 않는다.
- `src/application/ai_provenance/import.rs`의 canonical `ImportHistoricalProposal`은 proposal_id/source_id/source_revision/reference를 받는다. reference는 companion SHA·bytes·item_id이며 source request·candidate raw/task는 reader가 복원한다. 입력에 실행 파일 경로나 임의 raw를 추가할 필요가 없다.
- `src/assembly/meari/proposal_manifest.rs`와 `proposals.rs`는 여전히 create/inspect/edit/accept의 4개 capability·typed binding이다. SDK command/schema의 import_historical 지원과 GUI plugin registry 지원은 서로 다른 상태다.
- 공통 `application/launch/historical.rs`는 verifier 절대 경로·실행 파일 SHA·workspace·package의 명시 설정과 local proposal storage를 요구한다. 미설정 reader로 import하면 canonical Unsupported다. store load/publish도 bound proposal을 재검증한다.

## 후속 배정 시 연결 범위

1. SDK 소유 manifest/typed adapter/registration에 `proposals.import_historical` 지원 여부를 확정한다. capability descriptor·request/reply/error pin과 canonical ImportHistoricalProposal/StoredProposal을 사용하고 DTO를 복제하지 않는다. 현재 4개 목록에 이름만 추가해 지원 완료로 표시하지 않는다.
2. UI의 검증된 package/item 선택 결과를 해당 launch package·companion reference 및 명시 proposal/source ID·revision과 연결한다. 실행 설정은 신뢰한 launch 조립이 소유한다. 브라우저/웹 메시지/시각 컴포넌트가 verifier 경로·workspace·package를 임의 선택하거나 셸을 실행할 권한을 받지 않는다. source/raw는 같은 공통 import가 복원한다.
3. 별도 명시 import action을 같은 registry·client·request context로 호출한다. import 완료 StoredProposal의 정확 ID/revision/source SHA와 명시 항목 선택을 bootstrap에 전달한다. 첫 package/item/항목 또는 demo fixture를 제품 기본값으로 사용하지 않는다. create로 우회하여 binding 없는 proposal을 만드는 fallback도 두지 않는다.
4. capability 미지원과 명시 verifier 미설정을 구별해 사용 불가 이유를 표시한다. SDK가 반환한 Unsupported 및 typed 오류를 보존한다. 자동 설치/실행 경로 추측·자동 설정을 하지 않는다. 실제 실패 뒤에도 원 선택·raw·NeedsRepair/import_error를 보존한다.
5. 원본 evidence binding/검증 상태·원 후보 품질 판정·사용자 수정 draft·accept output을 구별해 표시할 canonical 계약을 SDK와 확정한다. evidence_verified는 증거 무결성, review_ready는 baseline 존재이며 품질 승인이 아니다. 현재 caller 제공 품질 문구만으로 원본 decision 연동 완료라고 표시하지 않는다.
6. 동기 verifier 지연을 UI thread에 연결하기 전에 SDK scheduling port와 typed Pending→실제 terminal wake를 조율한다. worker에는 Rc document/registry/GL을 보내지 않는다. 기존 headless Pending 방어 시험은 실제 provider 비동기 완료 증거가 아니다.

인수는 canonical version3/binding1·legacy None·binding 누락/불일치·stale 선택·Unsupported·원본/수정 output 품질 분리의 기능 회귀를 SDK/consumer에서 먼저 수행한다. 실제 창에서는 선택/불가 사유/처리 중 상태/terminal 반영·키보드/IME·시각 표현만 검증한다. 현재 source 동결 및 중앙 실행 슬롯을 유지한다.
