---
id: evidence:kinekino/20261029113514
title: 광고 차단 웹·mpv 실제 재생 검증
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# 광고 차단 웹·mpv 실제 재생 검증

## 환경과 판정 방법

2026-10-18, Windows x64. Rust 앱 자체의 WebView2 155.0.4283.60에서 광고 차단 체크박스를 계속 켠 상태로 검증했다. Codex 내장 브라우저의 무차단 재생 결과를 제품 검증으로 재사용하지 않았다. `computer-use` 스킬로 앱의 Play 및 Rust 네이티브 제어 버튼을 실제로 클릭했다. iframe video 속성을 1초마다 읽고, mpv JSON IPC 응답과 속성을 별도로 관찰했다. 실제 영상 화면도 확인했다.

검증 표본은 [최초 조사](../../player/infrastructure/browser/sot-001-browser-research.md)의 동일 세 회차다. 반복 재생은 해당 공개 frame-host 표본 URL을 `--start-url`로 열었다. 이 검증은 목록 탐색 전체의 자동화나 전체 사이트 콘텐츠 호환성 보장을 뜻하지 않는다.

웹 성공 기준은 시간 증가 30초 이상, 영상 크기·디코딩 프레임 확인, media error=0이다. 네이티브는 연결뿐 아니라 전체 길이 일치(허용 오차 2초 미만), 30초 이상 시간 증가, 영상 크기, 오디오 채널, 오류 부재를 확인한다. 조각 파일을 반복 재생하는 경우를 성공으로 인정하지 않는다. 탐색으로 건너뛴 시간을 연속 재생 시간으로 계산하지 않는다.

## 현재 관측 결과

| 분류·표본 | 차단 ON 웹 시간 증가 | mpv 전체 영상·연속 시간 증가 | 영상·오디오 |
|---|---|---|---|
| 방영중: 어떤 암부의 소녀공서 2화 | 55.997652초 | 길이 1420.002239초, 제어 전 35.623656초 증가 | 1920×1080, mpv 2채널 |
| 종영: 치오의 통학로 12화(完) | 49.998328초 | 길이 1439.980896초, 제어 전 61.144411초 증가 | 1280×720, mpv 2채널 |
| 극장판: 파리에 피는 에투알 | 77.000113초 | 길이 7206.199501초, 66.191133초 증가 | 1920×1080, mpv 2채널 |

세 표본 모두 웹·네이티브 각각 30초 이상 진행 판정을 통과했다. 종영 웹은 네이티브 성공 후 50.837457초, 극장판 웹은 78.201199초에서 pause되어 이후 시간이 유지됐다. 위 증가량은 관측된 최초/최종 재생 중 표본의 차이이며, pause 직전의 절대 시각과 구별한다. 방영중은 HLS 수정 직후 기록, 종영·극장판은 정확 탐색 및 전체화면 속성 구독을 포함한 최종 빌드 기록이다.

세 웹 표본에서 차단 요청 카운트가 각각 1 이상이고, 관측한 알려진 광고 슬롯의 visible_ads=0, hidden_ads=1이었다. 제거된 광고 개수나 전체 광고 차단율로 해석하지 않는다. 영상 자체에 합성된 워터마크·홍보 문구는 남을 수 있다. 오디오의 디코딩·채널·음소거 상태를 확인하며 스피커로 실제 청취한 결과는 아니다.

## 재현한 문제와 수정

