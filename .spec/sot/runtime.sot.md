---
id: sot:kinekino/runtime
title: 소유 자식 문서의 읽기 전용 runtime 관측
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 소유 자식 문서의 읽기 전용 runtime 관측

## 계약과 현재 검증 경계

이 기능은 [실제 자산 응답 인수](../../verification/application/player-asset-session-cli-2026-10-19.md) 다음 단계다. `application/browser_provider/frame_runtime`은 typed 값·검증·표본 예약과 현재 문서 수명을, infrastructure frame_runtime은 실제 Frame2의 고정 DOM 투영만 담당한다. [player-runtime-cli-r1 검증](../../verification/application/player-runtime-cli-2026-10-19.md)은 fmt/strictClippy/전체276pass17ignored/두bin 및 실제 CLI1pass20.11초로8개 표본·원문5개 전체재읽기·EOF 종료를 확인했다. 영상 요소는 생성됐으나 readyState0/networkState2/paused/time0이며 decode·재생 성공은 미확인이다.

준거 스킬은 기존 [modoo.today fixture 계약](../../verification/application/test-fixture-contract-2026-10-19.md)과 Rust 소유권/수명·lint-and-validate다. 정의 기반 synthetic 자료와 실제 native 관측을 구분하고 pending/cancel/재탐색·종료 경계를 검사한다. source의 completed/player_loaded 플래그를 성공으로 바꾸지 않는 판단은 확보26795bytes script의 읽기 전용 연구 `runtime/coordination/player-script-research/findings.md`에 근거한다.

## 범위와 표본

성공 HTTP script body가 현재 protocol scope 및 native owner에 귀속된 뒤, 같은 native owner 아래에서 실제 FrameCreated/Frame5/Frame2로 관측한 현재 player child를 읽는다. script의 CDP frame ID와 native 숫자 frame ID를 조인하지 않는다. 각 표본의 권위는 실제 native FrameDocumentIdentity와 callback 시 재검증이다. source와 runtime을 URL 동일성만으로 하나의 frame이라고 주장하지 않는다.

문서별 한 pending 일회 request만 예약하며250ms 간격으로 최대8개 표본을 받는다. 표본 간격은 Rust Instant, 기록 시각은 관측 Unix ms다. callback 도착·실제 값 확보와 예약 접수를 구별한다. 부모/자식 재탐색·superseded/destroyed·취소·terminal 후 callback은 payload 해석 전에 거부한다. 새 문서는 별도 frame state다. 전체 job deadline은 연장하지 않는다.

상태 DTO는 다음 사실을 별도로 기록한다.

- document readyState 및 visibilityState
- FirePlayer/fireload/jwplayer 함수의 존재와 optional player_loaded 플래그
- `.play-button-outer`, `#playerbase`, `.pppx`의 존재·CSS 표시/가시성·layout·DOM viewport 중심 및 hit-test
- video 개수와 최대8개 element의 readyState/networkState/paused/ended/muted/error code/currentSrc/유한 time·duration/video 크기

문서 complete, 함수 존재, 사이트 플래그, CSS 표시, DOM hit-test는 JW ready event·native window의 실제 가시성·사용자 gesture·decode·시간 진행·오디오 출력의 대체 증거가 아니다. ready event를 실제로 받았다고 표시하는 필드는 없다. `.pppx`의 존재를 읽어도 popup을 허용하거나 해당 control을 활성화하지 않는다.

## 외부 데이터와 최소 어댑터

native callback은 identity 재검증과96KiB envelope 상한 후 큐로 값만 넘긴다. Rust는 strict DTO, URL8192bytes/control 문자, video 개수64/표본8, readyState0–4/networkState0–3/error1–4, 유한 비음수 시간·duration, 현재 document URL 일치를 검사한다. 다른 document·중복 sample의 malformed JSON도 먼저 폐기한다. currentSrc는 관측 자료이며 어떤 scheme이든 media 지원이나 실행 입력으로 승격하지 않는다.

WebView2 Frame2의 ExecuteScript는 고정 read-only DOM serialization에만 사용한다. 이는 Rust infrastructure의 최소 외부 엔진 어댑터이며 기존 WebView 주입 코드의 이관/최소화 추적 범위에 포함한다. 사용자·사이트가 전달한 script/selector/명령을 실행하지 않는다. 네트워크 요청·player 함수 호출·CSS 변경·DOM event 주입·fireload 호출·복호화·packed code 평가를 추가하지 않는다. 예약·재시도 상한·유효성·성공 판정·저장은 Rust가 담당한다.

typed snapshot은 private browser trace 원문에만 저장한다. 일반 job error에는 runtime 관측 여부/개수만 노출한다. raw URI/query·currentSrc를 일반 상태 보고로 복사하지 않는다. 실제 CLI는 기존 DOM/script/trace의 전체 재읽기를 유지하고 runtime typed 필드와8표본을 추가 확인한다.

## 후속 실제 인수

우선 같은 actual CLI에서 현재 child의 표본을 확보한 뒤 관측된 값으로 다음 단계를 결정한다. r2 trace에는 실제 bootstrap POST XHR200/text/html/finished encoded1011이 이미 있으나 본문을 받지 않았다. 필요하면 실제 same-session/request/loader/owner의 일회 body capability를 확장해 그 응답을 보존한다. endpoint를 새로 호출하거나 query/body를 재구성하지 않는다. 자연 발생 manifest/segment·필요한 명시 activation·track·최종 media 검증과 소유 mpv 수명/제어는 별도 잔여이며 전체 SDK/CLI goal을 유지한다.
