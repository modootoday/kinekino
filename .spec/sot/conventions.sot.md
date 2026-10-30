---
id: sot:kinekino/conventions
title: kinekino 설계 계층·4원칙·네이밍 컨벤션
status: active
verify: cargo xtask lint-structure
created: 2026-10-29
updated: 2026-10-30
rel:
  governs: [crates]
---

# kinekino 설계 계층·4원칙·네이밍 컨벤션

## Rule

kinekino의 모든 코드·디렉토리·이름에 적용하는 정본이다. 우선순위는 L0(Rust/Cargo 표준) > L1(4원칙: 단일책임·플랫파일나열금지·모듈화·맥락기반 디렉토리) > L2(네이밍: 환경변수 `KINEKINO_*`, 바이너리 `kinekino`) > L3(AGENTS.md 운영 규칙)이며, 충돌하면 표준이 이긴다.

아래 본문은 [[plan:kinekino/20261029135636]] §0–3에서 옮겼다(그 계획에는 이 문서를 가리키는 링크만 남김).

### 0. 설계 계층 (적용 우선순위)

```
L0  Rust/Cargo 공식 표준      ← 충돌 시 항상 우선
L1  4대 원칙 (SRP·플랫금지·모듈화·맥락 디렉토리)
L2  kinekino 네이밍 컨벤션
L3  프로젝트 운영 규칙 (AGENTS.md: CLI 우선, AI 품질 승인, 보류 범위)
```
위 계층 규칙과 충돌하는 아래 계층 규칙은 **예외로 명시**하고 L0을 따른다 (§1.3 충돌표).

---

### 1. L0 — Rust 표준 골격

#### 1.1 기준 문서
- Cargo Book: Package Layout, Workspaces(virtual manifest, `workspace.package`/`workspace.dependencies`/`workspace.lints` 상속), Features, Target auto-discovery
- Rust Reference: 모듈 경로 규칙 (`foo/mod.rs` 또는 `foo.rs` + `foo/`)
- Rust API Guidelines C-CASE (RFC 430), C-CONV(`as_`/`to_`/`into_`), C-GETTER(`get_` 접두 없음), C-ITER, C-FEATURE(`use-`/`with-` 접두 금지)
- rustfmt 기본값, clippy (`-D warnings`)

#### 1.2 워크스페이스 골격 (표준 그대로)
```
kinekino/
├── Cargo.toml            # virtual manifest: [workspace] members=["crates/*", "xtask"], resolver="3"
├── Cargo.lock            # 바이너리 포함 워크스페이스 → 커밋
├── rust-toolchain.toml   # 1.99.1 유지
├── .cargo/config.toml    # target-dir=../.cargo-target (공유), alias xtask
├── crates/<crate>/
│   ├── Cargo.toml        # version.workspace=true, lints.workspace=true
│   ├── src/lib.rs | src/main.rs
│   ├── src/bin/<name>.rs # 보조 바이너리 (여러 파일이면 src/bin/<name>/main.rs)
│   ├── tests/            # 통합 테스트 (크레이트 공개 API만 사용)
│   ├── benches/  examples/
├── xtask/                # 개발 자동화 (cargo xtask 관례; Rust-only 원칙과 일치)
├── vendor/meari/         # 외부 vendored, workspace exclude
├── assets/  docs/  runtime/(git 제외)
```
- 루트 `[package]`(현 `kinekino-player`, 1,321 .rs / 253,140줄 [사실])는 **없앤다** → virtual manifest. 근거: 루트 lib 하나가 GUI·CLI·OS 어댑터를 모두 가져 Linux check 실패 9건의 원인([사실], baseline).
- 모든 크레이트 공통 메타·의존 버전은 `[workspace.package]`, `[workspace.dependencies]`, 린트는 `[workspace.lints]`.

