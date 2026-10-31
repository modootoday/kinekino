---
id: adr:kinekino/20261030200000
title: "ADR 0011: 미디어 소스 플러그인 인프라와 source 비공개 분리"
status: active
created: 2026-10-30
updated: 2026-10-30
rel:
  governs: [crates]
---

# ADR 0011: 미디어 소스 플러그인 인프라와 source 비공개 분리

## Context
- 사용자 제안(2026-10-30): source 로직을 플러그인으로 바꿔 플러그인 인프라를 만들고 source는 별도 비공개 저장소로 분리한다. 공개 저장소 이력에도 흔적이 없어야 한다(아직 push 전, `modootoday/kinekino`는 빈 저장소).
- 결합 조사(main `54bf73c`, rg 기준): 작업 트리 source 언급 약 1260회. crates 80개 파일(테스트 제외 43), `.spec` 179개 문서, tools 다수. 이력 전체 집계(2026-10-30 05:55 KST, 모든 ref): 커밋 111개 전부가 source/media-cdn/frame-host 문자열을 포함, 해당 문자열을 담은 고유 경로 818개, 커밋 메시지 언급 0건, ref 이름 0건, 태그·stash 0.

| 영역 | 위치 | 결합도 |
| --- | --- | --- |
| 카탈로그 크레이트 | `kinekino-catalog`(1502줄): `Category`가 `https://source.example.test/{방영중,종영,극장판}` 고정, `PageLocation` 호스트 검증, scraper 파서, reqwest 전송 | 크레이트 전체가 source 전용 |
| player 응용 | `application/catalog`, `episode_resolve`, `browser_provider/catalog_jobs`, `browser_network/document_scopes`, `infrastructure/{catalog,episode_document}` | `kinekino_catalog` 타입 직접 사용(16개 파일) |
| 공개 API·스키마 | `api/catalog.rs` `series_url` 설명, `project_schema.rs` `episode_url` 설명 | 스키마 문자열에 도메인 노출 |
| 재생 정책 | `domain/playback.rs` 허용 호스트 `media.example.test`, `frame.example.test`, `source.example.test` | 하드코딩 허용목록(스트림 URL 추출이 브라우저 캡처 기반) |
| 시작·UI | `launch.rs` `--start-url=https://source.example.test/방영중`, `presentation/workspace.rs` 카테고리 이동 | 기본 진입점 |
| 플러그인 id | `com.kinekino.player.proposals`, `com.kinekino.player.review-intents` | 일반 Meari 플러그인이지만 이름에 흔적 |
| 테스트·fixture | 37개 테스트 파일, `parser/category_fixture.rs`, replay fixture | 도메인 일반 기능 테스트가 source URL 사용 |
| 문서 | `.spec` 179개(evidence, sot `catalog-http-chain` 등), `tools/*` | 이력·근거 문서 |
- 대안 사이트: 사용자 결정(2026-10-30)으로 an alternative media source site은 source의 폴백이 아니라 별개 대안 사이트다. source 접속이 막히면 별도 `alternative-source` 플러그인을 만든다(후속 항목).

## Decision
1. **코어 포트 `kinekino_core::source`** (ADR 0006·0007의 프로세스 소유·IPC 위, 0008 시계·0009 창과 같은 중앙 포트 규약):
   - `SourceProvider{manifest() -> SourceManifest}`와 선택 능력 `Browse(categories, page)`, `Search(query, page)`, `Series(id) -> episodes`, `Episode(id) -> adjacency(prev/next)`, `Resolve(episode) -> StreamCandidates{url, headers, allowed_hosts, expires}`.
   - URL은 불투명 `SourceRef{provider, id}`로 바꾼다. 호스트 허용목록은 플러그인 매니페스트가 선언하고 코어 `playback` 정책은 그 값을 받는다.
   - 브라우저 캡처가 필요한 `Resolve`는 코어 `browser` 포트를 호스트 능력으로 플러그인에 위임한다(플러그인이 브라우저를 직접 소유하지 않는다).
2. **방식: 별도 프로세스 + IPC 스키마**(추천). 근거와 비교:

