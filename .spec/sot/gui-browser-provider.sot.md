---
id: sot:kinekino/gui-browser-provider
title: GUI 없이 구동하는 browser provider 어댑터
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# GUI 없이 구동하는 browser provider 어댑터

상태: 2026-10-19 SDK 포트 인계 후 첫 DOM 카테고리 수직 소스를 작성하고 freeze했다. rustfmt 작성은 종료0이며 컴파일/회귀/strict Clippy·실제 CLI 인수는 SDK 중앙 묶음 대기다. root API/Cargo/main/CLI/domain 정책은 SDK 소유다. 새 GUI/브라우저 실행·네트워크·독자 cargo는 없다.

## 기존 소스의 재사용 경계

| 기존 모듈 | 재사용할 의미 | 그대로 재사용할 수 없는 부분 |
|---|---|---|
| browser/mod.rs | 허용 navigation, popup 거부/요청 전환, page load·IPC 관측 | parent HWND·Rc·WebView가 GUI 수명에 연결됨. callback의 GUI Session 의존과 unbounded Sender를 provider 외부 handle로 내보내지 않음 |
| browser/events.rs | 비동기 callback의 frozen generation, catalogue에서 늦은 media 폐기 | generation 단독으로 request/session identity를 대체하지 않음. 신규 typed envelope와 bounded 송신 필요 |
| browser/requests.rs | 기존 adblock::engine/blocked와 resource kind·referer 의미 | filters.local.txt 파일 읽기·engine 생성은 요청 callback 밖에서 준비. 설정값과 실제 filter 설치·관측 counts 구분 |
| browser/responses.rs | 응답 MIME·URL 관측과 GetContent 완료 callback | 현재4096bytes manifest prefix는 전체 network raw가 아님. COM callback 안 HTML parse·파일 게시 금지 |
| browser/location.rs | 실제 SourceChanged와 generation 중복 증가 방지 | 새 application의 session/generation 계약과 함께 검토. URL validation은 root 정책 재사용 |
| video_surface.rs | 기존 Win32 소유 HWND/Drop 패턴 참고 | mpv child/eframe parent는 재사용하지 않음. provider가 숨김 부모와 message pump를 독립 소유 |

기존 Browser·GUI Session을 단순 hidden 실행해 제품 GUI를 선행 조건으로 만들지 않는다. 기존 GUI 호환은 유지하며 공유 추출은 순수 resource 분류·필터 적용·generation 관측의 최소 책임으로 제한한다. 정책 복제나 두 provider의 서로 다른 allowlist를 만들지 않는다.

## 예정 수명과 비동기 경계

전용 worker가 STA 초기화·숨김 부모 HWND·WebView2/COM/Rc·message pump를 생성하고 같은 스레드에서 해제한다. 외부 Send+Sync handle은 bounded 명령 채널과 bounded 상태 저장소만 소유한다. unsafe Send로 COM/창을 넘기지 않는다. try_start/poll/cancel은 즉시 응답하고 LocalHost dispatcher에서 네트워크 완료를 기다리지 않는다.

capture는 request ID+session+generation+final URL과 network raw/DOM kind를 붙인 owned 자료만 넘긴다. DOM serialization을 실제 network raw로 표시하지 않는다. 수신 body의 copy/read 예산·COM stream 수명/마샬링 방식·overflow disposition은 SDK 계약 인계 때 확정해야 한다. 완료 callback은 느린 HTML 파싱/파일 게시/네트워크 대기를 수행하지 않는다. 후속 CPU/파일 작업은 별도 executor와 typed completion으로 연결한다.

cancel/timeout/새 request 뒤 늦은 callback은 identity와 generation을 검사해 폐기한다. 채널 포화·worker 초기화 실패·부모 창/engine 종료는 명시 terminal 오류이며 빈 목록 성공으로 바꾸지 않는다. shutdown은 소유 worker의 종료 통지를 남기며 사용자 앱·다른 WebView/GUI를 종료하지 않는다. Drop에서 dispatcher를 무기한 대기시키지 않는 owned shutdown 계약을 확인한다.

## 계약 도착 때 필요한 확인

