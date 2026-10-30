---
id: sot:kinekino/asset-projects
title: 불변 자산 프로젝트 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 불변 자산 프로젝트 계약

[2D 자산 공통 계약](asset-generation.md)의 원 입력·변환 값을 저장하고 다시 선택하기 위한 문서/저장 경계다. 저장된 image/mask/review 참조는 원 파일·관측·사용자 검토의 admission을 대신하지 않는다. 실제 렌더 전에는 기존 `asset_generation::prepare`로 원 lease를 다시 열어야 한다.

## 문서와 순서

AssetDocument는 working revision, 원 frame ID별 source slot, 명시 순서의 frame entries와 layer entries를 가진다. source slot은 StaticPng 또는 예상 VideoPosition이며 selected_image는 없을 수 있다. 빠진 frame은 expected source slot과 bounded reason으로 남겨 성공 frame으로 위장하지 않는다. Available frame의 image pin은 source slot의 선택과 같고 mask/review는 전체 원 입력 binding을 유지한다.

`single_frame`은 빈 draft 또는 최대1frame, `declared_frame_order`는 사용자가 지정한 순서, `source_video_clock`은 동일 source SHA/stream/clock/회전의 증가하는 canonical i64 PTS 순서다. 정적 PNG에 시간이나 영상 관측을 만들지 않는다. VideoPosition은 예상 값이며 실제 frame-observation resolver가 없는 native 렌더는 계속 Unsupported다. layer의 visible/locked/order/transform을 보존하고 hidden layer를 frame_request로 몰래 렌더하지 않는다. locked는 렌더 금지가 아니다. 공통 typed 편집/undo 유스케이스와 실제 sequence 출력은 후속 구현이다.

source slot은 저장 후 제거하거나 position을 바꿀 수 없다. 최초 selected_image None→Some은 허용하되 선택된 image를 잊거나 바꾸지 않는다. frame을 삭제해도 slot을 유지해 같은 ID의 다른 원본 재사용을 거절한다. 원본 재연결은 새 frame identity로 명시한다. mask revision/명시 미검토 확인·review 선택은 새 working revision에서 공통 입력 규칙을 다시 검증한다.

현재 문서 한도는 sources/frames 각각4096, layers64, canonical document4MiB다. 빈 frame 목록은 저장 가능한 draft이며 렌더 성공이 아니다.

## publication과 native 파일

application AssetProjectStore는 create/save/inspect와 별도의 publication revision·parent SHA·canonical bytes를 공유한다. source history 불변과 working revision 증가를 검증한다. native adapter는 원 writer lock 아래 현재 head를 다시 읽어 CAS를 검사하고, 원 write+DELETE handle의 no-replace rename으로 `asset-ID/r-N-SHA.json`을 발행한다. 같은 revision의 복수 파일·비정규 bytes·parent/문서 hash 불일치는 NeedsInput이며 임의 head를 고르지 않는다.

실패한 `.pending-PID-counter.json`은 별도 비발행 파일이다. 정상 head를 가리지 않고 pending count로 드러내며, 수동 save 재시도도 원 partial을 보존한다. 최초 create가 불완전한 directory를 남겼다면 새 create로 덮어쓰지 않고 inspection을 요구한다. 자동 삭제/채택/재시도는 하지 않는다. post-rename 검사 오류에는 publication_committed·SHA/revision, commit 오류에는 publication_may_exist를 원 오류와 함께 보존한다.

현재 publication4096·pending64·검증할 전체 history256MiB 한도를 적용한다. native 저장은 동기 원자 publication이며 timeout/idempotency context를 지원으로 광고하지 않는다. 실제 disk-full/permission/프로세스 강제중단/ACK 유실·power-loss 내구성·recovery 명령은 이 구현 선언으로 인수하지 않는다. 기존 root index·다른 담당 Card/Clip store를 수정하거나 새 저장으로 옮기지 않는다.

## 다음 연결

현재 R225 이후의 owned 출력/공유 원 보유자와 plan/2 계약은 [자산 묶음 출력](asset-bundle-output.md)을 따른다. R222/R223의 borrow 기반 단계 증거는 역사로 보존하며, 최신 출력은 wrapper의 Arc 공유로 원 identity를 유지한 채 owned Jobs에 전달 가능한 값으로 정리했다.

