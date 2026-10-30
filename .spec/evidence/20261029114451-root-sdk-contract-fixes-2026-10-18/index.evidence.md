---
id: evidence:kinekino/20261029114451
title: SDK 계약 수정의 root 집중 재검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SDK 계약 수정의 root 집중 재검증

2026-10-18. [SDK 담당 수정·검증](sdk-contract-fixes-2026-10-18.md)에서 보고한 전체76tests/fmt/Clippy와 별도로 root가 새 CLI의 identity 집중 회귀를 실제 실행했다. 전체76개를 root가 다시 수행했다고 표시하지 않는다. root의 이전 전체73개 검사는 [당시 회귀 관측](root-integration-regression-2026-10-18.md)에 남긴다.

확인한 CLI는 `runtime/parallel-build/sdk-j2/debug/player-cli.exe`, SHA256 `b8eb54bcae105720656f26692f173fffb1f0ce050d54d968c356e6249eabf944`로 담당 인계와 일치했다. 해당 target의 `cli_request_identity-14db16c449ae13ec.exe`에서 정확한 `actual_cli_preserves_valid_unicode_context_errors_and_never_reflects_invalid_input` 테스트를 선택해1test/9case가 통과했다. 최초 필터 이름 오류의0선택 결과를 통과 증거로 사용하지 않고 정확한 이름으로 다시 실행했다.

SDK_REQUEST_ID_EVIDENCE_DIR은 이 테스트를 시작한 shell/child process에만 지정했으며 사용자 전역 환경을 변경하지 않았다. [root 원시 관측 manifest](../../../evidence/player/sdk-contract-fixes-root-2026-10-18-r1/manifest.json)와 각 request/stdout/stderr를 새 폴더에 create_new로 저장했다. manifest SHA256은 `2a513c20af509420397c364656bd7c2b3dca53c06b5295da488d9ddbd71e3b12`이며 담당의 관측 manifest와도 같았다. 실제 실행은1.88초였지만 다른 실행 시간과 속도를 비교하지 않는다.

검사 범위는 Unicode128codepoints/384UTF8bytes ID 허용, invalid API version의 유효 correlation ID 보존, unsupported의 exit3,129codepoints·빈 값·제어 ID의 cli fallback, 가짜 secret payload와 invalid raw JSON/unknown field의 비반사다. 동일 요청의 SDK/CLI Response 일치·UTF8 단일 envelope·빈 stderr도 Rust 테스트에서 검사했다. 실제 사람의 secret이나 모델·GUI·미디어 변환은 사용하지 않았다.

에러12종의 schema 중앙 목록·Unicode 길이/제어문자 정책·full Request help와 실제 지원5개/미지원47개의 상세는 담당 수정 정본을 따른다. 이번 수정을 이유로 GUI frame 연결·legacy playback 공통 facade 이관·AI/jobs/models/project 지원 또는 전체 UI 시각 인수를 완료로 표시하지 않는다. 이후 source 변경은 새 검증 범위다.
