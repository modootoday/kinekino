---
id: sot:kinekino/ui-2
title: 실제 모델 표본 검토 UI 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 실제 모델 표본 검토 UI 계약

2026-10-19. 사용자에게 검토 방법과 실제 자료를 제공해야 한다는 후속 요구에 대한 UI 설계다. [SDK 계약](../../application/api/model-review-contract.md)과 [모델 정책](../../application/model-quality-review.md)을 사용한다. 기존 [MODEL 시안 계약](models-visual.md)의 설치/후보 정보와 역할을 구분한다. 현재 문서는 설계이며 GUI 구현·실행 인수가 아니다.

## 위치와 공용 구성

2026-10-19 구현 연결: `StudioShell`의 기존 MODEL 화면에 `model_review`를 등록했다. 네 가지 `reviews.*` 명령의 실제 supported capability를 함께 확인하고 기존 client를 worker에 전달한다. 공용 `text_document`는 전체 UTF-8 줄바꿈/선택/복사와 `REVIEW_TEXT_HEIGHT` 토큰 스크롤을 소유한다. 상태 테스트는 선택 변경 시 메모/판정 초기화, 필수 입력, pending 중 선택 잠금, 오래된 generation과 다른 package의 텍스트 배제를 다룬다. 이 기록 시점에는 통합 cargo gate 및 실제 GUI 인수가 아직 남아 있다. 아래의 ‘모듈 등록 전’ 기록은 이전 단계의 관측이다.

root의 후속 r3 자료는 `sdk-text-results-r3`, package SHA `659b3cb6757259d8e770f6d3c96c4a77dafbab43c6ea540bcc7e0c58e4eb0197`이다. UI 코드에는 이 값을 고정하지 않는다. 측정값의 `batch_*`와 `request_*` 키를 그대로 표시하고 모델 적재 및 여러 작업을 포함한 batch 자원을 단일 작업 자원으로 표현하지 않는다. r2 관측을 r3 GUI 인수로 확대하지 않는다.

실제 자료 관측: root가 제공한 [9개 표본 package](../../../../artifacts/quality-review/sdk-text-results-r2/packages/98762d00f8b3bb3f2db9f82a395e29e42e5e48072a143f3a5ff65244ba2d1f78.json)를 읽어 Gemma270/Llama1B/Qwen0.6B의 correction/cards/highlights 각3개 task, 총9개 item과 item별 candidate asset1개/criterion1개를 확인했다. 예를 들어 correction의 방법은 ASR 초안과 결과를 단어별 대조해 변경/누락/반복/잘림을 확인하는 것이고, 기대 결과는 구두점 교정에서 단어·의미 보존과 불확실한 단어를 만들지 않는 것이다. 이 관측은 실제 의미 품질 판정이나 SDK 실행 재검증이 아니다.

기존 모델 관리 중앙에 ‘실제 표본 검토’ 영역을 추가한다. 새 전역 탐색이나 임의 모델 채택 탭으로 만들지 않는다. 공용 Accordion/Card/Badge/Button/Notice·토큰·Pretendard를 조합하고 설치 정보 검사기의 다운로드 동작과 판정 저장을 혼합하지 않는다.

package SHA 입력32와 자료 열기 → package/item 선택 목록 → 검토 방법/기대 결과 → source·baseline·candidate 비교 → 알려진 문제/자동 검사/측정값 → 사용자 판정/필수 메모 → 저장된 불변 이력 순서다.1440에서는 조건을 공유한 비교 패널을 병렬 배치하고980 미만은 같은 순서로 적층한다. 긴 조건/본문은 전체 읽기·줄바꿈/스크롤과 원문 복사를 제공하며 텍스트를 말줄임한 상태로 검토 완료를 유도하지 않는다.

표본 목록은 item_id/task와 후보 model/revision/precision/engine 조건을 함께 표시한다. 원문 baseline은 사람 gold/모델 출력과 구분한다. 비시간 표본 range=null을 임의 PTS로 채우지 않는다. 각 criterion의 how_to_review와 expected를 함께 보여 주고 알려진 문제와 자동 검사 결과를 사람 판정과 분리한다. 값이 없는 자원 측정은 미측정으로 표시한다.

같은 task의 여러 model/precision/engine 표본도 같은 결과 컴포넌트로 비교할 수 있게 구성한다. package의 task/item identity를 그대로 사용하고 표시 이름 문자열로 같은 조건을 추측하지 않는다. 기준선이 동일해 보여도 hash/조건을 확인하지 않고 하나로 병합하지 않는다. 화면 폭이 좁으면 결과 카드를 적층하되 각 모델 조건과 원문/기준선의 대응을 잃지 않는다. 판정은 비교 화면 전체가 아니라 명시적으로 선택한 item에만 기록한다.

## SDK 연결과 상태

r153 후속 검증 설계: 원본 자료 접기/펼치기 전환을 실제 입력과 같은 egui pointer event로 재현하고, 여러 frame에서 discard와 idle 안정 여부를 검사한다. 레이아웃 경고를 숨기지 않고 최소 원인을 수정한다. 일반 실행의 기존 `--record-validation` 활성 시에만 제작 heading에 검토 창 크기 선택기를 제공한다. 이 도구는 egui viewport 크기 명령만 사용하며 실제 SDK client·자료·스토어는 그대로 사용한다. 760/980/1180/1440 preset은 개발 검증 값으로 격리하고 제품 기본 설정이나 DPI/모니터 변경으로 해석하지 않는다. r153의138 통과 체크포인트와 후속 소스/검사를 구별한다.

