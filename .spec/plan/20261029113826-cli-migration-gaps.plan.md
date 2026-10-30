---
id: plan:kinekino/20261029113826
title: UI 호출에서 공통 CLI로 이관할 경계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# UI 호출에서 공통 CLI로 이관할 경계

2026-10-19 최신 사용자 지시: 제품 전체 기능을 CLI로 구현·검증한 뒤 GUI 단계로 진행한다. 이 문서는 UI 소스에서 확인한 이관 후보이며 실제 CLI 지원·인수 완료 목록이 아니다. UI 신규 기능·수동 점검·폴리시는 보류한다. 구현 소유권은 SDK 담당에 있고 역할 이관은 PM 경로를 따른다.

## 소스에서 확인한 누락 후보

`src/presentation/player_app.rs`의 `navigate`는 EpisodeQueue 갱신, native 정지, Session 초기화, browser.navigate와 오류 처리를 조합한다. `pump_events`는 browser generation으로 오래된 이벤트를 배제하고 회차 목록 관측·transition.page_ready·open_episode_player·Session.observe의 후속 이동을 연결한다. toolbar는 카테고리 이동과 browser.set_blocking을 직접 호출한다. 이 실행 조합이 GUI 없이 같은 application 유스케이스로 노출되는지 SDK에서 대조해야 한다. CLI가 GUI 입력을 재생하거나 이 조합을 복사하는 방식으로 이관하지 않는다.

| 경계 | 공통 SDK/CLI 인계 시 확인할 계약 | 필요한 인수 근거 |
|---|---|---|
| 카테고리·검색·작품·회차 | provider 요청/결과, 안정 ID, pagination, 실패와 미지원 구분 | 실제 사이트 CLI 조회와 결과 provenance. 소스에 버튼이 있다는 사실은 지원 증거가 아님 |
| 회차→플레이어 이동 | navigation generation, page identity, 명시 timeout/cancel, stale event 거부 | frame.example 도달과 다음 처리의 CLI 수직 검증 |
| 광고 차단 | 실행 host에 적용된 정책과 관측 상태를 구분, 요청 실패 | 동일 provider 경로의 실제 적용 근거. checkbox 설정값만으로 성공 판정 금지 |
| 영상 해석·재생·회차 전환 | media candidate/route, host 수명, accepted와 observed 구분, 종료·오류 | GUI 없는 CLI 실행 및 실제 backend 관측, 오래된 회차 이벤트 배제 |
| 자막·다운로드·에셋·jobs·AI | 기존 각 typed 계약 재사용, source identity·출력·오류·취소·provenance | 각 SDK 기능 시험과 실제 CLI 결과. UI 표시/디자인 표본으로 대체 금지 |

`src/bin/player-cli/runner.rs`는 in-process SDK 또는 명시 endpoint를 호출하며 help에서 discovery/subtitles.convert/frame capture와 review/proposal/project backend를 설명한다. help 문구는 현재 모든 명령의 실제 지원을 증명하지 않는다. 전체 capability·schema·dispatch·adapter·실제 CLI fixture를 SDK 담당이 대응시켜야 한다. 특히 browser/provider 작업과 재생 host 수명은 oneshot CLI 프로세스만으로 자동 충족된다고 가정하지 않는다.

## 명령별 정적 대응표

아래 지원 상태는 `application/api/{capabilities,schema,mod}.rs`와 `application/playback_control/mod.rs`의 현재 소스 판정이다. 실제 CLI 실행 인수는 이 조사에서 수행하지 않았다. 조건부 지원은 실행 host에 해당 adapter가 주입되어야 한다.

