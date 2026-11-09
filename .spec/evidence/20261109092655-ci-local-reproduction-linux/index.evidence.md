---
id: evidence:kinekino/20261109092655
title: "CI 로컬 재현 점검 (Linux, 2026-11-09)"
status: active
created: 2026-11-09T09:26:55Z
updated: 2026-11-09T09:26:59Z
rel:
  governs: [.github/workflows/ci.yml]
  depends_on: [adr:kinekino/20261029201516]
---

# CI 로컬 재현 점검 (Linux, 2026-11-09)

## What was measured
`.github/workflows/ci.yml`(수동 실행, ADR 0010)의 Linux 게이트 단계를 개발 호스트(x86_64 Linux, rustc 1.99.1 = `rust-toolchain.toml`)에서 같은 명령으로 돌렸다. 커밋 `1f049c9d` 기준, 2026-11-09 KST. 로컬 차이: `CARGO_BUILD_JOBS` 대신 `-j 1`, 공유 빌드 캐시, specfold는 CI 고정 `v0.2.6`이 아니라 설치된 `0.5.1`.

| CI job·단계 | 로컬 명령 | 결과 |
|---|---|---|
| lint: fmt | `cargo fmt --all --check` | 통과 |
| lint: clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 통과 |
| lint: 구조 린트 | `cargo xtask lint-structure` | 통과(위반 348, 모두 허용 목록, 신규 0) |
| lint: spec check | `cargo xtask spec-check` | 통과(경고만, 오류 0). specfold 버전 차이로 CI 결과와 같다고 보지 않는다 |
| test (ubuntu-24.04) | `cargo test --workspace --locked --no-fail-fast` | 통과: 1852 통과, 53 무시, 실패 0 |
| test (ubuntu-24.04-arm) | - | 미실행: ARM 호스트 없음 |
| site | `bundle exec jekyll build` (docs/) | 미실행: 이 호스트에 Ruby 런타임이 없음 |
| windows-gate (권고) | `.github/scripts/windows-gate.sh` | 미실행: Windows 타깃(x86_64-pc-windows-msvc) 미설치 |
| test-macos (권고) | - | 미실행: macOS 호스트 없음 |

관측 1건: 첫 테스트 실행에서 `TMPDIR`을 긴 경로(작업 트리 안)로 두었더니 `kinekino-platform-unix`의 IPC 테스트 2개가 `socket path exceeds sun_path`로 실패했다. 테스트가 `std::env::temp_dir()` 아래에 소켓을 만들기 때문이며, 짧은 `TMPDIR`(`/tmp` 아래)로 다시 돌리자 전부 통과했다. CI 러너는 `TMPDIR`을 바꾸지 않으므로 영향이 없다. 제품 결함이 아니라 테스트 환경 조건으로 기록한다(낮음).

원격 CI: `gh run list --workflow ci.yml` 결과 CI 워크플로 실행 기록이 0건이다(같은 저장소의 Dependabot 실행만 있음). 이번 점검은 원격 실행을 시작하지 않았다.

## How to read it
- 로컬 통과는 원격 녹색이 아니다. CI-1의 "GitHub 녹색 실행 기록"은 여전히 비어 있으며, 수동 실행 후 실행 ID와 결과가 생길 때만 채운다.
- 미실행 4건(ARM 테스트, site, Windows 게이트, macOS 테스트)은 실패가 아니라 이 호스트에서 재현할 수 없는 단계다. site는 Ruby 런타임을 갖추면 로컬에서 재현할 수 있다.
- spec check는 specfold 버전이 달라 규칙 집합이 다를 수 있다. CI와 같은 버전으로 확인하려면 `.github/scripts/install-specfold.sh v0.2.6 <dir>`로 받은 바이너리를 `KINEKINO_SPEC_BIN`에 지정한다.