request validation 결과와 허용 URL의 표현, body/DOM/메시지/동시 ticket 한도, 취소/terminal 우선순위, adblock 설정과 관측 필드, raw artifact 인계·publisher 책임, host 종료/재시작과 ticket 안정성은 SDK 정본을 따른다. engine 접근 성공과 catalog parser 성공, CLI 제품 지원을 각각 분리해 검증한다. 기존 HTTP connect 실패를 봇 탐지나 TLS 실패로 확정하지 않는다.

## 첫 소스 인계 범위

start(profile_root)는 기존 절대 profile 디렉터리를 받고 BrowserHost를 즉시 반환한다. SDK는 host를 보유하고 port()/owner() Arc를 얻는다. host Drop은 shutdown 신호만 보내며 호출 스레드에서 worker join이나 COM 종료를 기다리지 않는다. shutdown_snapshot의 worker_exited로 소유 종료 완료를 확인한다. Starting/Ready/Stopping/Stopped/Failed를 실제 worker 수명에 연결한다.

명령16개·보존 ticket32개·동시 active1개, callback16개·DOM2MiB 한도를 적용했다. 외부 상태 접근은 try_lock, 명령은 try_send이며 큐 실패 시 새 ticket/취소/탐색 상태를 바꾸지 않는다. navigate는 공통 BrowserWork clone에 reserve_navigation 적용 후 같은 잠금 범위에서 enqueue/교체한다. 두 개의 headless 회귀는 admission 실패 시 ticket/generation 미소비와 cancel 실패 시 기존 상태 보존을 검증하도록 작성했다. 아직 실행 결과는 없다.

COM/Rc/WebContext/숨김 부모/메시지 pump는 worker 소유다. callback은 bounded 자료 전달·광고차단 요청 판정만 하며 HTML parse/파일 publication은 하지 않는다. DOM serialization bridge의 실제 location.href와 frozen URL/document generation을 대조하고 늦은 load URL도 검사한다. 포화는 명시 실패이며 deadline과 cancel은 공통 BrowserWork에 반영한다.

첫 출력은 DomSnapshot/response=None이다. network entity raw·실제HTTP status를 생성하지 않는다. ResolveEpisode의 직접 media request 후보는 기존 MediaSource::from_request로 관측하지만 manifest prefix·network response body 관측은 아직 구현하지 않았다. 따라서 전체 회차 resolve 완료로 표시하지 않는다. popup은 새 창으로 열지 않고 명시 단계 이동 필요로 실패시킨다. 현재 필터는 공통 embedded adblock 규칙이며 GUI의 filters.local.txt 추가 설정 로드는 이 첫 경로에 포함하지 않는다. blocking_requested/설치 성공/실제 requested·blocked counts를 구분한다.

기존 GUI에 대한 변경은 browser::requests와 resource_kind의 crate 공유 가시성뿐이며 동작은 변경하지 않았다. SDK가 infrastructure 모듈 등록과 CLI 조립/필수 검사를 소유한다.

## 다음 manifest 관측 소스

별도 media_observer.rs를 미등록 상태로 준비했다. 기존 DOM checkpoint 파일은 수정하지 않았다. response callback은 기존 MIME/HTTPS/media-cdn .txt 범위를 검사하고, 비동기 GetContent 완료 callback은 COM stream을 최대4개 대기/완료 합산 큐에 보관한다. callback에서 stream Read·HTML parse·파일 publication을 하지 않는다. worker의 poll_one은 최대4096bytes prefix 하나를 판독하고 공통 MediaSource::from_manifest와 BrowserMedia로 반환한다. prefix를 complete network raw로 표시하지 않는다.

RequestIdentity closure는 원래 요청과 상관된 frozen ticket/document generation을 제공해야 하며, 응답 시점 현재 페이지를 그대로 stamp하는 것으로 대체하지 않는다. 이 lookup의 실제 bounded request registry 결합은 다음 접합 범위다. 별도 current closure를 응답/완료/판독 전후에 적용하며 stale 자료를 폐기한다. Drop은 handler를 해제하고 늦은 완료가 자료를 다시 넣지 못하게 닫는다. private COM/stream은 STA를 벗어나지 않는다.

manifest identity/prefix 한도와 기존 MIME/URL 범위의 synthetic2회귀를 작성했고 rustfmt 종료0이다. 모듈 미등록이므로 컴파일/테스트 실행·실제 회차 resolve는 미검증이다. 첫 DOM 인수 후 SDK와 별도 source freeze/필수 검사/CLI 인수를 조율한다.