| 기능 | 기존 책임 모듈 | 공통 application 포트/CLI 현재 상태 | 부족한 adapter·계약 | 실제 CLI 인수 시나리오 |
|---|---|---|---|---|
| 카테고리/검색 | presentation/workspace::open_category, player_app::toolbar, infrastructure/browser | catalog.search 등록만 존재·unsupported. 카테고리 전용 실행 계약 미확인 | provider catalog 조회·검색·페이지 결과 포트 | 방영중/종영/극장판 각 조회, 검색 일치/빈 결과/실패를 실제 사이트 provenance와 반환 |
| 작품/회차 목록 | application/episode_queue, browser의 episode observer | episodes.list 등록·unsupported | 작품 identity와 typed 목록 조회 adapter | 작품 선택 후 순서·현재 회차·경계/잘못된 작품 입력 검증 |
| frame.example 이동/영상 해석 | application/session, player_app::pump_events, infrastructure/browser/{events,responses,location} | episodes.resolve 등록·unsupported | GUI 이벤트 루프에 묶이지 않는 resolve 실행 수명·timeout/cancel | 회차→허용 player→media 후보, 차단 URL/오래된 generation 거부 |
| 광고차단 | player_app::toolbar, infrastructure/browser/requests, domain/navigation | 전용 실행 명령 없음; browser.set_blocking 직접 호출 | 정책 적용/관측 포트와 capability 설계 필요 | 차단 활성/비활성 실제 요청 결과, 실패와 설정값 구분 |
| 영상 열기/종료 | presentation/transport::start_native, workspace::open_local, player_app::stop_native, infrastructure/mpv | playback.open/close 등록·unsupported | headless host의 source 검증·프로세스 수명 adapter | 로컬·해석된 원격 source 열기, 오류/종료/reopen 및 자원 회수 |
| mpv/Web 전환 | application/playback_route/native_playback, transport::start_native/return_web | playback.engine 등록·unsupported | 공통 route 전환·위치/paused 관측·복귀 포트. 현재 Mpv::launch에 surface.id 사용 | 전환 성공/실패, 위치·음량·paused 유지와 stale frame/이전 engine 차단 |
| native 재생/일시정지/탐색/음량 | presentation/native_controls, application/playback_control, infrastructure/native_playback_control | play/pause/seek/set_volume 조건부 지원. 기본 unavailable; receipt는 accepted/unobserved | GUI 없이 control handle을 bind하는 명시 host 수명 | 실제 CLI submit 뒤 backend 관측, 잘못된 seek/종료된 instance 거부; enqueue를 실행 완료로 판정하지 않음 |
| Web 재생/탐색 | transport::seek/toggle/return_web, browser.control | 위 native control 지원을 Web 지원으로 확대 불가 | Web control application 포트 및 관측 결과 | GUI 자동입력 없이 Web backend 제어/실패/관측 구분 |
| 상태/속도/트랙 | application/playback_status, infrastructure/mpv/events | playback.status 조건부 지원; set_rate/select_track 등록·unsupported | Web 상태·속도·트랙 typed adapter | 실제 상태 unknown/null과 관측값 구분, 선택 트랙/속도 적용·실패 |
| 이전/다음 회차 | presentation/episode_panel::choose_episode/episode_button, application/episode_transition/episode_queue | playback.previous/next 등록·unsupported | 공통 선택·전환 usecase와 native intent/paused 복원 | 첫/마지막 경계, 빠른 연속 전환, timeout과 이전 회차 이벤트 배제 |
| 자막 | application/subtitles, infrastructure/subtitle_files | subtitles.convert 조건부 지원; transcribe/diarize/speakers/align_speakers/edit_cues/edit_speakers 등록·unsupported | 실제 각 작업 typed executor/AI 계약. convert와 다른 기능 혼동 금지 | SRT/VTT 손실 확인·원본 보존·충돌, 이후 AI 입출력 provenance/오류/취소 별도 검증 |
| 다운로드 | production.download_media 등록, 현재 API dispatch의 미지원 경로 | unsupported | source 해석 재사용·출력/취소/부분 결과·jobs adapter | 실제 media 신규 출력, 충돌/네트워크 실패/취소와 결과 검증 |
| 프레임 캡처 | application/frame_capture와 주입 frame port | production.capture_frame 조건부 지원; 로컬 MP4/MKV/WebM·PNG·FFmpeg 의존, network=false | 원격/현재 재생 source와의 연결은 별도 | CLI 고정 sourcePTS·출력 PNG·원본 보존·timeout/충돌, 원격 미지원 명시 |
| 에셋/제작 | production의 export_cards/segment_subject/build_assets/package_projection/render_short 명령 | 등록·unsupported | 각 application executor·typed 결과·jobs 연결 | source/revision·출력·취소·오류별 실제 CLI 인수 후 GUI 연결 |

Browser는 Wry WebView를 생성하고 기존 JS 관측 파일을 주입한다. 이는 현재 platform adapter의 관측 사실이며 CLI 지원을 위해 GUI 자동화를 호출하라는 뜻이 아니다. headless provider가 브라우저 엔진을 필요로 하는지와 Rust 대체 가능 범위는 SDK 설계 결정으로 남긴다. 기존 JS는 임의 삭제하지 않는다. root API/Cargo/main 소유권은 SDK 담당에게 유지한다.

