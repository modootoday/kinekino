---
id: sot:kinekino/native-2
title: 검증 회차 결과의 소유 native 재생
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 검증 회차 결과의 소유 native 재생

## 입력과 수명

명시 native host가 공통 `EpisodePlaybackSessions`와 `EpisodePlaybackEngine`을 조립한다. 독립 native CLI는 `--serve-native --catalog-root ABS_DIR --mpv-executable ABS_MPV_EXE --mpv-sha256 SHA256`로 시작하며, 실제 provider와 native를 함께 조립하는 host 계약은 [SDK 정본](api/sdk-contract.md)을 따른다. 제품 GUI도 같은 application 계약을 주입받는다. [source1448 실제4 검증](../../verification/application/product-review-next260-r3-actual-airing-2026-10-25.md)은 원 제품 GUI owner·CLI·공통 SDK의 airing/completed/movies/adjacent 경로를 확인했다. 이는 정상 제품 GUI 전체 인수나 이후 소스의 실제 성공으로 확대하지 않는다. GUI 수동 조작으로 CLI 지원을 대체하지 않는다.

`playback.open`은 resolution_artifact_id/paused/volume_percent를 받는다. 공통 결과 reader로 [저장 매니페스트](episode-resolution-result.md)의 원문·요청 소유·HLS를 재검증한다. URL·헤더·실행파일·셸·임의 engine 옵션을 외부 payload로 받지 않는다. process_started는 성공적인 미디어 로드나 decode가 아니다. URL 유효기간·쿠키 이전·참조 media 유효성은 저장 결과만으로 입증하지 않는다.

한 host의 active 엔진은 하나다. 기존 active가 있으면 새 open을 거절하고 기존 엔진을 보존한다. 제어·close·native capture는 실제 engine_instance_id를 같은 잠금 안에서 대조한다. 일반 close 뒤 새 open은 허용한다. EOF/shutdown은 영구 closed gate를 세워 이미 시작된 SDK 호출의 늦은 open도 차단한다. 모델·외부 프로세스 탐색·GUI 자동 클릭은 없다.

close는 소유 child의 종료·wait와 IPC 작업자 join이 성공한 뒤 반환하도록 구현했다. OS try_wait/kill/wait 실패는 Active를 유지하고 오류로 반환한다. write await·운영체제 종료 오류를 고려하면 모든 종료가100ms 안에 끝난다는 보장은 없다. 실제 hung 증거는 없다. host는 신뢰된 수명 guard를 소유하며 SDK clone은 종료된 host의 권한을 연장하지 못한다.

## 관측과 제어

`playback.engine`은 instance ID·resolution artifact ID와 native 관측을 함께 반환한다. `playback.status`는 기존 상태 DTO다. latest 스냅샷 하나만 유지해 미조회 이벤트가 무한히 쌓이지 않는다. publication age는 native property 자체의 정확한 생성 시각이 아니다.

play/pause/seek/set_volume의 native host payload에는 engine_instance_id가 필요하다. 접수 receipt의 completion은 unobserved이며 별도 native property로 적용을 확인한다. 이전 GUI control adapter의 현재 인스턴스 명령과 이 명시 수명 계약을 동일한 완료 보장으로 표시하지 않는다. rate·tracks는 아래 후속 정본, previous/next와 transition observe/commit/release의 원 adjacent 왕복은 source1448 실제4 근거에 한정한다. Studio 왕복 뒤 fresh Viewing으로 복귀한 경로는 별도 route 수명 검증 대상이다.

## Viewing과 Studio의 route 수명 후보

창의 parent lifetime 종료와 명시 Viewing route retirement를 구분한다. parent close는 영구 closed이며 다시 열지 않는다. route retirement 후보는 원 shared `TransitionSlot`의 새 begin/observe/commit/release를 일시 차단하고 원 pending resolution의 job ID·attempt를 유지한다. 동일 JobsApi로 취소 요청을 확인한 뒤 원 작업의 terminal 상태를 관측하기 전에는 fresh route 재개를 허용하지 않는다. 취소 요청 자체는 worker 종료 증거가 아니다. 오류는 pending owner를 보유하고 final end가 원 job ID를 반환한다.

원 child의 engine→surface cleanup과 pending transition settlement가 모두 끝난 뒤에만 retire complete다. 새 Viewing의 실제 checked frame/활성 route와 동일 router·opaque pause ticket을 대조한 뒤 원 slot을 재개한다. 기존 transition sequence는 재사용하지 않는다. 정리된/소유하지 않은 transition handle은 기존 `NotFound`, foreign/stale pause identity는 `RevisionConflict`로 구분한다. parent close 비트는 이 재개 과정에서 변경하지 않는다.

