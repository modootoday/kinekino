---
id: sot:kinekino/identity
title: 변경 가능한 제품 식별
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 변경 가능한 제품 식별

상태: 사용자 요구 확정, 이관 설계. Source Player는 가제이며 표시명 확정이 아니다. 배포 실행 파일 요구는 `player.exe`, `player-cli.exe`다.

순수 값 `ProductIdentity`는 stable_product_id, display_name, executable_names, publisher, website/support metadata, config_schema_version을 구분한다. 표시명은 한 제품 메타데이터 원천에서 주입하고 GUI 제목·About·CLI help/version·설치 메타데이터가 참조한다. domain ID·작업 상태·오류 code·공개 API namespace는 표시명으로 생성하지 않는다. URL 연동 source provider 이름은 소스 식별이며 제품 표시명과 다르다.

OS 저장 위치·AppUserModelID·mutex/pipe 이름·registry·설치 upgrade identity는 infrastructure가 stable_product_id로 결정한다. 공통 SDK DTO에 브랜드 문자열을 필수 prefix로 넣지 않는다. 모델/다운로드 캐시는 프로젝트 표시명과 독립한 식별과 검증된 hash로 관리한다. 기존 데이터 경로의 `source`를 즉시 일괄 문자열 치환하지 않는다.

이름 확정 또는 변경 시 identity manifest의 표시값을 바꾸고 모든 GUI/CLI/인스톨러 출력의 하드코딩 잔여를 검사한다. OS stable ID를 바꿔야 한다면 기존 설정·프로젝트·작업 이력·캐시를 백업→구버전 발견→검증→한 번만 이관→새 위치 확정하는 마이그레이션이 필요하다. 기존 SDK import crate 명칭 변경은 SemVer/호환 alias 검토 대상으로 별도 계획한다. 사용자 생성 문서 안의 과거 제품명·조사 증거는 기록으로 보존한다.

2026-10-26 소스 대조 정정: `src/domain/product.rs`의 ProductIdentity/PRODUCT에 stable_product_id·display_name·gui_executable·cli_executable·config_schema_version이 존재하고, Cargo.toml은 player/player-cli 두 bin을 선언한다. CLI runner·GUI·local endpoint의 일부 소비도 소스에서 확인했다. 종전 ‘ProductIdentity 및 두 bin 미구현’ 표현은 초기 상태이며 현재 판정으로 사용하지 않는다. package/crate 이름 kinekino-player는 표시 이름과 별개다. publisher/support metadata·installer/upgrade identity·전체 하드코딩 제거·설정 이관의 구현/검증은 이 소스 확인으로 완료 처리하지 않는다. 새 호스트 정상 bins 실행은 미실행이다. [추적 R01](../../../../plans/player/design-completion/requirements.md)의 후속 범위를 따른다.
