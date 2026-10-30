---
id: plan:kinekino/20261029113933
title: Card 편집 → 시청 복귀
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Card 편집 → 시청 복귀

최신 상태는 [실제 CPU 소비자 검증](../../../sots/verification/presentation/card-viewing-return-consumer-2026-10-21.md)의 최종 source28 `C77A4351`이다. 같은 최종 BIN에서75 통과/제외5(신규18 포함)를 확인하고 실행 슬롯을 반환했다. 아래 source-only 작성 기록은 이력이며, 실제 사용자 컨트롤·whole-host/프레임·재열기·시각 성공은 여전히 미완료다.

현재는 source-only 후속이다. 실제 CPU 통과·제품 버튼 연결·원 Viewer 전체 종료/복귀·렌더링·시각 승인을 의미하지 않는다. 기존 [제품 진입 실제 관측](proposal-product-entry.md)과 구별한다.

검증 후보를 `runtime/coordination/ui-viewing-return-union26-r1/`의 고유26개로 단일 고정했다. source SHA `F67A4AAFBDDD413EAD81862C72C6AA7DBE48659EBA82773C094ACAF01DB1371B`, 기존13/신규13, 부모5·추가 덮어쓰기10을 기록했다. 원 union23의 before 차이0·direct rustfmt26 check0이며 Cargo/type/test/OS0다. 작성19 중 CPU18/별도 LocalHost ignored1은 실제 등록·통과 수가 아니다. PM 배정 전에는 기존 후보를 대체 적용하거나 shared snapshot을 변경하지 않는다.

독립 소스 `runtime/coordination/ui-viewing-return-transaction-r1/`는 원 ShellState의 복귀 ticket과 활성 편집기의 dirty/pending/recovery 가드, callback suffix 경계, 원 Card scope 해제, 원 읽기 owner 재등록, 동일 ProductFrame의 checked props 교체, 새 프레임 관측을 순서대로 연결한다. 각 실패 단계와 소유자를 보존하고 명시 재시도하며, props 성공을 경로 관측으로 표시하지 않는다. 원 실행/SDK/LocalHost 소유권을 바꾸지 않는다.

부모 union23·navigation guard·read resume과 함께 별도 후보로 검증해야 한다. 실행 배정 전에는 shared snapshot에 합성하거나 Cargo를 실행하지 않는다. 실제 원 Viewer fixture에서 미저장/처리 중 전환 거부, suffix 재진입, 원 입력 콜백 폐기, read export/props 실패·재시도, 종료 경쟁, 늦은 frame 거부, 동일 worker 재열기를 확인해야 한다. 현재 새 작성2는 실제 제품 소스의 primitive 연결1과 synthetic 프레임 정책1이며 이 전체 조건을 입증하지 않는다.

사용자 복귀 컨트롤은 가드/실패 회복/실제 프레임까지 검증한 뒤 공통 shell intent로 연결한다. 저장된 ProjectRevision capture와 현재 저장 revision은 별개로 유지한다. 프로젝트 자동 재기록이나 대기 선택의 임의 rebasing을 통해 재열기를 성공으로 만들지 않는다.

후속 `runtime/coordination/ui-viewing-return-whole-host-r1/`는 실제 ViewerHost/NativeHostRuntime/LocalHost/SDK를 구성하는 회귀1을 작성했다. named pipe를 시작하므로 기본 ignored이며 별도 중앙 실행 배정이 필요하다. 합성 ShellState 관측을 사용해 실제 frame context는 없으며, props 성공과 frame arm 실패의 동시 보존을 기대한다. dirty/suffix 거부·borrow 및 경쟁 attachment 실패·명시 재시도·원 read/terminal·원 close/join을 작성했지만 아직 타입/실행 근거가 없다. 실제 프레임·제품 컨트롤·전체 진입 성공으로 표시하지 않는다.