이는 [283 CPU 검증](../../verification/application/product-review-route283-cpu-2026-10-25.md)의 후계 후보 계약이며 현재 소스의 coherent56·정상 bins·실제4 receipt 전 완료로 표시하지 않는다. 메모리 slot/router·inert engine 시험, 원 coordinator lease 시험, 실제 OS/native 재생 및 사용자 GUI 인수는 각각의 근거 범위를 유지한다.

## 현재 프레임 PNG

`production.capture_frame`의 native 입력은 engine_instance_id/output_path다. 기존 local source_path/source_pts_us/output_path 입력은 기존 FFmpeg 포트가 제공될 때 유지한다. 두 포트를 함께 주입한 SDK는 request schema 전체를 oneOf로 구분해 각 context 규칙을 보존한다. 미지원 엔진에서 native capture를 지원한다고 표시하지 않는다.

소유 인스턴스 잠금 안에서 고정 screenshot-to-file/video 명령을 별도 IPC 연결로 보낸다. JSON 배열 API의 기본 property 비확장 계약을 사용하며 사용자 텍스트를 flat 명령이나 셸로 합치지 않는다. 로컬 출력 부모와 새 staging directory를 고정하고 새 PNG 파일만 게시한다. command reply를 기다린 뒤 최대64MiB·기존 픽셀 상한의 PNG 전체를 디코드·검증하고 SHA256을 반환한다. 기존 출력은 덮어쓰지 않는다. 실패 staging은 원인 근거로 보존할 수 있다.

현재 capture deadline은4초이며 timeout 뒤 자동 재시도하지 않는다. position_before_us/after_us는 관측된 엔진 시간 범위다. frame_time_exact/subtitles_included/window_capture/color_parity_verified는 false다. 원본 source PTS·GUI screenshot·자막/색상 출력 일치·HDR 품질로 확대하지 않는다. PNG 확보와 native 시간 진행은 실제 frame/재생 관측이며 전체 참조 media·프레임 정확도·화질 승인과 별개다.

## 원문 진단

native_engine_diagnostic은 HTTP 응답이나 browser capture가 아니다. private catalog JSON에 stream의 원본 바이트를 base64로 보존하고 원본 SHA/bytes·truncated/read_failed/complete를 구분한다. 공개 native 응답에는 원문 URL·query·로그 본문 대신 immutable artifact 참조와 요약을 반환한다. 명시 catalog.read_raw가 원문을 재읽는다. 부분 관측은 complete로 표시하지 않는다.

r2 준비본은 stderr만 수집하므로 모든 mpv 로그 보존이 아니다. [공식 mpv 로그 구현](https://github.com/mpv-player/mpv/blob/master/common/msg.c)의 기본 stdout 출력을 확인한 뒤 r3에서 두 스트림을 별도 수집하도록 수정했다. 각 stream512KiB·합계1MiB prefix를 보존하며 상한 뒤에도 계속 drain한다. stdout/stderr의 원본 SHA/bytes와 각 truncated/read_failed를 private JSON에 기록하고 공개 DTO에는 요약을 반환한다. complete는 두 worker 종료와 두 stream의 누락/읽기 오류 없음이 확인된 경우만 true다. 내부의 모든 메시지·명령 trace를 빠짐없이 기록한다는 뜻은 아니다. stdout worker join도 close 성공 조건이다. [공식 screenshot 계약](https://mpv.io/manual/stable/#command-interface)과 [JSON IPC](https://mpv.io/manual/stable/#json-ipc)는 로컬 고정 mpv의 실제 인수를 대체하지 않는다.

## 검증 상태

[native-r1](../../verification/application/native-resolution-cli-2026-10-19.md)은 전체293·strict/bins와 미디어 없는 host EOF를 통과했다. r2는 첫 Clippy unnecessary_unwrap 및 harness 참조 누락을 보존했고 수리 strict16951ms0·전체295pass18ignored317882ms0다. [r3 실제 인수](../../verification/application/native-episode-playback-2026-10-19.md)는 fmt0·strict14342ms0·전체295/18ignored315544ms0·두bin1633ms0 및 actual1pass44.39초다. 실제 resolve→native1920x1080 PNG·시간 진행→CLI 네 제어의 독립 적용 관측→close/reopen/stale ID/EOF를 통과했다. 청취·전체 회차·색상/PTS·GUI 시각 일치·전체 참조 media 검증은 미완료다. [후속 계획](../../../plans/player/native-resolution-handoff.md)의 전체 범위를 유지한다.
## 속도·트랙 후속 검증

[옵션 r3 실제 CLI 정본](../../verification/application/native-options-cli-2026-10-19.md)은 공통 rate/select_track API와 실제 audio/video disable·복원, 복원 PNG 및 기존 제어·소유 수명을 인수했다. subtitle 양성·청취·회차 전환은 이 근거에 포함하지 않는다. 아래 초기 native 계약의 unsupported 설명은 당시 checkpoint 범위와 구별한다.

