---
id: plan:kinekino/20261029113601
title: 병렬 작업 배분
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 병렬 작업 배분

2026-10-19 현재 운영 정본은 [세션 조율 계획](session-coordination.md)이다. 아래 단계별 기록은 당시 관측과 배정 이력이며 최신 소유권·goal·자원 정책을 대체하지 않는다. 중앙 세션은 PM으로 사용자 소통·의존성·자원·통합 검증을 관리하고, 제품 구현은 SDK/UI에 배분한다. 연구 담당에는 기존 연구 범위 외에 명시적으로 engine-lab/model-spike 기술 후속을 이관했다. UI 변환/HMR 독립 세션도 추가됐다. 제품 native target의 gate는 SDK/UI 간 한 소유자가 실행하고 인계하며, 아래 과거 병렬도2/별도target 안내를 근거로 중복 실행하지 않는다.

최신 통합 상태: SDK/CLI의 실제 frame capture와 Sintel52초 PNG를 구현·검증했고 원래 세션에서도 저장 이미지/응답 및 capability를 확인했다. UI는 사용자 직접 최신 지시에 따라 .spec 디자인은 유지하면서 참조 프로젝트의 egui_taffy0.14/Flexbox 패키지·구조를 적용 중이다. 해당 버전이 egui0.36.2를 요구하므로 Cargo 담당이 eframe/egui_extras를 함께 정렬했고 presentation API 이관은 UI 담당이 진행한다. 이관 중 컴파일 오류를 기존 green 결과로 덮지 않는다. 연구는 새 Rust Qwen ASR/align runtime을 고정 source로 재조사 중이다.

상태: 진행 중. 2026-10-18 사용자가 별도 세션으로 독립 업무 분산을 요청했다.

## 담당 세션

| 세션 | 식별자 | 독점 수정 범위 |
| --- | --- | --- |
| 공통 Rust SDK와 AI CLI 구현 | `01a14cc4-6b4d-77a3-a5b7-3e9fcd84b154` | 공통 API, CLI, Cargo 의존성과 실행 조립 |
| 승인된 네이티브 Rust UI 구현 | `01a14cc4-764b-71a1-b507-aead216ba652` | `src/presentation/**`, 해당 검증 문서 |
| 후속 AI 모델과 Rust 도구 조사 | `01a14cc4-818d-7b40-82b8-624da5700997` | 새 `research-followups/**`, 계획 008 |
| 원래 세션 | 현재 대화 | 모델 추론, 실측 증거 보존, 검증 결과 문서, 통합 확인 |

## 충돌 및 자원 관리

모두 같은 로컬 작업 디렉토리를 사용한다. 추론과 모델 다운로드는 원래 세션만 수행한다. UI 세션은 Cargo와 실행 조립을 수정하지 않고 필요한 의존성을 `runtime/coordination/ui-dependencies.md`에 기록한다. SDK와 UI는 각각 별도 target 디렉토리와 빌드 병렬도 2를 사용한다.

인계 상태는 `runtime/coordination/{sdk,ui,research}.md`에 기록한다. 세션 생성은 작업 완료를 의미하지 않는다. 통합 시 변경 충돌, 공통 API 연결, fmt/test/clippy 및 실행 검증 결과를 다시 확인해야 한다. 미구현 기능을 성공 상태로 표현하지 않는다.

Pages 갱신, VM, DPI와 새 장시간 테스트는 범위 밖이다. ChatGPT와 Terminal을 포함한 사용자 앱을 종료하지 않는다. Rust 통일, 단일책임과 계층 구조 규칙은 모든 세션에 적용된다.

## 지속 목표 지시

사용자의 후속 요청으로 세 세션 모두 `/goal` 형식의 명시적 목표와 완료 기준을 전달했다. 추가로 각 세션에서 `create_goal` 등록 또는 기존 `get_goal` 확인을 수행하고 인계 파일에 상태를 기록하도록 요청했다. 세 세션의 활성 상태는 확인했으며 지속 목표 등록 여부는 인계 확인 전까지 단정하지 않는다.

SDK는 discovery 첫 단계 이후 실제 자막 변환 수직 기능, UI는 탐색/포커스/접근성/좁은 창 검증, 연구는 출처/고정 파일/실제 CLI와 제안 명령 대조를 계속 담당한다. 원래 세션에서 SDK 단계의 31개 테스트·fmt·Clippy를 재확인했다. 이 수치는 후속 자막 변환 구현까지 완료했다는 의미가 아니다.

원래 세션도 사용자의 명시 요청으로 `create_goal`을 등록했고 `get_goal`에서 active 상태를 확인했다. 토큰 예산은 지정하지 않았다. 목표는 실제 미디어 AI 스파이크·문서화와 병렬 결과 통합 검증이며 기존 자원·보류 범위를 유지한다. 단순 실행 성공이나 미구현 기능 목록 작성만으로 전체 목표 완료를 선언하지 않는다.

2026-10-18 후속: root는 `--ui-demo` 조립과 옵션 정책, SDK는 readonly workspace snapshot/preview provider, UI는 공유 StudioShell과 시나리오 전환을 담당한다. [진입점 중간 검증](../../sots/verification/presentation/design-preview-entry-2026-10-18.md)을 남겼다. 최신 전체 빌드·실제 렌더링은 변경 안정화 후 재검증한다. Qwen ASR Windows CPU 어댑터는 별도 SDK 범위에서 준비 중이며 root가 검증된 빌드 인계 후 고정 weight·실제 음성 추론을 단일 워커로 담당한다.

SDK의 실제 자막 변환 및 공통 클라이언트 주입 단계가 완료돼 원래 세션에서 26cue 실제 corpus를 독립 검증했다. UI 세션은 주입된 클라이언트를 통한 실제 변환·키보드 실기를 진행 중이다. 원래 세션은 기존 browser 인프라의 생성 focus/visible→hidden 포커스 반환을 보완했고 테스트47개·fmt·Clippy를 통과했지만 실제 키 입력 해결은 UI 재검증 전까지 미확정이다. 연구 세션은 후속 provenance/license 조사 목표를 완료했고 재배포 권한 불명 항목은 별도 게이트로 남겼다.
