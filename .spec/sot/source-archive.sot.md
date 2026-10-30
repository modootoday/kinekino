---
id: sot:kinekino/source-archive
title: 원본 자산 source archive
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 원본 자산 source archive

R251 재가져오기 계획 구현 중: 새 draft는 새 document_id/revision1로 만들고 원 source/frame pin·누락·레이어/변환은 보존한다. 사용자 요청의 acknowledge_unreviewed_masks가 명시 true일 때만 진행하며 available mask의 review는 특정 mask pin의 UnreviewedAcknowledged로 설정한다. 원 archive/project/review 참조는 수정하지 않는다. 공통 `prepare_import`는 exact archive/manifest/project canonical pin을 대조하는 값 계획일 뿐 파일 admission·저장 완료·사용자 품질 권한이 아니다. 아직 컴파일/실제 시험 전이다.

후속 Native import는 원 archive를 보유하고 PNG를 재admission한 뒤, 새 프로젝트 ID 전용 create-only import plan을 먼저 보존하고 project publication1을 생성한다. 실제 publication pin이 계획과 같을 때만 별도 commit receipt를 쓴다. 부분 실패/취소 시 원 plan·만들어진 project를 보존하고 현재 호출의 단계/오류를 돌려준다. 자동 덮어쓰기·기존 ID 재사용·묵시적 완료 복구는 하지 않는다. 원 Jobs capture는 commit receipt·archive pin을 source identity에 포함하고 imported project에서 archive PNG를 선택하도록 연결해야 한다. 원 host catalog의 같은 artifact ID에 전역 fallback을 추가해 기존 프로젝트의 선택 경로를 바꾸지 않는다. 이 연결 전에는 draft 계획만으로 재편집/Jobs 재가져오기 전체 완료를 선언하지 않는다.

R249 구현: `CapturedAssetSourceProject`는 저장 전용 원 publication 보유자이며 렌더 carrier와 타입을 구분한다. `capture_source_project`는 실제 lease/pin/canonical publication/document·명시 missing policy를 확인하지만 visible layer나 render unit을 요구하지 않는다. 기존 렌더 capture는 visible unit 조건을 유지한다. 원 lease는 Arc로 공유하며 복제된 JSON으로 admission을 만들지 않는다. source 서비스도 저장 전용 capture를 사용한다. R248의 숨김 레이어 제약은 구현상 수정했고 실제 시험은 검증 기록으로 구분한다.

R247 발행 구현: 고정 root의 writer lock 아래 새 `sources-{id}` 디렉터리만 생성하고 기존 디렉터리·부분 파일을 재사용하지 않는다. 원 no-replace 파일 writer로 각 파일을 쓰고 fsync·원 핸들 SHA/길이/FileID를 확인하며 마지막 sources.json을 completion marker로 발행한다. strict read-only 재열기 뒤에도 원 writer FileID와 대조한다. marker 전 취소는 미완성, marker 후 취소는 완성 파일이 남을 수 있으므로 original ApiError와 directory_created/marker_committed/manifest SHA를 보존하고 자동 정리·재시도하지 않는다. 이는 독립 source 저장이며 원 Jobs publication CAS 완료 증명이 아니다. 비정상 crash/전원 유실·directory durability·모든 OS fault는 아직 인수 전이다.

R244 추가 구현: `AssetSourceManifest::validate_project`는 canonical project pin/parent 형태·available binding·missing 목록·원본 순서/이름/SHA/bytes 예산을 대조한다. Native `AssetSourceArchiveFiles`는 명시 root/ID/marker SHA 아래의 일반 파일·directory 핸들을 보유하고 원본 해시·목록을 검사한다. `frame_inputs`는 선택 frame의 고정 catalog만 만들며 실제 image/mask PNG admission은 기존 어댑터에서 다시 수행한다. 읽기만으로 인증 review/Jobs 완료를 만들지 않는다. no-overwrite 발행과 제품 SDK/CLI 재가져오기 연결은 아직 후속이다.

R243 private overlay의 공통 application 기반이다. 기존 출력 asset-bundle/1을 교체하지 않고 `CapturedAssetProject::capture_sources`가 원 프로젝트 carrier와 image/mask admission lease를 보유한다. 원 PNG가 없는 adapter는 Unsupported이며 decoded pixels를 재인코딩해 원본으로 대체하지 않는다. Native 고정 PNG 입력은 해시 검사한 원 encoded bytes를 그대로 보존한다.

모든 available frame의 image/mask를 full pin·역할별로 admission한다. 같은 full pin의 admission만 재사용하고, 최종 파일은 실제 SHA별로 중복 제거한다. source-{sha}.png·정본 project.json·sources.json을 borrowed file view로 제공한다. `asset-sources/1`은 frame별 image/mask pin, 원 project publication pin, 원본 descriptor, 명시 missing frame 목록과 quality_approved=false/review_admitted=false를 보존한다. 프로젝트 안의 원 review 참조는 유지하지만 인증된 review receipt나 사용자 승인을 생성하지 않는다.

누락 frame과 삭제된 frame의 역사 source slot은 project.json에 그대로 남으며 해당 바이트가 포함됐다고 주장하지 않는다. 현재 available frame의 재편집 입력 묶음이며 모든 역사/누락 입력까지 self-contained인 전체 백업과 다르다. 원 lease와 project를 해제하지 않은 채 decoded/original/metadata 보유 bytes를 512MiB 한도로 계산한다. 기존 렌더 출력 builder도 새로 보유하는 원 encoded PNG bytes를 예산에 포함한다. hard RSS 보장이나 대규모 sequence 성능 인수는 아니다.

파일 no-overwrite 발행·읽기/admission·재가져오기·원 Jobs/SDKCLI 노출과 기존 render bundle과의 원자적 통합은 후속이다. 신규 carrier만으로 전체 source archive 기능을 완료로 표시하지 않는다. 원본 증거/소유권/WIP와 추론/다운로드/수동 GUI/push 보류를 유지한다. 기존 modootoday image-postprocess·QA fixture/변경 검증의 원본 불변/핀 고정/출력 재검증을 적용하며 정확한 출처·SHA는 [자산 입력 근거](../../../verification/application/asset-input-admission-20261029.md)에 연결한다.