| 방식 | 사례 | 장점 | 단점 | 판단 |
| --- | --- | --- | --- | --- |
| 프로세스 분리+JSON 스키마 | [VS Code Extension Host](https://code.visualstudio.com/api/advanced-topics/extension-host), [Raycast](https://developers.raycast.com/information/lifecycle), [LSP](https://microsoft.github.io/language-server-protocol/)/[MCP](https://modelcontextprotocol.io/specification/2025-06-18) | 크래시 격리, 언어 무관, 독립 빌드·서명, 기존 `kinekino schema`·ADR 0007 IPC 재사용 | 프로세스 수명·버전 협상 비용, OS 샌드박스는 직접 구성 | 채택 |
| WASM(wasmtime/WIT) | [Zed 확장](https://zed.dev/blog/zed-decoded-extensions) | 강한 기본 샌드박스, 단일 아티팩트 크로스 OS | WASI HTTP·브라우저 연동 미성숙, source는 HTTP+CDP 필요, 호스트 함수 설계 비용 | 반박 근거로 기록, 신뢰 낮은 제3자 플러그인 시 재평가 |
| 동적 라이브러리 | [Jellyfin](https://jellyfin.org/docs/general/server/plugins/)(.NET DLL 인프로세스) | 빠름 | Rust ABI 불안정, 크래시 공유, OS별 빌드 3종 | 제외(ADR meari-product-plugin-deployment와 동일) |
| 스크립트(Lua/JS/Python) | [mpv 스크립트](https://mpv.io/manual/master/), [Kodi 애드온](https://kodi.wiki/view/Addon.xml) | 배포 쉬움 | 런타임 내장, 타입 계약 약함, Rust-only 원칙(ADR 007) 위배 | 제외 |
   - 반박 정리: 프로세스 분리는 샌드박스가 아니다(VS Code도 확장을 신뢰 모델로 다룬다). 그래서 신뢰는 서명·사용자 승인으로, 격리는 OS별 어댑터(Linux seccomp/landlock, macOS sandbox-exec, Windows AppContainer)로 단계 도입한다.
3. **매니페스트** `plugin.json`(2026-10-30 개정: 코드 정본을 따른다. 스키마·검증은 `kinekino-plugin-sdk::manifest`, 발견은 호스트. 원안 `kinekino-plugin.toml`은 폐기, TOML 전환은 새 결정 필요): `id`, `version`(semver), `api`(호환 범위, 예 `source/1`), `entry`(OS별 실행 파일), `capabilities`, `permissions{network_hosts, browser}`, `settings`(JSON Schema → 설정 페이지 `settings.plugins.<id>`에 자동 렌더, CLI `kinekino settings` 동일), `signature`.
4. **설치·발견**: OS 관례 경로(Linux `$XDG_DATA_HOME/kinekino/plugins`, macOS `~/Library/Application Support/kinekino/plugins`, Windows `%LOCALAPPDATA%\kinekino\plugins`)를 `OsPolicy`가 제공. 시작 시 매니페스트 스캔, `kinekino plugins list|enable|disable`. 활성화·권한 승인·소스 우선순위는 설정 페이지 항목.
5. **비공개 저장소 배포**: 비공개 저장소가 코어의 공개 `kinekino-plugin-sdk`(스키마·타입) 크레이트를 git 태그로 의존. 자체 GitHub Actions(ADR 0010과 같은 규칙)로 5개 타깃 빌드, `.sha256`과 [minisign](https://jedisct1.github.io/minisign/) 서명(확정), 비공개 릴리스로 배포. 코어는 사용자 설정의 공개키로 서명 검증.
6. **1단계 격리(확정)**: 서명 검증 + 원 님의 설치·권한 승인만. OS 샌드박스(seccomp/landlock, sandbox-exec, AppContainer)는 후속.
7. **비공개 저장소(확정)**: `modootoday/kinekino-source-catalog`(빈 저장소, push는 별도 승인).
8. **비공개 분리 기준**: 공개 저장소 모든 ref에서 `git log --all -p -i -S source`, `git grep -i -e source -e media-cdn -e frame-host -e 방영중 $(git rev-list --all)` 결과 0, 파일 경로 0, 커밋 메시지 0, 플러그인 id는 `com.kinekino.*`.

## 마이그레이션 단계
| 단계 | 내용 | 완료 기준·검증 |
| --- | --- | --- |
| M0 | 이 ADR 승인, 배치 6~8 진행 중 source 신규 결합 금지 | lint 규칙 초안 |
| M1 | 코어 `source` 포트·`SourceRef`·호스트 허용목록 주입, 계약 테스트(가짜 소스 fixture `example.test`) | 테스트 통과, 코어 비테스트 코드 source 0 |
| M2 | 테스트·fixture URL을 `example.test`로 치환, 플러그인 id 개명 | crates rg 0(테스트 포함) |
| M3 | 플러그인 호스트(ADR 0006 프로세스 소유+0007 IPC), 매니페스트·발견·설정 페이지 | `kinekino plugins list` 실제 실행, 가짜 플러그인 e2e |
| M4 | `kinekino-catalog`와 source 어댑터를 로컬 별도 저장소로 이동, 플러그인 바이너리로 동작 | 기존 카탈로그·재생 e2e 동등 |
| M5 | `.spec`·tools 문서 분리(source 근거는 비공개로), lint R-src(금지어) 추가 | spec-check 0, lint 신규 0 |
| M6 | 이력 재작성(아래), 검증 후 첫 push(사용자 승인) | 분리 기준 6 충족 |
- 순서: 현재 배치 5b·trait 분리 → M0~M2 → 배치 6~8 → M3~M5 → P9 MCP(플러그인 능력도 `kinekino schema`로 노출) → M6 → 첫 push. P9 이전에 M3를 끝내야 MCP 도구 목록이 두 번 바뀌지 않는다.
- 예상 규모: 코드 이동 약 1500줄(catalog)+player 결합 16개 파일, 테스트 치환 37개 파일, 신규 호스트·SDK 약 2000~3000줄, `.spec` 179개 정리. 집중 작업 기준 수일.

## 이력 재작성 계획(실행 안 함)
1. **검색**: 별도 미러 `git clone --mirror /workspace/kinekino /workspace/kinekino-rewrite.git`. 경로 목록 `git log --all --name-only --format= | sort -u | rg -i 'source|catalog'`, 내용 `git rev-list --all | xargs -n50 git grep -il -e source -e media-cdn -e frame-host`, 메시지 `git log --all -i --grep=source`(현재 0건), ref `git for-each-ref`, stash·worktree는 원본에서 `git stash list`, `git worktree list`. 결과를 `paths.txt`, `strings.txt`로 고정.
2. **백업**: `git bundle create kinekino-full-<날짜>.bundle --all`(원본 전체, 비공개 보관). 도구는 [git-filter-repo](https://github.com/newren/git-filter-repo) 2.47.0(개발 장비 사용자 설치 `~/.local/bin`).
3. **재작성**(미러에서만): `git filter-repo --invert-paths --paths-from-file paths.txt --replace-text strings.txt --replace-message strings.txt`(`source.example.test==>example.test`, `regex:(?i)source==>source` 등). 원본 `/workspace/kinekino`는 건드리지 않는다.
4. **검증**: 재작성 클론 checkout 후 `nice cargo test -j 1`, clippy, `cargo xtask lint-structure`, `cargo xtask spec-check`, 분리 기준 6 검색 0. 모든 브랜치·태그에 반복.
5. **비공개 보존**: 비공개 저장소는 원본 번들에서 `git filter-repo --path crates/kinekino-catalog --path <source 경로들>`로 source 이력만 추출해 시작(작성자·날짜 유지). 원본 번들은 비공개 저장소 릴리스 자산 또는 오프라인 보관.
6. **시점·재정렬**: M5 완료 후, 첫 push 직전 한 번만. 그 전에 모든 executor 작업을 main에 병합하거나 중지하고 worktree를 비운다. 남은 브랜치는 재작성 미러에 함께 재작성되므로, 재작성 후 기존 worktree는 `git worktree remove` 후 새 클론에서 다시 만든다. stash는 사전에 브랜치로 커밋하거나 폐기. reflog는 새 클론에서는 비어 있다.
7. **위험·대응**: 모든 커밋 해시 변경 → filter-repo가 남기는 `.git/filter-repo/commit-map`으로 `.spec`·보고서 안 해시를 일괄 치환(`--replace-text`에 old==>new 추가 또는 사후 스크립트). 근거 문서 일부 삭제로 링크 깨짐 → spec-check로 검출 후 비공개 문서 참조로 대체. 원본과 공개본 혼동 → 원본 remote에는 push URL을 막아 둔다.

## Consequences
- 코어는 소스 중립이 되고 source는 비공개 플러그인이 된다.

## 후속 항목
- alternative media source plugin(별도 비공개 저장소, source 접속 차단 시 필요하면 생성).
- OS 샌드박스 어댑터.
- 프로세스 경계 비용(IPC 지연, 수명 관리)을 받아들인다. 강한 샌드박스는 후속 OS 어댑터 과제다.
