---
id: sot:kinekino/sot-002-product-architecture
title: 제품 및 아키텍처
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [player]
verify: manual
---
# 제품 및 아키텍처

2026-10-26 통합: [배포·신뢰·legacy 이관 경계](release-and-trust-boundaries.md), [실행·저장](../application/execution-storage-boundaries.md), [의존성 순 계획](../../../plans/player/design-completion/implementation-order.md)을 연결한다. 아래 최초 프로토타입의 일괄 미이관 문구는 역사이며 현재 구현/지원은 SDK host/기능별 정본과 선택 소스로 대조한다. 전체 SDK/CLI 구현 완료라는 뜻은 아니다.

## 확정 요구사항

2026-10-18 추가 확정: 사람용 GUI player.exe와 AI용 CLI player-cli.exe를 함께 제공하고 [공통 API/SDK](../application/api/sdk-contract.md)를 기능 설계 기준으로 사용한다. UI 전용 핸들러에 정책을 두지 않는다. [런타임 연결](../infrastructure/ipc/local-runtime.md)과 [제품 식별](../domain/product/identity.md)에 두 실행 경로와 가제 변경을 분리했다. 기존 프로토타입은 아직 이 구조로 이관되지 않았으며 [구현 계획](../../../plans/player/plan-006-sdk-gui-cli.md)을 따른다.

2026-10-18 방향 갱신: **네이티브 플레이어를 기본 UI로 확정**했다. 웹은 탐색·소스 확인·호환 재생을 담당한다. 아래 초기 구현 설명은 현재 프로토타입 기록이며 네이티브 우선 UX를 이미 구현했다는 뜻이 아니다. [최신 UX](../presentation/viewing/sot-011-native-first-studio-ux.md), [제작 기능](../application/production/sot-012-media-production.md), [HF·노트북 AI/투영](../application/analysis/sot-013-local-ai-projection-assets.md)이 확장 범위의 정본이다.

사용자 지시: Windows 우선, UI까지 Rust, 네이티브 컨트롤, 광고 제거가 핵심. Rust 호스트 내부의 경량 Chromium/WebView 호환 플레이어도 광고 차단을 포함해 구현한다. UX는 PotPlayer를 참고한다. 단일책임·모듈화·계층형 구조가 최상위 규칙이다.

초기 지원 대상은 Windows x64. Windows 최소 버전과 ARM64 지원은 배포 시험 후 확정한다. Rust UI는 egui/eframe, 브라우저는 Wry/WebView2, 네이티브 디코딩은 mpv를 사용한다. UI와 코어가 Rust라는 뜻이며 영상 코덱과 Chromium까지 Rust로 재작성한다는 뜻은 아니다.

## 계층과 책임

```text
src/
  main.rs                         # 조립과 실행
  domain/navigation.rs            # 허용 탐색·미디어 후보 값 정책
  domain/media.rs                 # 응답 본문으로 확인한 미디어 소스
  domain/playback.rs              # 검증 가능한 웹 재생 관측 값
  application/session.rs          # 이벤트 계약·재생 세션 상태
  application/native_playback.rs  # 네이티브 상태와 전체 영상 일치 판정
  infrastructure/adblock.rs       # ABP 규칙 해석·요청 매칭
  infrastructure/browser/mod.rs   # WebView 수명·탐색·표시 영역
  infrastructure/browser/requests.rs # COM 요청 관측과 차단 응답
  infrastructure/browser/responses.rs # 응답 헤더·제한된 본문 확인
  infrastructure/mpv/mod.rs       # 소유 프로세스 수명
  infrastructure/mpv/ipc.rs       # 비동기 named pipe 송수신
  infrastructure/mpv/events.rs    # mpv 응답·속성 이벤트 해석
  infrastructure/diagnostics.rs   # 명시적으로 켠 로컬 진단 출력
  infrastructure/fonts.rs         # Windows 글꼴 읽기
  presentation/player_app.rs      # Rust 화면과 사용자 동작
assets/
  filters.txt                     # 프로젝트 자체 시드 필터
  clean-page.js                   # 제한된 호스트의 cosmetic 처리
  media-control.js                # 모드 전환 시 웹 미디어 일시정지 전달
  playback-observer.js            # iframe 미디어 관측을 상위 프레임으로 전달
```

도메인은 외부 엔진에 의존하지 않는다. Application 계약은 인프라 구현을 참조하지 않는다. Presentation은 어댑터의 공개 메서드를 사용하고 COM/셸 API를 직접 호출하지 않는다. 초기 단일 바이너리의 조립은 UI 생성 시 수행하며, 카탈로그·저장소가 추가되면 application 포트와 주입을 추가한다. 파일 분리만을 위한 빈 계층은 추가하지 않는다.

## 모드 및 상태

`Browsing → OpeningEpisode → WebPlaying → MediaCandidate → NativeOpening → NativePlaying / Failed`.

- 웹 모드는 원본 플레이어를 호스팅하며 광고 차단이 기본 활성이다. 새 창 요청은 허용된 재생 도메인만 기존 영역으로 전달한다.
- 미디어 소스는 관측된 HTTPS 요청의 확장자 또는 허용된 응답의 `#EXTM3U`·`#EXT-X-` 내용으로 판정한다. 관측 표본의 실제 매니페스트는 media.example.test의 `.txt` 응답이었다. blob·HTML 주소를 파일 주소로 추측하지 않는다.
- 네이티브 재생은 사용자가 선택하고 mpv가 실제 열고 진행한 뒤 성공으로 판정한다. blob·HTML URL을 자동 전달하지 않는다.
- 네이티브 실패 시 오류를 표시한다. 광고 차단을 자동 해제하거나 다른 주소를 추측하지 않는다.
- 웹 모드의 키·탐색·자막은 웹 플레이어 기능에 따른다. 네이티브 버튼과 동일 능력을 보장한다고 표시하지 않는다.

