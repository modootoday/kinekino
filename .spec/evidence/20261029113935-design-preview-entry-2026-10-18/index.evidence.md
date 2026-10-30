---
id: evidence:kinekino/20261029113935
title: Rust 디자인 미리보기 진입점 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Rust 디자인 미리보기 진입점 검증

기준일 2026-10-18. 실제 UI 제작 중인 중간 검증이다. 화면별 디자인 승인 및 제품 기능 완료를 의미하지 않는다.

## 실행 계약

`player.exe --ui-demo`는 application의 `DesktopMode::DesignPreview`를 선택하고 main에서 `presentation::run_preview`를 호출한다. 일반 플레이어의 브라우저·mpv·네이티브 파일 선택기·SDK writer 조립보다 먼저 분기한다. 옵션이 없으면 일반 플레이어 경로를 유지한다. `--ui-demo`와 `--smoke-test` 또는 `--start-url=...`의 혼합은 오류다.

위치 지정은 기존 `--record-validation --validation-position=x,y` 계약을 따른다. 유한 좌표 및 범위 검사는 application에서 수행한다. UI 모듈은 렌더링과 입력을 담당한다.

## 공통 읽기 모델

application의 `workspace_read` 포트는 readonly snapshot을 공급한다. 미리보기는 `PreviewWorkspaceProvider`의 Empty/Populated/LongLabels/NeedsInput 시나리오와 DesignSample 출처를 사용한다. 일반 미구현 데이터는 Live/Unsupported 상태다. 샘플 행이나 진행률을 실제 AI 작업 성공으로 해석하지 않는다. 시간은 공통 SDK의 정수 마이크로초 `DecimalU64`다.

렌더링은 실제 StudioShell·공통 디자인 컴포넌트를 재사용한다. 샘플 자산은 asset_id를 UI의 고정 내장 allowlist에 매핑한다. 현재 자산과 시나리오는 [데모 계약](../../player/presentation/shared/design-preview.md)을 따른다. 초기 landscape 참조는 이후 사용자가 요청한 PNG 데모 자산으로 교체됐으며 임의 URL·파일 로딩 권한을 부여하지 않는다.

## 관측과 잔여 검증

진입 옵션 선택·혼합 거절과 좌표 정책의 application 단위 시험 2개가 통과했다. 이후 root가 전체 `cargo test -j2 --target-dir runtime/parallel-build/sdk -- --include-ignored --test-threads=1`를 실행해 lib43·CLI2·CLI계약2·프레임6·자막10, 총63개 통과를 확인했다. `cargo fmt --check`와 `cargo clippy -j2 --target-dir runtime/parallel-build/sdk --all-targets -- -D warnings`도 통과했다. 이 수치는 해당 시점 소스의 결과이며 UI 세션의 이후 변경과 실제 창 시각 검증은 별개다.

남은 검증은 최신 전체 fmt/test/clippy, 각 시나리오·탭의 실제 렌더링, 디자인 기준 자산과의 대조, 일반 플레이어 실행 회귀 및 키보드 접근성이다. 이 문서는 진입점과 샘플/실데이터 경계만 담당한다.

추가 통합 확인: 페이지별 typed table/inspector 확장과 긴 표 너비 회귀 시험 반영 후 root의 동일 전체 명령을 다시 실행했다. lib44·CLI2·CLI계약2·프레임6·자막10, 총64개 통과 및 fmt/Clippy 통과를 확인했다. UI 세션은 실제 데모 렌더 중 일본어 글리프 누락과 프레임 비율 문제를 발견해 수정하고 있다. 소스 검사 통과와 수정 후 실제 창 재확인은 구별한다.

PNG 데모 자산·페이지별 asset ID 매핑·egui_extras image/png 기능 반영 후에도 root의 전체64시험(include-ignored)·fmt·Clippy가 재통과했다. UI 담당은 별도 실제 창에서 일본어 글리프와 투명 PNG의 checkerboard 합성을 확인했다고 인계했다. root의 자동 검사와 해당 수동 관측의 주체를 구분하고 모든12화면·상태의 정밀 디자인 일치 완료로 확대하지 않는다.

후속 media rail/settings Esc/Canvas readonly 연결 중 root가 unused icon·색상 경고로 Clippy 실패를 관측해 담당에게 인계했다. UI의 실제 컴포넌트 연결로 해소한 뒤 root가 같은 명령으로 lib49·CLI2·CLI계약2·프레임6·자막10, 총69시험(include-ignored)·fmt·Clippy 재통과를 확인했다. 경고 억제로 우회하지 않았다. 최신 사용자 우선순위는 전체 선택 시안의 요소 대조·설계 정본화이며 이 자동 검사로 화면 전체의 디자인 일치를 승인하지 않는다.

공용 토큰·typed A2D projection/선택 좌표·CLI help 수정 후 root가 동일 include-ignored 명령으로 lib51·CLI2·CLI계약3·프레임6·자막10, 총72시험과 fmt/Clippy 통과를 확인했다. 이 결과는 해당 sdk target 빌드 시점의 관측이다. UI 세션이 이후13화면 전체의 tab/tool/inspector 위치 대조를 계속하고 있으므로 이후 수정의 최종 인수나 모든 시안의 시각 일치를 뜻하지 않는다. 추가 source 변경은 해당 시점 이후 별도 검증한다.