StudioShell이 이미 받은 ApplicationClientHandle을 사용한다. main은 같은 SDK에 review_store를 주입하며 UI가 경로를 열거나 파일을 직접 읽지 않는다. package SHA는 명시 입력으로 받고 특정 실제 package hash를 UI 코드 상수로 고정하지 않는다. 선택 진입값의 자동 제공은 별도 시작 설정으로 조율한다.

inspect/텍스트 asset 읽기/decisions/record_decision은 공통 usecase를 호출한다. 텍스트 자료는 SDK의 허용된 media type·범위·크기 계약을 따르고 GUI에 임의 경로를 전달하지 않는다. 미주입/unsupported·자료 없음·자료 준비 필요·검증 실패·로딩·읽기 완료를 구별한다. not_found를 사용자 승인 대기로 표시하지 않는다.

동기 SDK 호출은 UI 프레임에서 직접 수행하지 않고 기존 자막 작업과 같은 worker/poll 경계를 사용한다. 응답은 package SHA/item ID/asset SHA 및 요청 generation에 연결한다. 선택 변경 뒤 늦은 응답이 다른 표본의 원문/결과/판정 메모를 덮어쓰지 않도록 한다. 캐시도 같은 identity를 사용한다. hash/bytes 확인은 자료 무결성이고 의미 품질 통과가 아니다.

## 판정과 저장

‘사용 가능 / 수정 필요 / 판단 보류’는 사용자 표본 판정이다. 최초에는 미선택이며 필수 메모가 없으면 저장할 수 없다. AI가 예시/초기 선택을 사용자 승인으로 저장하지 않는다. 저장은 사용자가 명시적으로 실행한 요청 한 번에만 대응한다. pending 중 중복 실행을 막고 실패/unknown을 성공으로 표시하거나 자동 재시도하지 않는다.

저장 전 package SHA/item/조건/표본 범위를 표시한다. 저장 뒤 실제 decision_sha256와 반환된 decision·sample_review_passed를 구별해 표시한다. sample_only와 automatic_application_enabled=false를 계속 표시하고 모델 전체 승인/자동 적용 활성화로 번역하지 않는다. baseline 없음·자동 검사 미실행/실패·deferred/needs_changes는 SDK가 정한 통과 의미를 유지한다.

이력0개는 ‘사용자 표본 판정 없음’이다. 기존 이력의 시간순/최신 자동선택·집계 승인을 만들지 않는다. 새 판정은 새 decision ID로 추가하며 기존 기록을 수정·삭제하지 않는다. 실제 사용자 메모가 필요한 저장 기능은 대리 시각 검토 중 실행하지 않는다.

## 구현·검증 게이트

구현 준비 상태: presentation/studio/model_review/worker.rs에 typed client의 inspect/read_text_asset/decisions/record_decision을 UI 프레임 밖에서 실행하는 worker를 작성했다. 응답 envelope는 generation/package SHA/item ID를 포함하고 완료 후 repaint를 요청한다. 아직 모듈 등록·StudioShell/컴포넌트 연결 전이라 현재 cargo gate가 이 파일을 검증하는 것으로 주장하지 않는다. 실제 stale 응답 배제·저장 pending/오류 처리·자료 비교·전체 테스트와 GUI 인수는 남아 있다.

state.rs도 별도 준비했다. 단일 pending 동안 선택/중복 요청을 막고 package+asset SHA 캐시, 명시적인 이력 갱신, 선택 변경 시 판정/메모 초기화, domain.validate를 따른 저장 가능 여부, 저장 응답의 package/item/scope/automatic=false 확인을 소유한다. 모듈 등록 전인 점은 동일하며 rustfmt 확인만으로 타입/동작 검증을 주장하지 않는다. 화면 연결 후 pending·잘못된 응답 식별자·필수 메모·정확한 표본 저장 범위를 의미 있는 fake client 회귀와 실제 읽기로 검증해야 한다.

SDK 텍스트 읽기 계획 전달(아직 구현 완료 통지 전): ReadReviewTextAsset{package_sha256,asset_sha256} → ReviewTextAsset{package_sha256,asset_sha256,bytes:DecimalU64,media_type,utf8}, ApplicationClient::read_review_text_asset(context,input), command reviews.read_text_asset. package가 실제 참조한 asset SHA만 허용하며 검증/guard 유지 후64KiB·strict UTF-8·text/* 또는application/json을 지원한다. UI는 지원 밖 자료를 빈 텍스트 성공으로 표시하지 않고 원래 error code와 해결 안내를 표시한다. Unsupported(media/상한),NeedsInput(invalid UTF-8),NotFound(unknown asset),InvalidArgument(invalid SHA),RevisionConflict(변조)를 구별한다. 이 설계 기록을 실제 메서드 가용성의 증거로 사용하지 않는다.

UI 소유 세션이 typed 상태와 공용 비교 컴포넌트를 구현한다. SDK는 텍스트 asset 읽기 최종 계약, main은 backend 주입을 소유한다. 실제9개 package는 root가 제공한 자료이며 이를 GUI 인수 완료로 간주하지 않는다. 기존 readonly IAB 도구도 제품 GUI의 대체 인수 근거가 아니다.

실제 실행에서1440/760·긴 모델조건/일본문자·표본 선택/키보드·비교 스크롤·접기/펼치기·unsupported/자료 없음/읽기 실패·선택 변경 중 응답·필수 메모/미선택의 저장 비활성을 확인한다. fake client 테스트와 실제 자료 읽기는 구별한다. 사용자 판정을 자동 생성하지 않으며 실제 저장 검증은 명시적으로 허용된 테스트 store에서만 수행한다. 전체 제품 시안 일치 goal은 이 확장과 함께 유지한다.
