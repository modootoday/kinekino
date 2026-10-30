---
id: sot:kinekino/plugin-sdk-extraction
title: kinekino-plugin-sdk 추출 설계
status: active
created: 2026-10-30T06:40:19Z
updated: 2026-10-30T13:15:00Z
rel:
  governs: [crates/kinekino-source-host, crates/kinekino-plugin-sdk]
---

# kinekino-plugin-sdk 추출 설계

## Rule

SDK는 `serde`·`serde_json`에만 의존하고 kinekino 내부 크레이트에 의존하지 않는다. `publish = false`를 유지하고 배포는 git 태그 `plugin-sdk-v<semver>`로만 한다. 저장소 안의 예제·fixture는 합성 데이터만 쓰며 특정 비공개 플러그인의 이름·코드·설정을 두지 않는다.

## Why

비공개 소스 플러그인이 호스트 내부 없이 공개 계약만으로 빌드되어야 하고, 호스트와 플러그인이 같은 와이어 정의·골든 fixture를 공유해야 프로토콜 드리프트를 테스트로 잡을 수 있다.

## Verification

`cargo test -p kinekino-plugin-sdk`(골든 왕복), `cargo test -p kinekino-source-host`(같은 골든을 호스트 쪽에서 읽음), `cargo xtask lint-structure`(SDK는 Contract 등급, 신규 위반 0).
## 목적

[ADR 0011](../adr/20261030200000-adr-0011-media-source-plugin.adr.md) 결정 5에 따라 비공개 소스 플러그인 저장소가 git 태그로 의존할 공개 크레이트 `kinekino-plugin-sdk`의 경계·버저닝·분리 순서를 정한다. 2026-10-30 22:11 KST 원 님 지시로 승인(S0)되어 진행 중이다. 조사 기준은 main `5bcf812`.

## 현재 위치(관측)

| 타입·기능 | 현재 위치 | 성격 |
|---|---|---|
| `SOURCE_API = "source/1"`, `SourceRef`, `SourceCapability`, `SourceManifest`, `SourceItem`, `SourcePage`, `EpisodeAdjacency`, `StreamCandidate`, `SourceError` | `kinekino-core/src/source/mod.rs` | 도메인 값(serde 없음) |
| `SourceProvider` 트레이트 | `kinekino-core/src/source/ports.rs` | 코어 포트(호스트 쪽) |
| `document/1` 값: `CatalogPage`, `RawDocument`, `Provenance`, `CatalogError`, `EpisodeDocumentData`, `PageLocation`, `LocationRules` | `kinekino-core/src/source/document/*` | serde 값 + 호스트 규칙 설치 |
| `source/1` 와이어: `Ref`, `Request`, `Item`, `Stream`, `Response`, `serve_fixture` | `kinekino-source-host/src/plugin_host/wire.rs` | 직렬화 계약 |
| `document/1` 와이어: `DocRequest`, `DocResponse`, `PROTOCOL_FIELD`, `DOCUMENT_PROTOCOL` | `kinekino-source-host/src/plugin_host/document_wire.rs` | 직렬화 계약 |
| 매니페스트 파서 `PluginManifest`, `DocumentSpec`, `Categories`, `MANIFEST_FILE = "plugin.json"`, `discover`, `default_dir` | `kinekino-source-host/src/plugin_host/manifest.rs` | 호스트 발견 로직 + 파일 스키마 |
| 프로세스 실행 `ProcessSource`, `DocumentPlugin` | `kinekino-source-host/src/plugin_host/{process,document}.rs` | 호스트 전용 |
| HTTP 워커 프레임 코덱 | `kinekino-source-host/src/application/codec.rs` 등 | 호스트 내부 IPC(플러그인 계약 아님) |

불일치: ADR 0011 결정 3은 매니페스트를 `kinekino-plugin.toml`로 적었지만 코드는 `plugin.json`이다. SDK 0.1은 코드의 `plugin.json`을 정본으로 하며, ADR 0011 결정 3을 2026-10-30에 같은 내용으로 개정했다(TOML 전환은 별도 결정 필요).

## 공개 API 경계

