---
id: plan:kinekino/20261029113715
title: mpv 패키지 경계 후속 조사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# mpv 패키지 경계 후속 조사

상태: 읽기 전용 조사와 권장안. submodule 추가·clone·libmpv 채택·패키지 구현은 수행하지 않았다. 현재 커서·오디오 결함 수리의 실행 입력을 바꾸지 않는다.

사용자는 외부 mpv 바이너리 대신 packages 아래 저장소 gitlink 사용이 나은지 질문했다. 현재 Rust는 검증된 mpv.exe를 실행하고 IPC로 제어한다. Git submodule은 소스 커밋을 고정하며 빌드한 Windows 실행 파일의 공급·검증을 대체하지 않는다. 공식 근거는 [Git submodules](https://git-scm.com/docs/gitsubmodules), [mpv 임베딩 API](https://github.com/mpv-player/mpv/blob/master/DOCS/man/libmpv.rst)다.

## 책임 경계

| 책임 | 후속 분리 후보 |
| --- | --- |
| 저수준 프로세스·IPC·고정 argv·이벤트·소유 종료 | packages/media-mpv 후보. root SDK 역의존 금지 |
| backend 중립 control/status/track/typed launch 계약 | 기존 media-contracts 오류·Decimal 재사용. 독립 playback 계약의 필요 범위를 먼저 결정 |
| 회차 권한·목록 인접성·resolve job·commit·artifact 검증 | 제품 application에 유지. playback_session 전체를 공통 패키지로 옮기지 않음 |
| bounded diagnostics/PNG·불변 게시 | 명시 sink/검증 port. catalog naming·schema·증거 정책은 제품 glue에 유지하고 media-file-publication 재사용 |
| 제품 식별·실행 파일·parent window | composition에서 주입. 패키지 내부에 가제명·Request/ApplicationClient를 넣지 않음 |
| 바이너리 공급 | 버전·upstream revision·배포 출처·SHA·아키텍처·빌드 옵션·codec 의존·라이선스/notice를 별도 manifest로 관리 |

## 현재 소스 관측

ipc는 Tokio Windows named pipe와 serde_json을 사용한다. owned_startup은 Child/worker를 소유한다. control/status는 제품 playback DTO에, session_engine/diagnostics/capture는 제품 CatalogArtifacts·EpisodePlaybackSource·CaptureEpisodeFrame·EpisodeReplacementOwner 등에 의존한다. TrustedMpvExecutable의 SHA 검증은 공급 provenance manifest가 아니다. 단순 폴더 이동은 root→adapter→root 순환을 만들 수 있다.

## 인계와 미검증

배포 단순화에 대한 권장 순서는 검증된 mpv 엔진/필수 의존 파일 동봉→Rust 어댑터 패키지 경계 구현→libmpv 별도 통합 검증이다. 릴리스 빌드가 고정 배포물을 확보·해시 검증·패키징하고 사용자에게는 완성된 단일 설치물을 제공하는 후보를 제안했다. Git에 대형 실행 파일을 직접 커밋하거나 사용자 첫 실행에 임의 다운로드하는 방식으로 대체하지 않는다. 앱과 엔진은 함께 검증된 조합으로 업데이트한다. 이는 제안 상태이며 설치기/업데이트/라이선스 공급 manifest 구현·검증이나 submodule 채택 완료가 아니다.

SDK 담당이 향후 계약/port→저수준 adapter→제품 조립 순서로 구현·실제 CLI 회귀를 인수한다. 현재 진행 중인 커서·오디오 checkpoint 완료와 공유 입력 반환 후 구체화한다. mpv 소스를 직접 패치·빌드할 필요가 입증되면 third_party/mpv submodule을 검토한다. libmpv Rust 어댑터는 별도 수직 검증 대상이며 현재 독립 프로세스 성공을 임베딩 성공으로 해석하지 않는다. 기존 Rust 통일·단일책임·계층 규칙을 유지한다.
