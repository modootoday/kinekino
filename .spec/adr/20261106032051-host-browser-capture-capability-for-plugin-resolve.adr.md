---
id: adr:kinekino/20261106032051
title: 플러그인 Resolve용 호스트 브라우저 캡처 능력
status: active
created: 2026-11-06T03:20:51Z
updated: 2026-11-06T04:20:00Z
rel:
  governs:
    - crates/kinekino-source-host
    - crates/kinekino-browser
    - crates/kinekino-core/src/source
  depends_on: [adr:kinekino/20261030200000]
---

# 플러그인 Resolve용 호스트 브라우저 캡처 능력

## Context

로드맵 plan:kinekino/20261031153240의 M3-4(PG-8)다. ADR 0011 Decision §1은 `Resolve(episode) -> StreamCandidates`를 두고, 브라우저 캡처가 필요한 Resolve는 **코어 browser 포트를 호스트 능력으로 플러그인에 위임**하며 플러그인이 브라우저를 소유하지 않는다고 정했다. M3 완료 기준 (3)은 그 위임 설계를 ADR로 정하고 첫 경로가 계약 테스트를 통과할 것을 요구한다. 첫 경로 구현은 **M3-5**이고, 이번 조각은 설계만 닫는다.

코드 현황(2026-11-06):

- `kinekino-browser`의 `NetworkInterceptor`는 Allow/Block만 제공한다(`engine/port/network.rs`). Resolve 위임 API는 없다.
- `SourceProvider::resolve`와 process host의 IPC Resolve 전달은 있다(`kinekino-source-host`). `StreamCandidate { url, headers }`는 `kinekino-core`에 있다.
- in-process player의 `browser_provider` / `browser_network` / `episode_resolve`는 옛 카탈로그 경로의 미디어 캡처이며, 플러그인 Resolve용 **호스트 능력**이 아니다. PG-8을 이미 만족한다고 보지 않는다.
- 현황표 PG-8 = 미구현(“browser crate NetworkInterceptor만, source-host 위임 0건”).

권고안(루틴 진행):

1. Resolve 캡처용 브라우저는 **호스트가 소유**한다. 플러그인은 Chromium/WebView를 시작하지 않는다.
2. 페이지 로드·네트워크 관찰이 필요하면 플러그인이 기존 process IPC(source-host)로 **호스트 브라우저 캡처 능력**을 요청하고, 호스트가 `kinekino-browser`(또는 이후 얇은 파사드 뒤의 현재 player browser 포트)를 구동한다.
3. 호스트 능력 계약(설계만): 이미 허용된 불투명 탐색 대상, 선택적 대기 힌트 → 제한된 **관측 네트워크 요청** 목록. 플러그인이 `StreamCandidate`로 매핑해 Resolve를 반환한다. 호스트는 사이트별 DOM/JS로 회차 신원을 해석하지 않는다.
4. `NetworkInterceptor`만으로 플러그인 대면 API를 늘리지 않는다. Allow/Block은 엔진 정책용으로 유지하고, Resolve 위임용 **capture/observe 세션 API**를 따로 둔다(구현은 M3-5).
5. 거절: (a) 플러그인 소유 브라우저 프로세스, (b) 호스트의 사이트별 resolve(호스트·플러그인 경계 위반), (c) 이 조각에서 실제 CDP resolve 배선, (d) 이 조각에서 `StreamCandidate` 스키마 확장(필수가 아니면 하지 않음).

skill: 조회 결과 무관(도메인 스킬 미적용).

## Decision

1. **호스트가 Resolve 캡처용 브라우저를 소유한다.** 플러그인은 Chromium/WebView를 시작·소유하지 않는다. Resolve에 페이지 로드와 네트워크 관찰이 필요하면 플러그인은 기존 source-host process IPC로 호스트 능력 `BrowserCaptureSession`(이름 확정)을 요청하고, 호스트가 `kinekino-browser` 엔진(또는 후속 파사드가 감싼 현재 player browser 포트)을 구동한다.
2. **호스트 능력 계약(설계만, M3-5에서 코드화).**
   - **요청:** 이미 허용된 불투명 탐색 대상(URL 또는 이미 인정된 `SourceRef` 파생 문서 위치), 선택적 대기 힌트(예: 네트워크 유휴·제한 시간).
   - **응답:** 상한 있는 **관측 네트워크 요청** 목록. 각 항목은 `url`, `method`, `resource_type`, 호스트가 재노출하기로 한 request headers.
   - 플러그인이 관측 결과를 `StreamCandidate{url, headers}`로 매핑해 `Resolve` 응답을 만든다. 호스트는 사이트별 DOM·JS로 회차·시리즈 신원을 파싱하지 않는다.