#### 1.3 표준 vs 원칙 충돌·예외 표
| 지점 | 표준 | 4대 원칙 요구 | 결정 |
|---|---|---|---|
| 모듈 파일 스타일 | `mod.rs`, `foo.rs+foo/` 둘 다 합법(2018+는 후자 권장 경향) | 한 스타일 일관 + 진입점 명확 | **`foo/mod.rs` 단일 스타일** 채택. 진입점이 디렉토리 안에 있어 "디렉토리=모듈" 경계가 명확하고 플랫 카운트가 단순. clippy `self_named_module_files = deny`로 강제. (열린 질문 Q2) 현재 혼용: mod.rs 132개, `x.rs + x/` 병존 283건 [사실] |
| `tests/` 직속 파일 | Cargo는 `tests/*.rs` 각각을 개별 테스트 바이너리로 자동 발견. 하위 디렉토리는 자동 발견 안 함 | 플랫 금지 | **예외**: `tests/` 직속은 "테스트 바이너리 진입점" 파일만 허용(상한 8). 실제 케이스는 `tests/<context>/mod.rs` 하위로. 또는 `tests/<context>/main.rs` + `[[test]]` 명시 |
| `src/bin/` | 파일당 바이너리 자동 발견 | 플랫 금지 | 크레이트당 bin 3개 이하. 초과 시 바이너리 크레이트 분리 |
| `src/lib.rs`/`main.rs` | 크레이트 루트 고정 이름 | 진입점 외 나열 금지 | 루트는 `mod` 선언·re-export·조립만 (≤150줄) |
| 맥락 디렉토리 vs 계층 | 표준은 무관 | 기술 계층(domain/application/infrastructure) 최상위 금지 | 계층은 **맥락 내부**에서만 `domain/ app/ adapters/`로 사용 |
| `build.rs`, `benches/`, `examples/` | 고정 이름 | — | 표준 이름 그대로 (원칙 적용 제외) |
| 금지어 `common`/`utils` | `tests/common/mod.rs`는 Cargo Book 관례 | 금지 | `tests/support/` 사용 (현 `packages/media-vad-protocol/tests/common` 1건 [사실] 개명) |
| 크레이트 이름 kebab vs 경로 | 크레이트 `kinekino-core` → 코드에선 `kinekino_core` | — | 표준 자동 변환 그대로 |

---

### 2. L1 — 4대 원칙의 구체 규칙 (검증 가능한 lint 기준)

#### 2.1 단일책임원칙 (SRP)
- R1. 파일 하나 = 타입 하나 또는 하나의 책임(한 문장으로 설명 가능). 파일 상단 `//!` 첫 줄에 책임 한 줄 필수.
- R2. **파일당 줄 수 상한: 600줄(경고 400)**, 테스트 파일 800줄. 현재 800줄 초과 28개, 500줄 초과 93개 / 1,804 .rs [사실]. 최대 `assembly/meari/viewer_native_probe/live_retained.rs` 2,496줄.
- R3. 함수 80줄 이하 (clippy `too_many_lines` threshold=80), 인자 7개 이하(`too_many_arguments`).
- R4. 정책 판단·OS 이벤트 처리·UI 렌더링·프로세스 제어를 한 모듈에 섞지 않는다 (기존 규칙 유지).
- 검증: `cargo xtask lint-structure` (줄 수·`//!` 유무), clippy 설정 `clippy.toml`.

#### 2.2 플랫파일나열금지
- R5. **디렉토리당 직속 `.rs` 파일 상한 8개** (진입점 `mod.rs`/`lib.rs`/`main.rs` 제외, 경고 6개). 직속 하위 디렉토리 상한 12개.
- R6. 진입점 파일(`mod.rs`, `lib.rs`, `main.rs`)은 `mod`/`pub use`/조립만, 로직 금지(≤150줄).
- R7. 같은 접두어 파일 3개 이상(`vad_job_*.rs`, `clip_*`)이면 접두어 이름의 하위 디렉토리로 묶는다.
- R8. 테스트는 `foo/tests.rs` 혹은 `foo/tests/` (같은 맥락 안). 형제 `*_tests.rs` 금지 (현 `vad_worker_payload_tests.rs`, `activation_tests.rs` 등).
- 비코드(docs·assets)는 상한 15개.
- 현재 위반 상위 [사실]:

| 디렉토리 | 직속 파일 |
|---|---|
| docs/spec/sots/verification/application | 413 |
| docs/spec/sots/verification/presentation | 227 |
| docs/spec/plans/player | 147 |
| assets/design/icons | 62 (SVG 카탈로그: 예외 후보) |
| tests | 54 |
| src/presentation/components | 53 |
| src/infrastructure | 44 |
| src/presentation/web_ui_adapter/proposals | 38 |
| src/application/api | 31 |
| src/infrastructure/mpv · src/assembly/meari | 24 · 24 |
| src/presentation · src/domain · src/application | 22 · 21 · 21 |

#### 2.3 모듈화원칙
- R9. 크레이트 경계 = 배포·OS·의존성 경계. OS 종속 의존(wry, webview2-com, windows)은 **`kinekino-platform-*` 크레이트에만** 존재.
- R10. 기본 가시성 `pub(crate)`; 크레이트 공개 API는 `lib.rs` re-export로만. 맥락 간 접근은 맥락의 `mod.rs` 공개 항목만 (deep path `a::b::c::d` 접근 금지, 2단계까지).
- R11. 포트(trait)는 사용하는 쪽(core) 정의, 어댑터는 구현 쪽. `#[cfg(...)]`는 **platform 선택 지점 한 곳**(`kinekino-platform/src/lib.rs`)에서만. 현 `cfg(windows)` 계열 189회 [사실] → 목표 ≤10.
- R11a. (ADR 0003, adr:kinekino/20261029155443) 크로스 OS 경계 lint 5종 R-cfg·R-dep·R-ver·R-entry·R-port는 `cargo xtask lint-structure`가 검사한다. 크레이트 등급 단일 원천은 `xtask/src/structure/boundary/mod.rs` `CLASSES`. 포트 규약(`ErrorCode`·`Registry`·`drain_events`·비밀값은 `CredentialStore`)은 `kinekino-core::port`. 진입은 `main → kinekino_app::entry`.
- R12. 순환 의존 금지 (`cargo xtask lint-deps`: §4 방향 위반 검출, `cargo-deny`/`cargo metadata` 기반).

#### 2.4 맥락기반디렉토리구조
- R13. 최상위 모듈 이름은 **사용자 맥락 명사**: `playback`, `library`, `studio`, `projects`, `browser`, `sdk` … 기술 계층(`domain`, `application`, `infrastructure`, `presentation`)은 최상위 금지.
- R14. 맥락 내부 표준 하위: `domain/`(정책·값, 외부 의존 없음) · `app/`(유스케이스·포트) · `adapters/`(외부 기술). 작은 맥락은 파일 3개 이하면 계층 생략 가능.
- R15. 금지 이름: `utils`, `util`, `common`, `misc`, `helpers`, `helper`, `shared`, `stuff`, `base`, `manager`(단독). 현재 해당 경로 1건 [사실].
- R16. 현재 구조는 같은 맥락이 3계층에 흩어짐 [사실]: 예) `clip_*`가 application 2 + domain 1 + infrastructure 10개 디렉토리, `vad_*` infrastructure 직속 11개 파일+3디렉토리.

---

### 3. L2 — 네이밍 컨벤션