R222 제작 입력은 SavedAssetProjectPin의 document/revision/SHA를 원 AssetProjectAdmission으로 연다. NativeAssetProjectLease는 선택 publication뿐 아니라 검증한 history/project directory 및 원 ancestor handle의 복제 핸들을 보유한다. catalog가 해제돼도 원 project/source 렌더 carrier 수명이 끝나기 전에 해당 보호를 잃지 않는다. writer lock은 capture 동안만 보유하고 선택 publication은 새 head 저장 이후에도 불변이다. 이 host lease는 Clone/Deserialize로 만들지 않는다.

CapturedAssetProject의 plan은 원 document의 frame 순서와 0기반 position, source position/clock/PTS 및 visible layer 순서를 고정한다. locked layer는 렌더 가능하며 hidden layer는 제외한다. MissingAssetFramePolicy::Reject는 NeedsInput, PreserveGap은 원 위치와 이유를 그대로 남긴다. available frame/visible layer가 없는 draft는 출력 성공이 아니다. 현재4096 frame/layer render unit 한도를 초과하면 ResourceExhausted다. plan DTO 자체는 source/review admission이 아니며 JSON 재역직렬화를 실행 권한으로 사용하지 않는다.

각 render_frame은 원 publication에서 frame/layer request를 다시 선택하고 기존 AssetInputAdmission과 공통 raster를 실행한다. 결과 carrier가 CapturedAssetProject를 빌려 원 project lease와 실제 source lease를 함께 유지한다. 이 단계는 원 Jobs capture 및 manifest 연결에 필요한 입력 경계이며 Jobs 등록·파일 발행·전체 sequence 내구성·사용자 품질 승인을 구현했다고 표시하지 않는다.

R219/R220에서 공통 ApplicationClient create_asset_project/save_asset_project/inspect_asset_project와 PlayerSdk의 명시 AssetProjectStore 포트를 추가했다. `asset_projects.create/inspect/save_revision`은 strict payload 및 publication expected_revision을 사용한다. 로컬 CLI는 `--asset-project-root EXISTING_ABSOLUTE_DIRECTORY`로 host가 저장 루트를 선택하며 request JSON으로 경로를 공급하지 않는다. endpoint/serve 조합은 거절한다. timeout/idempotency/in-flight cancel은 지원하지 않으며 schema/capabilities는 source_admitted/review_admitted/build_assets_supported/quality_approved=false를 명시한다. 실제 SDK/CLI 실행 결과가 확보되기 전까지 이 연결 구현을 기능 인수 완료로 표시하지 않는다.

R217에서 scan으로 검증한 기존 publication과 pending 파일 및 project directory의 원 핸들을 보유하는 RetainedAssetHistory를 추가했다. save는 CAS 확인부터 새 publication/readback 종료까지 carrier를 유지한다. load의 DTO 반환과 저장 중 보호 수명을 구분한다. 실제 쓰기 open 거절/해제 시험을 추가했으며 실행 결과는 별도 검증 기록으로 확정한다. R217 첫 strict(2916)는 frame enum large_enum_variant로 실패했고, R218은 serde 형태를 유지하는 Box 간접 참조로 수정한 별도 입력이다. 아직 저장 구현의 실제 시험 성공을 선언하지 않는다.

원 저장 publication SHA를 Jobs capture와 출력 manifest에 고정하고, 동일 Rust 유스케이스의 SDKCLI create/save/inspect·실제 render/오류/취소/재조회에 연결한다. source/model/review resolver·마스크 수정/undo·sequence·투영 파일 묶음/GLB·GUI 인수와 기존 후속 범위를 유지한다. production.build_assets 지원 또는 전체 제작 완료로 승격하지 않는다.

기존 modoo.today image-postprocess의 원본 불변/출력 규격/재검증과 QA fixture·변경 검증 skill을 적용한다. 정확한 출처·해시는 [공통 계약](asset-generation.md)과 [입력 검증](../../../verification/application/asset-input-admission-20261029.md)에 연결한다.