SDK에 들어가는 것(플러그인 작성자가 의존하는 최소 집합):
1. 프로토콜 상수: `SOURCE_API`, `DOCUMENT_PROTOCOL`, `PROTOCOL_FIELD`.
2. 와이어 타입: `source/1`의 `Request`/`Response`/`Ref`/`Item`/`Stream`, `document/1`의 `DocRequest`/`DocResponse`와 그 안의 값(`CatalogPage`, `RawDocument`, `Provenance`, `CatalogError`, `EpisodeDocumentData`).
3. 매니페스트 스키마: `plugin.json`의 직렬화 구조(`id`, `version`, `api`, `capabilities`, `media_hosts`, `executable`, `document`)와 검증 함수. 발견(`discover`, OS 경로)은 호스트 책임이라 넣지 않는다.
4. 플러그인 쪽 실행 도우미: stdin 한 줄 읽기 → 디스패치 → stdout 한 줄 쓰기를 하는 `serve(handler)`와 `SourcePlugin` 트레이트(능력별 기본 `unsupported`). `serve_fixture`는 `fixture` 기능 뒤로 옮긴다.

SDK에 넣지 않는 것: `SourceProvider`(호스트 포트), `ProcessSource`, `DocumentPlugin`, `LocationRules` 설치, HTTP 워커 코덱, reqwest·tokio 의존. SDK 의존성은 `serde`, `serde_json`만 허용한다.

코어와의 관계: `kinekino-core::source` 도메인 값은 SDK 와이어 타입에서 `From` 변환으로 만든다. 코어는 SDK에 의존해도 되지만 SDK는 코어에 의존하지 않는다(의존 방향 SDK ← core ← source-host).

## 버저닝

- 크레이트 버전(semver)과 프로토콜 버전(`source/N`, `document/N`)을 분리한다. 프로토콜 major가 바뀌면 크레이트 major도 올린다. 0.x 동안은 minor를 major처럼 취급한다.
- 호환 규칙: 와이어 필드 추가는 `#[serde(default)]`로만(같은 major), 제거·의미 변경은 major. 요청 enum은 알 수 없는 메서드에 플러그인이 `Error{kind:"unsupported"}`로 답하게 한다.
- 협상: 호스트는 매니페스트 `api`가 자신이 아는 major와 같을 때만 실행한다(현재 `is_compatible`의 정확 일치). 여러 major 동시 지원은 필요해질 때 `api` 목록으로 확장한다.
- 배포: 공개 저장소 태그 `plugin-sdk-v<semver>`. 비공개 플러그인은 `git = ..., tag = ...`로 고정한다. crates.io 게시는 하지 않는다(`publish = false` 유지, 바꾸려면 별도 승인).
- 계약 테스트: SDK에 와이어 JSON 골든 fixture를 두고 호스트·플러그인 양쪽 테스트가 같은 fixture를 읽는다.

## 크레이트 분리안

```
crates/kinekino-plugin-sdk/
  src/lib.rs            재노출만
  src/protocol.rs       상수·버전
  src/wire/source.rs    source/1
  src/wire/document.rs  document/1 + 값
  src/manifest.rs       plugin.json 스키마·검증
  src/serve.rs          플러그인 쪽 루프, SourcePlugin
  src/fixture.rs        feature = "fixture"
  tests/golden/*.json
```

## 마일스톤

| 단계 | 내용 | 완료 기준 |
|---|---|---|
| S0 | 이 설계 승인 | 이 문서 active |
| S1 | 새 크레이트 생성, `wire.rs`·`document_wire.rs`·document 값 이동, source-host는 재노출로 유지 | 기존 테스트 무변경 통과, clippy·lint-structure 0 |
| S2 | 매니페스트 스키마 분리(발견은 host에 남김), 골든 fixture | 양쪽 계약 테스트 통과 |
| S3 | `serve`/`SourcePlugin` 추가, 테스트용 가짜 플러그인을 SDK만으로 작성 | 가짜 플러그인 e2e(`kinekino plugins list`) |
| S4 | 비공개 플러그인 저장소가 로컬 경로 의존으로 빌드 | 로컬 빌드 성공(push 없음) |
| S5 | 첫 공개 push 뒤 `plugin-sdk-v0.1.0` 태그(사용자 승인) | 비공개 저장소가 태그로 빌드 |

## 미결

- ~~매니페스트 형식~~: `plugin.json`으로 결정(ADR 0011 결정 3 개정).
- 서명 필드: [minisign 서명 검증 설계](plugin-signature-verification.sot.md)의 `signature` 위치를 S2에서 반영.