| 대상 | 규칙 | 예 |
|---|---|---|
| 크레이트/패키지 | `kinekino-<context>` kebab-case, lab 전용은 `kinekino-lab-<service>` | `kinekino-core`, `kinekino-lab-source` |
| 바이너리 | 제품: `kinekino`(CLI), `kinekino-player`(GUI). 별칭 `kk`는 설치 시 링크/별도 `[[bin]]` (Q3) | `kinekino.exe`, `kinekino-player.exe` |
| 디렉토리 | snake_case 명사, 맥락명. 크레이트 디렉토리만 kebab (=크레이트명) | `crates/kinekino-core/src/studio/subtitles/` |
| 모듈 | snake_case 단수 명사(컬렉션 개념은 복수 허용: `projects`) | `playback::session` |
| 타입/트레잇 | UpperCamelCase, 약어도 단어 취급(`HlsPlaylist`, `Vad`), 트레잇 포트는 역할명(`BrowserEngine`, `VideoSurface`) – `I`접두/`Trait` 접미 금지 | |
| 함수/메서드 | snake_case 동사구; `get_` 금지, 변환 `as_/to_/into_`, 생성 `new`/`with_*`/`from_*`, 실패 가능 `try_*` | `to_cue_list()` |
| 상수/static | SCREAMING_SNAKE_CASE | `MAX_SEGMENT_MS` |
| feature | kebab, 기능 명사, `use-`/`with-` 금지 | `card-review-export`, `test-support` |
| 환경변수 | `KINEKINO_<CONTEXT>_<KEY>`; lab 전용 `KINEKINO_LAB_<SERVICE>_<KEY>` | `KINEKINO_PLAYBACK_MPV_PATH`, `KINEKINO_LAB_SOURCE_REVIEW_OUTPUT_DIR` (현 `SOURCE_REVIEW_OUTPUT_DIR` 등) |
| 설정 키 (TOML) | snake_case, 맥락 테이블: `[playback] mpv_path=` | 파일 `kinekino.toml` |
| 제품 식별 | **단 한 곳**: `crates/kinekino-core/src/product/identity.rs` (`PRODUCT_ID="kinekino"`, `DISPLAY_NAME_KO="키네키노"`, `DISPLAY_NAME_EN="Kinekino"`, `BUNDLE_ID="com.kinekino.app"`). 표시명은 다른 코드에 리터럴 금지 (grep lint) | 현 `src/domain/product.rs` 계승 [추정] |
| 프로토콜/데이터 ID 접두어 | 역도메인 `com.kinekino.<context>.<kind>.v<N>`; 표시명과 무관하게 고정 | `com.kinekino.projects.manifest.v1` (현 `kinekino.player.proposals` 등 → 마이그레이션) |
| 문서 | kebab-case.md; spec `docs/specs/<context>/<topic>.md`; ADR `docs/adr/NNNN-kebab-title.md`(4자리 일련, 날짜는 front-matter); plan `docs/plans/<context>/<topic>.md` | `docs/adr/0001-browser-engine-port.md` |
| 테스트 함수 | `<행동>_<조건>_<기대>` snake | `import_cancelled_keeps_source` |
| 브랜치 | `<type>/<context>-<topic>`; type ∈ feat, fix, refactor, docs, chore, spike, lab | `refactor/core-playback-split` |
| 커밋 | Conventional Commits `<type>(<crate|context>): <요약>`; 본문에 검증 결과·미검증 구분(기존 규칙 계승) | `refactor(kinekino-browser): extract BrowserEngine port` |
| 금지 문자열 | `oink`, `오잉크`, (lab 밖) `source` — CI grep | |

---


## Why

사용자 지시(2026-10-29): 4원칙을 Rust/Cargo 표준 위에 적용하고, 충돌 시 표준을 우선한다. 이름 규칙은 한 곳에서만 정의해 중복을 막는다.

## Verification

`cargo xtask lint-structure`가 플랫 나열·`x.rs`/`x/` 공존·금지 이름과 경계 규칙 R-cfg/R-dep/R-ver/R-entry/R-port(신규 위반 0)를 검사한다. 네이밍(L2)과 SRP 일부는 사람이 검토한다.