3. **허용(admission).** 탐색 대상과 캡처된 미디어 호스트는 이미 AdmittedHosts / 매니페스트 `media_hosts`(M3-1)를 통과해야 한다. deny-by-default를 유지한다. 미허용 호스트는 탐색도 반환도 하지 않는다.
4. **`NetworkInterceptor`를 플러그인 대면 Resolve API로 과부하하지 않는다.** Allow/Block은 엔진 정책(차단·허용)용으로 유지한다. Resolve 위임용으로는 별도 `BrowserCaptureSession`(open → observe/collect → close) 포트를 `kinekino-browser`(또는 source-host가 노출하는 호스트 능력 면)에 둔다. 구현·계약 테스트는 M3-5.
5. **이번 조각에서 하지 않는 것.** 실제 CDP/WebView resolve 배선, mock `BrowserEngine` 경로, `StreamCandidate` 스키마 확장, player in-process 캡처를 PG-8 완료로 재해석하는 일.
6. **M3-5 범위.** 첫 경로 = mock `BrowserEngine` + 계약 테스트: 플러그인 Resolve가 호스트 캡처를 거쳐 `StreamCandidate`를 반환한다. 실사이트·실 WebView 없음.

## Rationale

- ADR 0011 §1과 로드맵 M3 기준 (3)을 설계 단계로 닫아, M3-5가 코드·테스트만 다루게 한다.
- 호스트·플러그인 경계: 사이트 해석은 플러그인, 브라우저 프로세스·허용 정책·관측 재노출은 호스트. 호스트가 사이트별 resolve를 하면 경계를 깨고, 플러그인이 브라우저를 소유하면 격리·허용목록·서명 모델을 우회한다.
- `NetworkInterceptor`는 정책 결정(Allow/Block)만 담는다. 세션형 캡처·관측을 여기에 얹으면 엔진 정책과 플러그인 능력이 섞인다.
- in-process player 캡처는 옛 카탈로그 경로이며 process 플러그인 Resolve IPC와 연결되지 않았으므로 PG-8 근거가 아니다.
- `StreamCandidate`는 이미 `{url, headers}`로 충분하다. 이 조각에서 스키마를 키우지 않는다.

## Alternatives rejected

| 대안 | 이유 |
| --- | --- |
| 플러그인 소유 브라우저 프로세스 | ADR 0011·격리·허용목록 모델과 충돌 |
| 호스트가 사이트별 DOM/JS로 resolve | 호스트·플러그인 경계 위반(사이트 용어·해석을 호스트에 둠) |
| `NetworkInterceptor`만으로 위임 API 제공 | Allow/Block 정책과 세션 관측이 섞임 |
| 이 조각에서 CDP/실엔진 배선 | M3-4는 ADR-only; 구현은 M3-5 |
| player in-process 캡처를 PG-8 완료로 간주 | source-host 위임 0건; 옛 카탈로그 경로 |

## Consequences

- PG-8은 ADR 완료로 **부분**이 된다. 코드·계약 테스트는 M3-5까지 미구현으로 남는다.
- M3-5는 mock `BrowserEngine`과 `BrowserCaptureSession` 계약 테스트로 첫 경로를 증명한다. 실사이트는 넣지 않는다.
- source-host IPC에 호스트→플러그인 캡처 능력 메시지가 추가될 예정이다(스키마는 M3-5). Resolve 응답의 `StreamCandidate` 형태는 유지한다.
- 기존 `NetworkInterceptor` 호출부는 그대로 두며, Resolve 위임 경로와 섞지 않는다.

## Record

- 2026-11-06: M3-4 — host-owned browser capture capability for plugin Resolve designed (`BrowserCaptureSession`); NetworkInterceptor stays Allow/Block; first path deferred to M3-5.
- 2026-11-06: M3-5 — first path: mock BrowserEngine + BrowserCaptureSession contract tests; plugin maps observations to StreamCandidate; NetworkInterceptor unchanged; no real site/WebView.
