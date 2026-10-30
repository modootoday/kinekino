---
id: sot:kinekino/native-proposal-ui-helper
title: 제품 Native proposal UI helper
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 제품 Native proposal UI helper

2026-10-20 UI 연결 4단계 검증·인계 계약. main 실행 연결과 제품 시각 인수를 완료했다는 의미가 아니다. [계획](../../../plans/player/presentation/product-native-window-helper.md), [검증](../../verification/presentation/product-native-window-helper-2026-10-20.md), [application 입력](../application/native-proposal-launch.md)을 함께 읽는다.

## 조립 입력과 단일 소유자

`presentation::web_ui_adapter::proposals::product_window::prepare_product_window`는 이미 생성된 ApplicationClientHandle, ExecutionOwner, NativeProposalLaunchRequest, ProductWindowConfig를 소비한다. SDK/client/store/NativeLoop를 생성하지 않는다. 동일 context를 초기 inspect와 실제 scope 등록에 전달한다. JSON이 TSX/CSS/assets/실행 파일을 선택하지 않으며 신뢰된 조립 측만 visuals를 제공한다.

Card/Highlight는 product proposal-panel을, CueCorrection/DocumentCorrection/DocumentRepair는 product correction-panel을 사용한다. repair용 verification entry를 제품으로 승격하지 않는다. source/actions/text-comparison을 공통 inventory로 제공하며 고유 namespace를 사용한다. 한국어 label은 ProductWindowLabels에서 제공한다. 추후 다국어·시각 토큰 인수는 별도 게이트다.

## 명시 선택

NativeProposalSelection의 checked_item_index를 통해 BootstrapSelection으로 변환한다. ID/revision/SHA/full DocumentTextRef를 보존한다. PanelSelectionMode::Item은 item_index Some이며 실제 draft가 있어야 한다. DocumentRepair는 None이며 draft 없는 document correction이어야 한다. 모순을 초기 handoff와 factory에서 거부한다. None을 repair로 추정하거나 index0/첫 원문을 대용하지 않는다.

repair Selected는 initial projection과 factory의 실제 ReviewSession에 동일 fullref를 설치한다. Unselected는 원문 선택 전 startDisabled를 유지한다. sourceChoice와 onStartManual은 실제 공통 correction entry의 입력/버튼에 연결한다. 수락은 기존 SDK 정책을 사용하며 자동 적용을 활성화하지 않는다.

## 오류와 종료

ProductWindowBuildFailure는 입력 오류의 원 ApiError/request/config/동일 worker 또는 거부된 WindowStartupInput을 보존한다. ProposalWindowStartup은 initial/handoff/mount failure/bridge failure/runtime 중 하나의 소유 경계를 확인한다. SDK/모듈/등록/attach 원인과 partial owner를 별도 필드에 남긴다.

OwnedProductCleanup은 BuildFailure/Headless/Returned owner와 관측 근거를 보유한다. poll은 terminal을 수락하거나 ACK하지 않는다. 초기 결과는 accept_initial_for_shutdown으로 명시 수락하며 immutable typed/raw Arc를 보존하고 반복 수락은 같은 Arc를 반환한다. 잘못된 UI 선택의 성공 terminal도 이 경로로 격리하며 live mount로 전환하지 않는다. invalid envelope/oversize를 정상 terminal로 수락하지 않는다.

양수 deadline/최대 관측 횟수/worker delay를 필수로 사용한다. 예산 소진·오류 뒤에도 into_parts는 owner와 evidence를 함께 반환한다. 재시도는 명시 인계로 진행하며 새 loop/worker fallback은 없다. SDKQuiescent는 SDK worker join과 미ACK0을 의미한다. Native 상태 NoDocument/HeadlessClosed/Closed/ForcedIncomplete/Unconfirmed를 따로 제공한다. headless owner 재인계는 HeadlessClosed를 유지하며 returned owner 자체나 SDK 정리 성공을 OS Closed로 해석하지 않는다.

## 관측과 미검증

최종 canonical 실행은 실제 SDK headless10과 factory20을 통과했다. 기본 구성 전체440, 초기 root-on lib414(제품366/ignored5), consumer on 전체63/ignored9, off 전체96/ignored9 및 최종 보강 종료 포트43/ignored9가 통과했다. 동일 시험의 반복/복제 실행을 새 고유 시험 수로 합산하지 않는다. actual Native 정상 종료와 deferred attach 거부는 각각 새 exact process1개씩 통과했다. format과 영향 strict는 검증 문서의 회차를 따른다. 최초 실패와 미확정 timeout도 보존하며 이전 OS9/CLI/전체 시각 결과를 상속하지 않는다.

변경21 source inventory SHA256은 `535EC6B64BF0737C3C19384620E4EBD856FE982DCD963A77610330FEFBC855A9`, 고정 TSX/CSS10 입력 inventory는 `58E4C98A8ED0A2FD62EA72C1CFAB57ACBF89A8CAB485689C793A64338CCA818B`다. source 사본·diff·원 로그·receipt와 SDK8/Native316 보호 입력 관측을 runtime에 보존한다. main은 staged Unsupported를 유지하며 SDK 후속 인계 범위다.

기능 fixture의 empty CSS/assets None은 제품 테마/폰트 검증이 아니다. product_css()와 검증된 자산을 실제 main에 연결한 실행, renderer 속성·폰트 지원, 승인 시안 전체 13화면 1:1 검토는 남아 있다. 사용자 관측 흰 창과 특정 fixture의 동일성은 확인되지 않았다. VM/DPI 및 시각 반복 실행은 현재 보류 범위를 유지한다.
