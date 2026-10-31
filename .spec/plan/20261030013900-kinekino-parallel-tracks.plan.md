---
id: plan:kinekino/20261030013900
title: kinekino 병렬 작업 분류(P1 3·4단계 진행 중 기준)
status: active
created: 2026-10-30
updated: 2026-10-31T11:40:00Z
rel:
  governs: [crates]
---

# kinekino 병렬 작업 분류

## Goal

다른 세션이 P1 4단계(플레이어 플랫폼 cfg 약 80개 파일을 어댑터로 이동)를 진행하는 동안, 파일 집합이 겹치지 않는 로드맵 단계를 골라 별도 worktree에서 병렬로 진행한다. 기준 로드맵은 [kinekino 로드맵](20261029151512-kinekino.plan.md), 기준 커밋은 main `6908a43`(P1 3단계 병합 직후)이다.

## 진행 중 세션의 파일 집합

- P1 3단계(완료, `6908a43`): `kinekino-app`, `kinekino-core`(OsPolicy 포트), `kinekino-platform`, `kinekino-player`, `kinekino-process`, `kinekino-host`.
- P1 4단계(진행 예정·진행 중): `kinekino-player` 전반(cfg 99개 파일 중 다수), `kinekino-platform*`, `kinekino-app`, `assembly/meari`(마지막), lint allowlist(`_baseline`·xtask allowlist).
- 공유 충돌 지점: 루트 `Cargo.toml`(members·workspace.dependencies), `xtask/src/structure/boundary/mod.rs`의 `CLASSES`, lint allowlist 파일. 병렬 작업은 이 세 곳을 한 줄 추가로만 건드린다.

## 분류

|단계|내용|건드리는 파일|분류|근거|
|---|---|---|---|---|
|P1b|`InferenceBackend` 포트, HF 어댑터, 목 서버 계약 테스트, 비용 가드, 동의, `settings.inference` 스키마|신규 crate 2개 + 루트 Cargo.toml·CLASSES 각 1~2줄|A 지금 병렬|신규 파일 중심, core는 읽기만|
|P1b-후속|`kinekino settings get/set` CLI, 원장 파일 저장, Linux Secret Service CredentialStore 어댑터|`kinekino-app`·`kinekino-cli`·platform-unix|B 4단계 후|app·platform 겹침|
|P5 일부|ASS 렌더·자막 편집(`kinekino-subtitles`)|subtitles crate(cfg 위반 여부 확인 필요)|A 조건부|player를 건드리지 않는 범위만|
|P6 일부|`MascotEventPort` 포트 + no-op/log 어댑터(신규 crate)|신규 crate|A 지금 병렬|포트는 프리셋 없이 정의 가능(로드맵 의존성 절)|
|P4|제작 executor(cards, shorts, assets, package_projection)|player studio·jobs|B 4단계 후|player 대량 이동과 겹침|
|P2|Linux mpv 재생 어댑터|platform-unix·player playback|C 승인 필요 + B|mpv 설치 승인, player 겹침|
|P3|BrowserProviderPort|browser·player catalog|B + 브라우저 엔진 병합 대기|`feat/browser-engines` 미병합|
|P6 나머지|GUI 셸·설정 페이지 GUI·Meari 연결|app·player·vendor/meari|B|설정 창·플레이어 창(Meari) 병합(2026-10-31). 작품 탐색은 재생·비동기·검색·스크롤·격자 보기까지 병합. 남은 것: 기존 player 화면 제거·파일 끌어놓기|
|P7|실 HF API 평가|-|C 승인 게이트|실호출·과금, 사용자 시각 확인|
|P8|Windows·macOS 재검증|-|C 환경|DATALAB·macOS 장비|
|push·CI|원격 반영|-|C 승인|첫 push는 사용자 승인|

## 착수(2026-10-30)

- `feat/inference-port`(worktree `../kinekino-wt-inference`): crate `kinekino-inference`(Domain; `port/`에 `InferenceBackend`, 태스크·capability·오류 매핑, `MockBackend`, 레지스트리와 `mode`→정책, `CostGuard`·JSONL 원장, 업로드 동의, `settings.inference` 스키마·검증·점 표기 get/set)와 `kinekino-inference-hf`(Adapter; `HttpTransport` 포트, loopback 전용 HTTP 클라이언트, HF 요청·응답 변환, 401/402/429/5xx/타임아웃→typed 오류, 429·5xx만 지수 백오프, 취소, 토큰 마스킹).
- 실제 HF 호출 불가 구조: 이 crate가 제공하는 전송은 `127.0.0.1`/`localhost`만 허용한다. TLS 전송(`router.huggingface.co`)은 P7 승인 게이트와 함께 추가한다.
- 비용 추정: HF 태스크별 단가 미확인이라 `estimate_usd`는 `None`이고 가드는 항상 사용자 확인을 요구한다.

## 설정 페이지 항목 후보(이번 산출물)

`inference.mode`, `models.*`, `budget.*`(원장 표시·초기화), `consent.scope`(철회), `network.*`, `diarization.mode`, `local_packs`, 토큰 등록/삭제(값 비표시, CredentialStore). 스키마는 `kinekino_inference::InferenceSettings`가 정본이다.

## 다음 병렬 후보

1. `MascotEventPort` 신규 crate(포트 + no-op/log 어댑터, 계약 테스트).
2. `kinekino-subtitles` ASS 렌더(크레이트가 player cfg 이동과 무관한지 확인 후).
3. 4단계 완료 후: settings CLI·원장 파일 경로·Secret Service 어댑터를 app/platform-unix에 연결.

## Record

- 2026-10-30: 분류 작성, P1b 착수. 실호출 0, push 없음.
- 2026-10-31: 소스 플러그인 재생 연결 병합(범용 container/playable 경계, 플러그인이 재생목록 구성). push 없음.