## 목표 데이터 계약

| 타입 | 필드 및 규칙 |
|---|---|
| CatalogQuery | category, text?, genres[], year?, sort, page; 로컬 상태와 사이트 매핑 분리 |
| Work | canonical_url, title, poster?, category, metadata; 제목만으로 ID 생성 금지 |
| Episode | canonical_url, work_url, label, ordinal?, published_at?, previous_url?, next_url? |
| PlaybackTarget | episode_url, intermediary_url?, embed_url?, observed_at |
| MediaSource | url(메모리 전용), referer?, content_type?, expires_at?, session_id, evidence; 인증 요구는 별도 명시 |
| PlaybackCapabilities | seek, speed, subtitles, volume, time, fullscreen 각각 지원 여부 |

위 표는 목표 계약이다. 현재 BrowserEvent·Session·NativePlayback, domain의 MediaSource·WebPlayback을 구현했다. MediaSource는 URL·Referer·HLS 판정만 보유하며 만료·쿠키·세션 세대 계약은 후속 작업이다. URL은 실제 href를 보존한다. 원문 URL의 `(2)`, `/`, `?` 등은 작품명의 일부일 수 있어 제목으로 경로를 합성하지 않는다.

## 동시성과 수명

WebView2 COM과 egui는 소유 UI 스레드에 둔다. 이벤트 채널로 상태를 갱신한다. 네트워크 차단 판정에 외부 I/O를 넣지 않는다. 비동기 응답 완료 콜백에서 매니페스트의 앞부분 4096바이트만 해석하며 본문을 저장하지 않는다. mpv의 named pipe 송수신은 별도 Tokio 런타임에서 비동기로 수행한다. 응답 request_id와 시간·영상·오디오 속성을 관찰한다. 앱 종료 및 페이지 전환 시 소유 mpv를 정리한다. 네이티브의 전체 영상 길이·영상 크기·시간 증가가 웹 영상과 일치한 뒤 웹 미디어에 pause를 전달한다. 프로세스 시작만으로 웹 재생을 끊지 않는다.

## 초기 구현과 목표의 차이

현재 UI는 Rust 범주·주소·차단 배지, 공통 진행바·시간·볼륨·웹/mpv 전환·앱 전체화면이다. 네이티브 영상은 전용 자식 HWND에 출력하며 별도 최상위 창이 없다. `infrastructure/video_surface.rs`가 Win32 소유권·bounds·포커스를, `presentation/transport.rs`가 컨트롤 렌더링과 입력을 담당한다. 카탈로그 본문은 광고 차단 WebView다. 전체 Rust 카탈로그, DB 이어보기·자동 다음화는 후속이다. 실제 범위는 [단일 창 검증](../../verification/presentation/sot-006-integrated-ui-validation.md)과 [엔진 검증](../../verification/playback/sot-005-playback-validation.md)을 따른다.

## 회차·자동숨김 구현 갱신

우측 회차 패널·이전/다음화·P 토글 및 전체화면 3초 비활동 컨트롤 숨김을 구현했다. 최신 구현과 실제 관측은 [회차/자동숨김 검증](../../verification/presentation/sot-007-episode-controls-validation.md)을 따른다. 종전 미구현 문구는 이 기능에 대해 갱신되며, 전체 키맵·자동 다음화·다중 DPI 회귀는 후속이다.

## 제작·분석 확장 계약

재생 세션과 제작 작업 수명을 분리한다. 현재 회차에서 미디어 ID/트랙/원본 PTS를 복사해 초안을 만들고, 이후 회차 전환은 이미 시작한 작업 입력을 변경하지 않는다. SourceHandle은 수명 있는 세션 참조이며 프로젝트에 서명 URL·쿠키를 영속화하지 않는다. 작업은 검증된 선언적 ProjectRevision을 받아 임시 출력→검증→원자적 완료로 진행한다.

계층 간 방향은 `presentation → application → domain`, infrastructure는 application 포트를 구현한다. 제작 정책(구간·변환 손실·출력 규격), 작업 큐(상태/자원/취소), 기술 어댑터(전송/FFmpeg/HF/추론/파일), 화면을 나눈다. main은 조립만 한다. 현재 어댑터를 presentation에서 직접 쓰는 초기 코드와 확장 목표의 차이는 단계적으로 해소하고, 새 기능에 플랫폼 API 직접 호출을 늘리지 않는다.

재생·제작·분석 공유 단위는 원본 미디어/시간/에셋 ID이며 하나의 거대 전역 상태가 아니다. [제작 모듈 계약](../application/production/sot-012-media-production.md)과 [HF/AI 모듈 계약](../application/analysis/sot-013-local-ai-projection-assets.md)에 목표 경로·검증 게이트를 기록했다. 모델은 선택 설치·로컬 실행이며 사용자 확인에 따라 현 i9/64GB/RTX8GB 호스트를 동급 노트북의 주 실측 기준으로 삼는다.