1. **외부 iframe 요청 누락**: 기존 AddWebResourceRequestedFilter는 같은 출처 iframe만 처리한다. ICoreWebView2_22의 AddWebResourceRequestedFilterWithRequestSourceKinds(ALL)로 교체하여 media-cdn와 worker 요청을 포함한다. 구 인터페이스로 조용히 되돌아가지 않는다. [Microsoft 공식 설명](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2#addwebresourcerequestedfilter).
2. **미디어 버튼 미활성화**: 실제 HLS 매니페스트가 media.example.test의 master.txt로 전달됐다. 확장자/MIME만으로는 찾지 못했다. 정상 수신한 응답의 앞 4096바이트에서 #EXTM3U 및 #EXT-X-를 확인하고 HTTPS 소스로 보관한다. 원문 URL·본문·쿠키는 진단 로그에 저장하지 않는다.
3. **3초 조각을 전체 영상으로 오인**: mpv 기본 판정이 .txt를 일반 M3U로 열어 1080p_001.html 등 조각 파일을 별개 항목으로 재생했다. 실제 IPC duration=3.003초와 제목 변경으로 재현했다. 확인된 HLS에는 lavf/hls를 지정하고 allowed_extensions=ALL, extension_picky=0을 적용했다. 지원 프로토콜은 https/tls/tcp/crypto로 제한하여 로컬 파일 프로토콜을 열지 않는다.
4. **명령 송신만으로 성공 표시**: Tokio 비동기 duplex pipe, request_id 응답, time-pos/duration/video-params/audio-params/pause/volume/fullscreen 관찰을 추가했다. 전체 길이 일치 후에만 웹 미디어를 pause한다. 연결 실패·로드 오류·프로세스 종료도 상태로 전달한다.
5. **탐색 오차**: 기본 상대 탐색에서 36.124초→74.037초로 약 38초 이동했다. 버튼을 relative+exact로 변경했다. 수정 후 종영 표본을 pause한 상태에서 +30초 버튼을 눌러 62.104256초→92.134256초(30.030초) 이동을 확인했다. mpv 명령 성공 응답도 함께 확인했다.
6. **페이지 전환 시 중복 재생 위험**: 새 페이지로 이동할 때 소유 mpv를 종료하고 네이티브 상태를 초기화한다. 앱 종료 시에도 동일하게 정리한다.
7. **계층 위반 정리**: COM 응답·프로세스·IPC 전송·IPC 이벤트 해석을 각각 분리했다. 글꼴 파일 읽기와 엔진 버전 조회를 infrastructure로 이동하고 시작 인자는 main에서 조립한다.

추가 제어 관측: 방영중 pause 후 동일 시간 유지, resume 후 증가, volume 80→45 적용 및 성공 응답 확인. 종영에서는 fullscreen=false→true→false를 명령 응답과 속성 변화로 확인했다. -10초 정확 탐색은 92.134256초→82.165956초(-9.968300초)였다. 프레임/타임스탬프 단위의 약 30ms 차이가 남으며 프레임보다 정밀한 탐색을 보장하지 않는다.

## mpv 준비와 추적 가능성

[mpv 공식 설치 안내](https://mpv.io/installation/)가 연결하는 shinchiro 빌드를 작업 폴더에만 준비했다. 시스템 설치나 레지스트리 변경은 하지 않았다. 실행 버전은 mpv v0.41.0-1200-gdde067107, 빌드 날짜 2026-10-17이다.

- [다운로드 출처](https://github.com/shinchiro/mpv-winbuild-cmake/releases/tag/20261017): mpv-x86_64-20261017-git-dde0671072.7z
- 수신 아카이브 SHA-256: `2821D9EEF879215E99975FAE9616E1777F5E3747168EDD3A571C41E21B8ABBBC` (로컬 계산값이며 게시자 서명 검증이라는 뜻은 아니다).
- 아카이브·라이선스 자료는 `runtime/mpv/`, 검증용 실행 파일은 `target/debug/mpv.exe`에 있다. 두 경로는 버전 관리에서 제외된다. 배포판에 바이너리를 포함하는 작업은 별도다.

## 로그와 한계

`--record-validation`을 명시한 실행만 로컬 stdout 진단을 출력한다. `runtime/verification/*.jsonl`에는 호스트·MIME·확장자·시간·광고 슬롯 수·IPC 상태만 남긴다. 서명 미디어 URL·요청 헤더·응답 본문은 출력하지 않는다. JSONL은 앱을 닫고 완전히 flush한 뒤 집계한다. 검증 중 PowerShell 리디렉션 버퍼로 마지막 줄이 부분적으로 보일 수 있다.

현재 미완료: 물리 오디오 청취, 영상 전체 길이 시청, 장시간 안정성, 다른 회차/서버, 인증·쿠키 필수 소스, 만료 후 재리졸브, 단일 창 영상 합성, 사용자용 진행바 및 키맵. 검증 중 결과와 후속 항목은 [계획](../../../plans/player/plan-001-delivery.md)에도 동기화한다.

완전히 닫힌 JSONL을 `scripts/validation/summarize.ps1`로 집계한다. 네이티브는 첫 속성 구독 응답 수가 유지되는 구간만 연속 재생으로 계산하여 제어 명령 후의 탐색 점프를 제외한다. 요약에는 원본 진단 파일의 SHA-256과 표본별 판정이 포함된다. [기계 판독 결과](../../../evidence/playback/playback-2026-10-18.json)는 이 절차로 생성한다.

fmt 검사, Rust 테스트 7건, Clippy(-D warnings), 세 표본 집계 스크립트가 통과했다. 화면 근거로 [극장판 mpv 캡처](../../../evidence/playback/movie-native.jpg)를 함께 보관한다. 한 프레임 캡처만으로 재생 성공을 판정하지 않았으며, 시간·전체 길이·영상/오디오 속성을 함께 사용했다.
