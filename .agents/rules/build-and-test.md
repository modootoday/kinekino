---
paths:
  - "**/*.rs"
  - "**/Cargo.toml"
  - "**/Cargo.lock"
---
# 빌드·테스트 [사용자 결정 2026-11-03 전달, 근거 문서 없음; 환경 스크립트 .github/scripts/dev-env.sh]
- 정확한 경로·명령 등 로컬 환경 값은 `.agents/local/build-env.md`(추적 안 함)를 본다.
- 먼저 툴체인 환경과 `.github/scripts/dev-env.sh`를 불러온다.
- 항상 공유 빌드 락(`flock`) 아래 `nice -n 10 cargo <cmd> -j 1`. 한 번에 한 빌드. 공유 `CARGO_TARGET_DIR` 캐시를 쓴다.
- `cargo clean` 금지. 공유 폰트 캐시 삭제 금지.
- `KINEKINO_LIBMPV`·`KINEKINO_MPV`는 로컬에 설치된 libmpv·mpv를 가리킨다.
- TMPDIR은 짧은 /tmp 경로, 같은 날 정리. 오래 둘 것은 로컬 작업 영역 아래.
- 게이트: 실패 테스트 먼저 → 전체 `cargo test` → `cargo fmt --check` → `cargo clippy --all-targets -- -D warnings`(기존 경고 수 증가 금지) → `spec check`. [AGENTS.md]
