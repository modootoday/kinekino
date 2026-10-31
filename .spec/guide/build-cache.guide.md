---
id: guide:kinekino/build-cache
title: 빌드 캐시 규칙 (target 크기 예산)
status: active
created: 2026-10-29
updated: 2026-10-30
---
# 빌드 캐시 규칙

## 규칙
- `profile.dev`/`profile.test`는 `debug = "line-tables-only"`, 의존성(`profile.dev.package."*"`)은 `debug = false`. 풀 DWARF가 필요하면 로컬에서만 `CARGO_PROFILE_DEV_DEBUG=true`로 일시 사용하고 커밋하지 않는다.
- 증분 컴파일 끔: `.cargo/config.toml`의 `[build] incremental = false` 한 곳(Cargo 표준 키). 일회성으로 켜려면 `CARGO_INCREMENTAL=1`을 환경변수로 준다(환경변수가 설정 파일보다 우선).
- 캐시 상한: `cargo xtask clean-cache [--max-size 8G] [--target-dir DIR] [--dry-run]`. 상한을 넘으면 ① `incremental/` ② 같은 crate·종류의 옛 해시 산출물 ③ 오래된 프로파일 디렉터리 순으로 지우고 상한 안에 들면 멈춘다. 대상은 `CARGO_TARGET_DIR` 또는 공유 `../.cargo-target`.
- 빌드는 `-j 1`(config `build.jobs`)과 `nice`로 돌린다(공유 개발 장비).

## 측정 (2026-10-29, kinekino-browser `--all-targets`, 새 target, -j 1)
| | cold 빌드 | touch 후 재빌드 | target 크기 |
|---|---|---|---|
| 이전 (debug 기본값) | 108s | 4s | 1.4G |
| 이후 (line-tables-only, deps 0) | 91s | 3s | 696M |
링크·DWARF 쓰기가 줄어 빌드도 약간 빨라졌다. 트레이드오프: 의존성 내부로 디버거 스텝·변수 조회 불가, 백트레이스는 워크스페이스 코드의 파일:줄까지만.

## 공유 target (2026-10-30부터 기본)
- 모든 저장소·워크트리는 `.cargo/config.toml`의 `target-dir = "../.cargo-target"`을 쓴다. 상대 경로는 저장소 루트 기준이라 개발 장비에서는 `/workspace/.cargo-target`, Windows에서는 `<부모>\.cargo-target`이 되어 OS별 절대 경로 분리가 필요 없다. 기계별로 다른 위치가 필요하면 `CARGO_TARGET_DIR` 환경변수로 덮어쓴다(설정 파일보다 우선).
- 저장소 설정을 고른 이유: 환경변수만 쓰면 셸을 거치지 않는 에이전트 실행에서 빠지고, `~/.cargo/config.toml`의 `target-dir`은 저장소 설정에 가려진다.
- 단점: cargo 락으로 빌드가 직렬화됨(`-j 1` 방침과 일치), 한 곳의 `cargo clean`이 모두를 지움 → `cargo clean` 대신 아래 점검 스크립트와 `clean-cache`를 쓴다.
- 툴체인: `rust-toolchain.toml`이 1.99.1을 고정하고, `~/.cargo/bin`(rustup 프록시)이 PATH 앞에 있어야 한다(개발 장비 `~/.profile`, `~/.bashrc`에 설정). `/usr/bin/cargo` 1.85.1은 `[cache]` 자동 정리를 지원하지 않는다. 자동 정리 `[cache] auto-clean-frequency = "1 day"`는 개발 장비 `~/.cargo/config.toml`에 둔다.
- 정리 기준: `scripts/cargo-cache-report.sh`(읽기 전용 기본) — 크기 합계, 30GB 초과 경고, 7일 지난 deps/build/fingerprint 후보. 삭제는 `--apply`(cargo-sweep 필요)로만, 사람 승인 후.
- xtask 경로는 실행 시점에 찾는다(`xtask/src/workspace.rs`): `cargo run`이 주는 런타임 `CARGO_MANIFEST_DIR`, 없으면 현재 디렉터리에서 위로 `[workspace]` 매니페스트 탐색(`cargo locate-project --workspace`와 같은 방식). 공유 target에서는 여러 체크아웃이 같은 `debug/xtask`를 실행할 수 있어, 컴파일 시점 `env!` 경로는 다른 저장소를 검사하게 된다(2026-10-30 실제 발생: lint-structure가 다른 워크트리를 읽어 0건 보고). 회귀 테스트: `xtask/tests/runtime_root.rs`. `clean-cache`·`install`·`demo-engine`의 기본 target도 `CARGO_TARGET_DIR` → config `[build] target-dir` → `target` 순으로 읽는다.
- 2026-10-30: 옛 저장소 내부 target `runtime/build-cache/native`와 `runtime/build-cache/wincheck`는 공유 target 전환 후 원 님 승인으로 삭제했다.
- 패키지 이름은 저장소별로 고유하게 둔다(`kinekino-xtask`, meari는 `meari-xtask`). 공유 target에서 `debug/<이름>`은 마지막으로 빌드한 저장소의 바이너리로 남고 cargo는 fresh 판정 시 다시 복사하지 않아, 같은 이름이면 다른 저장소의 xtask 코드가 실행될 수 있다. alias `cargo xtask`는 그대로다.
- 2026-10-30 수정: kinekino는 `../.cargo-target/kinekino`를 쓴다(저장소 단위 분리, 같은 저장소 워크트리끼리는 계속 공유). kinekino는 meari를 `vendor/meari`로 벤더링하는데, `/workspace/meari` 워크스페이스의 `meari`와 이름·버전·기능이 같아 같은 `-C metadata`를 받는다. 한 target을 공유하면 서로의 rmeta를 덮어써 kinekino 빌드가 meari 원본 타입을 보고 실패했다(`ConvertSubtitles: DeserializeOwned` 등 22건). 저장소별 target이 Cargo 문서상 지원되는 분리 단위다. 2026-10-31: kinekino는 vendor를 없애고 crates.io의 meari 0.1.0을 쓴다(registry 소스라 path 빌드와 metadata가 달라 충돌하지 않는다).