## UI 담당 후속 범위

2026-10-19 추가 소스 관측: experimental HTTP catalog.search/episodes.list의 공통 SDK·--catalog-root 연결은 별도 [계약](../../../sots/player/application/api/catalog.md)으로 진행한다. 위 표의 unsupported는 조사 당시 checkpoint이며 실제 사이트 성공으로 승격하지 않는다. 사용자의 기존 브라우저 정상 접근 관측과 직접 HTTP 연결 실패는 구별한다. CLI 우선은 HTTP-only 제약이 아니다.

### 화면 없는 브라우저 provider 인계 범위

현재 source 제안이며 실제 구현/실행이 아니다. SDK가 src/application/browser_provider/{mod,requests,state}.rs의 공통 DTO/비차단 포트·정책과 CLI/jobs host 조립을 소유한다. UI 담당은 src/infrastructure/browser_provider/{mod,worker,window,capture}.rs 및 기존 infrastructure/browser/{requests,responses,events}의 최소 공통 추출을 소유한다. root Cargo/API/CLI/domain 정책은 SDK 소유를 유지한다. 새 독립 crate나 presentation 정책 복제를 만들지 않는다.

- BrowserProviderPort의 try_start/poll/cancel은 네트워크/COM 완료를 기다리지 않는다. bounded command admission, ticket/session generation, 현재 snapshot/terminal result와 취소 수신/완료를 구분한다. queue full은 resource_exhausted다.
- 전용 STA worker가 COM/Rc/WebView2·숨김 native 부모·message pump를 소유한다. Send+Sync handle은 bounded owned DTO/bytes와 짧은 snapshot 상태만 공유하며 unsafe Send를 구현하지 않는다.
- 요청의 strict 허용 URL과 실제 최종 URL·request/ticket/session/generation을 대응한다. 이전 generation의 callback, 종료 후 callback, 외부 navigation은 결과로 채택하지 않는다. 허용 범위를 기존 느슨한 navigation_allowed의 host 검사만으로 확대하지 않는다.
- adblock의 기존 domain/엔진과 resource type·method·source 판단을 공유한다. 요청된 설정과 실제 차단 관측을 별도 기록한다. 진단에 cookie/header token/서명 URL/원문 Debug를 노출하지 않는다.
- complete network response는 기존 WebResourceResponseReceived/GetContent를 bounded 방식으로 읽어 Rust parser에 넘길 수 있다. 현재 responses는 HLS 4096B prefix만 읽으므로 HTML 전체 확보 구현이 필요하다. DOM snapshot을 쓰면 network bytes와 capture_kind/provenance를 구별하고 JavaScript는 최소 전달 bridge로 제한한다. 추출 정책은 Rust다.
- 장기 실행은 SDK의 기존 jobs executor에서 비차단 포트를 poll하며 deadline/cancel을 연결한다. 동기 CatalogFetchPort에 browser를 그대로 주입해 LocalHost dispatch를 장시간 막지 않는다. CLI wire의 완료 결과와 job 접수 receipt를 같은 DTO로 바꾸지 않는다.
- host 종료는 새 접수 차단→자기 소유 요청 취소→worker 종료/회수 순서를 가진다. 외부 사용자 브라우저·프로필·프로세스를 종료하거나 GUI 클릭을 자동화하지 않는다. timeout/disconnect를 성공으로 판단하지 않는다.

최소 인수는 network 없는 typed state/queue/stale/cancel/종료 회귀 후, 명시 CLI host에서 실제 category→series→episode/resolve와 실패·timeout·EOF 종료를 기록하는 순서다. 새 수동 GUI 검토나 사용자 화면을 기능 gate로 사용하지 않는다.

SDK 최종 계약이 바뀌면 기존 ApplicationClient 기반 얇은 adapter의 컴파일 호환만 조정한다. presentation에 provider 정책·다운로드·AI 실행·job 상태 전이를 새로 구현하지 않는다. 프로젝트 조회 UI의 현재 5파일은 검사 대기 상태로 고정하며 실제 GUI를 실행하지 않는다. 이 조사에는 네트워크 요청·미디어 실행·새 추론·cargo 검사가 포함되지 않았다.
