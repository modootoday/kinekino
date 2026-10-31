---
id: adr:kinekino/20261029201516
title: "ADR 0010: GitHub Actions CI/CD — specfold 방식(수동 CI, 태그 릴리스, 환경 승인, main:/docs Pages)"
status: active
created: 2026-10-30
updated: 2026-10-30
rel:
  governs: [.github]
---

# ADR 0010: GitHub Actions CI/CD — specfold 방식(수동 CI, 태그 릴리스, 환경 승인, main:/docs Pages)

## Context
- 2026-10-30 사용자 지시: 빌드·배포 CI/CD는 specfold와 같은 방식으로 GitHub Actions를 쓰고, 환경 구성과 배포 구성까지 갖춘다.
- 기존 `ci.yml`은 push/PR마다 돌고, 액션이 태그(`@v4`)로 고정돼 있었고, 실행된 적이 없다(plan 항목 #37).
- specfold 기준(읽기 전용 조사, `modootoday/specfold` main `fb8763c`): 아래 표.

| 항목 | specfold | kinekino 결정 | 근거 |
|---|---|---|---|
| CI 트리거 | `workflow_dispatch`만(push/PR 없음) | 같음 | 사용자 지시(specfold와 동일). push/PR 추가는 한 줄 변경이며 사용자 결정 항목 |
| CI 매트릭스 | ubuntu-24.04, -arm, macos-15, macos-15-intel, windows-2022 테스트 | Linux 2·macOS 2는 테스트, Windows는 clippy 컴파일 게이트만 | Windows 실행 검증은 P8(마지막) |
| 정적 검사 | fmt, clippy `-D warnings` | + `xtask lint-structure`, `xtask spec-check` | 저장소 공통 완료 기준 |
| spec 설치 | 자체 composite action(체크섬 검증) | 같은 방식 스크립트 `.github/scripts/install-specfold.sh`, `v0.2.6` 고정 | v0.2.6 태그에는 `action.yml`이 없음 |
| 캐시 | 없음 | `Swatinem/rust-cache` v2.9.3(main에서만 저장), 릴리스는 캐시 없음 | 워크스페이스 23개 crate. 캐시 오염 방지 업계 관행 |
| 툴체인 | `stable` 설치 | `rust-toolchain.toml`(1.99.1) 그대로 | 재현성 |
| target 디렉터리 | 기본 | `CARGO_TARGET_DIR=target`, `CARGO_BUILD_JOBS=4` | 개발 장비용 공유 `../.cargo-target`, `jobs=1`은 러너에 맞지 않음 |
| 릴리스 트리거 | `v*` 태그 + 수동 dry-run/bootstrap | `v*` 태그 + 수동 dry-run | bootstrap은 npm 신뢰 게시자 등록 전 첫 게시용이라 npm이 없는 kinekino엔 해당 없음 |
| 버전 규칙 | 루트 `version` = 태그 | `[workspace.package] version` = 태그(불일치 시 실패) | 같음 |
| 산출물 | `specfold-<target>.tar.gz`/`.zip` + `.sha256`, 5 타깃, Linux는 ubuntu-22.04(glibc 2.35) | `kinekino-<target>` 같은 형식·타깃, `kinekino`·`player`·`kinekino-vad-worker` + LICENSE(MIT)·README·THIRD_PARTY | 같음 |
| 게시 | 초안 릴리스 → `crates`·`npm` 환경 승인 → 게시 | 초안 → 5 타깃 업로드 성공 시 게시(환경 없음) | specfold에서도 GitHub 릴리스 게시(`finish`)에는 환경이 없고, 환경은 레지스트리 게시 잡에만 있다 |
| 레지스트리 인증 | Trusted Publishing(OIDC), 저장 토큰 없음 | 해당 없음 | - |
| 서명 | 없음 | 계획(아래 Secrets) | 설치형 패키지와 함께 |
| Pages | 워크플로 없음, `main:/docs` + `github-pages ~> 232` | 같음. CI `site` 잡은 `docs/Gemfile`이 있을 때 빌드만 확인 | specfold와 동일, 배포 워크플로 불필요 |
| 권한·고정 | 최상위 `contents: read`, 잡별 승격, 액션 SHA 고정, `persist-credentials: false` | 같음 | 최소 권한 |
| 의존성 갱신 | 없음 | Dependabot(cargo, github-actions) 주간 묶음 | SHA 고정 액션 갱신 경로 |

## Decision
1. 파일: `.github/workflows/ci.yml`(lint, windows-gate, test, site), `.github/workflows/release.yml`(draft, binaries, finish), `.github/dependabot.yml`, `.github/scripts/*.sh`, `.github/RELEASING.md`.
2. 포크 PR: CI가 수동 실행뿐이라 포크 코드가 시크릿과 함께 도는 경로가 없다. `pull_request_target`은 쓰지 않는다. PR 트리거를 켜면 `pull_request`만 쓴다.
3. GitHub Environments: specfold 규칙을 그대로 따른다(2026-10-30 사용자 지시로 `release` 환경 폐기). 규칙은 "레지스트리 하나당 환경 하나, 이름은 레지스트리 이름(`crates`, `npm`), 필수 검토자, 태그 규칙 `v*`, 해당 레지스트리에 게시하는 잡에만 `github.event_name == 'push'`일 때 붙임, 인증은 Trusted Publishing(OIDC)"이다. kinekino는 현재 게시할 레지스트리가 없으므로 환경은 **0개**다.
   - `crates`: 해당 없음. 앱 crate는 `publish` 대상이 아니다. 라이브러리 crate를 공개하면 specfold와 같은 `crates` 잡·환경을 추가한다.
   - `npm`: 해당 없음. npm 런처 패키지가 없다. 따라서 `bootstrap` 모드와 `npm/release/*`도 두지 않는다.
   - GitHub 릴리스 게시: specfold의 `finish`처럼 환경 없이 진행한다. 사람의 결정은 태그 push 자체다.
   - 앞으로 winget·Homebrew 등 배포 채널을 추가하면 같은 규칙으로 그 이름의 환경을 만든다. 서명 secrets는 서명 단계 도입 시 이 규칙과 맞춰 별도 ADR로 정한다.
4. 필요한 secrets/variables: 현재 **없음**(`GITHUB_TOKEN`만). 서명 단계(계획)에서 필요한 이름:

| 이름 | 종류 | 용도 |
|---|---|---|
| `APPLE_CERTIFICATE_P12`, `APPLE_CERTIFICATE_PASSWORD` | secret | macOS Developer ID 서명 |
| `APPLE_ID`, `APPLE_TEAM_ID`, `APPLE_APP_PASSWORD` | secret | notarytool 공증 |
| `AZURE_TENANT_ID`, `AZURE_CLIENT_ID` | variable | Windows Artifact Signing(OIDC, 저장 비밀 없음) |
| `AZURE_SIGNING_ACCOUNT`, `AZURE_CERT_PROFILE` | variable | 서명 계정·인증서 프로필 |

5. 설치형 패키지(MSI/DMG/deb·AppImage)와 서명은 P6/P8 이후 별도 단계에서 추가한다. 지금은 압축 파일과 체크섬만.
6. 설정 페이지 항목: 없음. CI/CD는 저장소 관리자 설정(GitHub 저장소 Settings)이며 앱 사용자 설정이 아니다.
7. 개발 장비 규칙과의 차이: 개발 장비는 `nice` + `-j 1` + 공유 target, 러너는 `-j 4` + 저장소 내 target + rust-cache.

## Consequences
- 로컬 검증: actionlint 1.7.12 + shellcheck 0.11.0 오류 0, 스크립트 로컬 실행(버전 추출, specfold 설치·체크섬, 패키징, Windows 게이트 clippy).
- 미검증: GitHub에서 한 번도 실행하지 않음(push 승인 대기). macOS 테스트, Windows 네이티브 clippy, 릴리스 빌드, `site` 잡.
- 승인 필요: push, remote 추가(저장소 `modootoday/kinekino`는 빈 상태로 존재), 환경 생성 없음(레지스트리 추가 시 그때 승인), Pages 활성화(Settings → Pages → main `/docs`), DNS, 서명 시크릿 등록.
