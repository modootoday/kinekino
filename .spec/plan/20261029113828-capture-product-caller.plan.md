---
id: plan:kinekino/20261029113828
title: 선택 프로젝트 caller와 처리 기록 인수
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 선택 프로젝트 caller와 처리 기록 인수

독립 source-only 후속 `runtime/development/ui-capture-product-caller-r1`이다. 부모22 B5CE는 덮어쓰지 않는다. 부모의 실제 검증·배정을 이 후속 때문에 지연하지 않는다.

`player --native-viewer --project-root=<절대 경로> --viewer-project=<ID> [--viewer-project-revision=<양의 정수>]`는 기존 `projects.inspect`를 사용한다. revision 생략은 해당 SDK의 latest 선택 의미를 그대로 사용하며 반환된 StoredProjectRevision 전체를 원 frame/entry로 전달한다. ID를 생략하면 기존 기본 화면을 유지한다. 파일 검색·임의 item0 선택·origin 발급·자동 새 project 생성은 없다. 입력 오류와 SDK inspect 실패는 LocalHost/Native/worker 생성 전에 반환한다.

기록 저장은 기존 CatalogArtifacts/CatalogFiles의 immutable body+metadata publication과 재검증을 재사용한다. 설정된 project root 아래 별도 `viewer-execution-evidence` 책임 디렉터리를 사용하며 `p-<project ID>` revision 저장과 섞지 않는다. 분류는 NativeEngineDiagnostic, representation은 native_ui_sdk_execution_diagnostic이며 웹 응답·모델 품질 승인·지속 가능한 SDK operation identity를 주장하지 않는다.

원 request/terminal/disposition/첫 fault를 부모가 인수한다. 직렬화된 원 bytes와 metadata는 첫 시도 뒤 고정하며 publication 영수증을 잃어도 같은 저장만 재시도한다. 저장 성공 전 원 carrier는 부모 pending에 남고 새 편집·user close를 막는다. 성공한 불변 저장 영수증을 받은 뒤 메모리 carrier를 소비하여 정상 세션이 임의8회 제한에 걸리지 않게 한다. SDK mutation/ACK는 저장 재시도에 의해 반복하지 않는다. 명시적 결과 결정 전 자동 소비·버리기는 하지 않는다.

저장 오류는 첫 진단과 현재 pending을 보존한다. 일반 after_dispatch/close poll은 실패한 저장을 자동 재시도하지 않는다. 제품의 재시도는 같은 원 저장으로 연결하고, 닫기 중에는 신뢰된 host recovery API로만 저장 재시도를 요청한다. 원 host/attachment/worker cleanup을 완료로 가장하지 않는다.

검증 후보는 선택 ID/revision/중복·누락·scope 파싱, 실제 SDK 선택→product callback→9회 편집/불변 기록 reopen, publication 완료 뒤 영수증 손실과 같은 bytes의 명시 재시도, 닫기 중 원 carrier 보존이다. 기존 artifact-port를 통한 저장 fault와 미지원 concrete SDK ACK fault는 구분한다. 모든 시험은 다음 배정에서 실행하며 현재 Rust leaf format 외 타입·module graph·실행·GUI 근거는 없다.
