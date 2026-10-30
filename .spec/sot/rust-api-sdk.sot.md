---
id: sot:kinekino/rust-api-sdk
title: 공통 Rust API·SDK 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 공통 Rust API·SDK 계약

2026-10-26 정밀 경계: [host/명령별 주입·context 행렬](host-support-boundaries.md), [실행·저장 수명](../execution-storage-boundaries.md)을 함께 따른다. 공통 RequestContext 필드가 모든 명령에서 지원되는 것은 아니다. 이번 추가는 canonical 소스의 읽기 대조이며 source375 private 후계/새 호스트 전체190·정상 bins 실행 성공이 아니다.

기준: 2026-10-25. GUI `player.exe`와 AI 친화 CLI `player-cli.exe`는 같은 `ApplicationClient::execute(Request)`·application/domain 정책을 사용한다. Source Player는 가제다. 10월18~19일의 명령 수/지원 수와 “catalog·endpoint·durable host 전부 미구현”은 역사 checkpoint이며 현재 지원표를 대신하지 않는다.

## 현재 계약의 권위와 판정

명령 목록은 `src/application/api/capabilities.rs::COMMANDS`, wire는 `commands.rs`/`errors.rs`, 실행은 `api/mod.rs`와 기능별 유스케이스, discovery는 기능별 capabilities/schema 합성이 권위다. CLI 조립은 `src/bin/player-cli/runner.rs`와 explicit host 어댑터, GUI 조립은 원 SDK/LocalHost 소유자에 있다. 명령 이름/JSON schema 존재는 실행 구현이나 지원 완료를 의미하지 않는다. 기본 SDK의 unavailable, 명시 port 주입 시 capability, 실제 표본 검증, 사용자 승인을 분리한다. 정적 지원 개수를 제품 완성률로 사용하지 않는다.

`capabilities`/`schema`는 요청을 받은 해당 SDK instance 기준이다. 원 `ApplicationClientHandle`은 UI/CLI/transport 중립이며 GUI 자동 클릭을 요구하지 않는다. catalog의 `experimental`/`site_verified=false`는 실제 첫 표본 성공 이후에도 유지하며 전체 사이트·제공자 지원으로 올리지 않는다. 일부 오래된 capability 설명/CLI help의 category-only/media-resolve 제한은 직접 catalog adapter와 provider/combined host를 구분해 읽는다; 실행한 원 host receipt가 없는 경로를 지원으로 추론하지 않는다.

| 명령/기능군 | 실제 구현 조건 | 인수 범위 |
| --- | --- | --- |
| version/capabilities/schema | 공통 discovery. Default SDK는 없는 port를 unavailable로 보고 | 고정 catalogue 개수 대신 현재 COMMANDS/schema를 확인 |
| subtitles.convert/export_range | 명시 SubtitleStorage. UTF8 SRT/VTT·손실 확인·신규 파일. 지원하지 않는 context 옵션 거절 | [변환 계약](../../../verification/application/sdk-contract-fixes-2026-10-18.md); 전사/화자분리 등과 분리 |
| production.capture_frame/export_snapshot 및 카드/자막 내보내기 | 기능별 trusted capture/export port·store 필요. local frame과 native instance frame은 별도 조립 | schema의 입력 hash/PTS/crop/저장 조건 준수. 이름만으로 모든 production 작업을 지원하지 않음 |
| catalog.search/episodes.list/catalog.read_raw/episodes.inspect_resolution | 명시 catalog/archive, 또는 provider jobs의 저장 결과 읽기 | [카탈로그](catalog.md). airing/completed/movies 표본과 전체 pagination·검색/장르/제공자 보장을 분리 |
| jobs.submit/get/list/watch/cancel/retry, episodes.resolve | readonly store와 실행 worker capability를 구분. provider host/combined host는 명시 browser/profile/roots | [jobs](jobs.md), [초기 고정 CLI](../../../verification/application/jobs-2026-10-19.md), 최신 실제 원 host 경로는 아래 실제4 |
| playback.open/engine/status/close·제어 | native observer/control·shared episode session/원 engine_instance_id·권한 scope 필요 | [native 계약](../../../verification/application/native-episode-playback-2026-10-19.md). accepted/unobserved 제어 receipt는 적용 완료가 아님 |
| playback.previous/next/transition.observe/commit/release | 원 SDK의 검증한 회차목록·해석 작업·명시 원 transition commit/정리 | [실제3category+인접 왕복](../../../verification/application/product-review-next260-r3-actual-airing-2026-10-25.md): source1448 첫 표본3과 next/previous 왕복, 실제 mpv 진행/프레임 및 정리 |
| projects.create/inspect/save_revision 및 capture 변경 | 명시 ProjectStore, live capture 변경은 ProposalStore와 원 불변 anchor/CAS 필요 | [프로젝트](projects.md). locator 저장은 원 media 자동 적용이나 PTS 최신성 보증이 아님 |
| reviews.inspect/read_text_asset/record_decision/decisions | 명시 ReviewStore·package/assets 참조·불변 표본 결정 | [모델 검토](model-review-contract.md). 전체 모델 승인/자동 적용과 구분 |
| proposals.create/list/inspect/edit/accept/import_historical 및 output review | 명시 ProposalStore, historical reader/검증기 등 각 기능의 실제 port·revision 필요 | [제안](proposals.md). 기술적 accept, 원본 표본 판정, output 결정/이력, 실제 적용은 서로 다름 |
| segments 및 그 밖의 AI/models/production/project 명령 | 각 typed host/port와 기능별 capabilities/schema 기준; 이름만 있는 경로는 unsupported | 새로운 모델 실행·다운로드는 중앙 보류 유지. 전체 CLI 기능 인수 완료 아님 |

## 현재 실행 근거와 미완료

Windows named-pipe의 명시 endpoint와 `--serve-jobs/provider/native/playback`은 구현된 조립 표면이다. [전송](../../../verification/application/local-host-transport-2026-10-19.md), [CLI 조립](../../../verification/application/local-host-cli-wiring-2026-10-21.md), [권한](../../../verification/application/local-host-privilege-boundary-2026-10-19.md), [정리](../../../verification/application/local-host-shutdown-2026-10-21.md)를 따른다. in-process CLI에 live port가 없다는 사실은 명시 native/combined/원 GUI endpoint도 미지원이라는 뜻이 아니다. 자동 host 시작/재시도/fallback 없이 원 소유자와 작업 namespace를 사용한다.

최신 실제 재생 근거는 [source1448 실제4](../../../verification/application/product-review-next260-r3-actual-airing-2026-10-25.md)에 한정한다. 이 원 jobs/CLI/SDK 경로에서 3category 첫 재생·인접 회차 왕복·Ready/실제 진행·정상 close/worker join을 확인했다. 후속 UI/포커스/ManualRepair source 변경의 CPU 검사나 합성 SDK fixture를 실제 GUI 인수로 승격하지 않는다. [ManualRepair 실GUI 실패](../../../verification/presentation/normal-gui-mount261-manual-repair-2026-10-25.md)와 최신 원 gate 실패/미실행은 별도로 유지한다. [최종 ManualRepair CPU27](../../../verification/application/product-review-projection267-r3-cpu-2026-10-25.md)은 실제 SDK scratch의 상태/strict NodeSlot 및 소유 mount 회귀를 통과했다. 합성 raw-review UI 상태와 SDK-persisted output 결정을 구분하며, 원본/후보 비교·수정·명시 판정·이력 GUI 및 사용자 AI 품질 승인은 미완료다.

이 표는 실제 port/조건과 근거의 정규화이며 이번 문서 작업에서 CLI/모델/GUI를 새로 실행하지 않았다. 전체 무필터 테스트·모든 명령/플랫폼/작품/제공자 인수 완료로 읽지 않는다. 현재 CLI 문법·host 경계는 [CLI 정본](../../presentation/cli/automation-contract.md)을 따른다.

## 책임과 공개 표면

`presentation/gui → application/api → application usecases → domain`, `presentation/cli → application/api`를 따른다. SDK는 공개 Rust 라이브러리 파사드이며 GUI나 CLI를 실행해 기능을 대신하지 않는다. infrastructure는 application의 PlaybackPort, CatalogPort, ProjectStore, JobStore, ModelPort, ArtifactStore, EventStream 계약을 구현한다. OS·WebView2·mpv·GPU 자료형을 공개 SDK DTO로 노출하지 않는다.

현재 구조는 `src/lib.rs` 공개 모듈, `src/application/api/{commands,queries,events,errors,capabilities,schema,client}`, 파일 유스케이스 application 모듈, `src/bin/player-cli.rs`와 그 runner, GUI `src/main.rs`/presentation이다. Cargo의 두 bin은 player/player-cli다. `src/presentation/{gui,cli}` 세부 이관과 infrastructure/ipc host는 여전히 목표다. 공개 Request/Response, typed conversion/capture client, Event DTO가 있다는 것과 모든 port/event stream 구현은 별개다.

## 기능 단위

| 영역 | 명령·조회 | 반환·이벤트 |
|---|---|---|
| discovery | capabilities, schema, version | protocol/schema/SDK 버전, 지원·제약·모델 필요 상태 |
| catalog | search, list_episodes, resolve_episode | 불투명 WorkId/EpisodeId, cursor 페이지, 가용 재생 경로 |
| playback | open, play, pause, seek, set_volume, set_rate, select_track, switch_engine, previous, next, close | SessionId, generation, 상태 snapshot·변화. 실제 엔진 능력에 따른 거절 |
| segments | detect, edit, accept_skip, set_skip_policy | 원본 시간 구간과 revision, 자동/수동 OP·ED 정책 |
| subtitles | transcribe, diarize, align_speakers, edit_cues, edit_speakers, convert | Transcript/Cue/Diarization revision, 변환 손실 |
| production | download_media, capture_frame, render_short, export_cards, segment_subject, build_assets, package_projection | 검증 전 JobId, 검증 완료 후 ArtifactId |
| models | list, inspect, install, verify, remove | task/revision/정밀도/자원·가용성, 의존 작업 |
| jobs | submit, get, list, cancel, retry, watch | durable JobId, 상태, 단계별 진행·실패·산출물 |
| projects | create, inspect, save_revision, reconnect_source, export | ProjectId, Revision, 변경 충돌·소스 없음 |

각 API는 typed request/response와 validation policy를 공유한다. UI 전용 기능(창 배치·전체화면·컨트롤 자동숨김)은 별도 window capability다. CLI가 headless에서 화면 제어 성공을 가장하지 않는다. 자동 다음화·화자 분리 등 계획 기능은 `planned` 또는 `unsupported`로 노출하며 명령 이름이 있다고 구현된 것은 아니다.

## 명령·조회·동시성

명령은 `RequestContext(request_id, api_version, expected_revision?, idempotency_key?, timeout_ms)`와 구체 payload를 받는다. 조회는 부작용이 없으며 cursor·limit 기본50/최대200 제안, 정렬 기준과 다음 cursor를 반환한다. session generation과 문서 revision을 독립 검사한다. 오래된 입력은 `revision_conflict`로 거절하고 현재 revision을 반환한다. 상태 조회와 입력 사이 경쟁을 UI/CLI별 임시 예외로 해소하지 않는다.

소요가 긴 작업은 접수된 JobId를 즉시 반환하고 큐가 저장한 입력 revision을 고정한다. `Completed`는 출력 검증 후에만 사용한다. 검토가 필요한 제안은 검토 대기 결과이며 수락·내보내기는 별도 명령이다. 입력 대기 작업은 자동 무한 재시도하지 않는다. 기존 [작업 수명](../jobs/state-transitions.md)을 공유한다.

재시도 가능한 변경 명령은 idempotency_key를 지원한다. 같은 키·정규화 입력은 기존 결과/JobId, 다른 입력은 `idempotency_conflict`. 키는 저장소·사용자·작업 종류 범위에서 보관하고 유효기간/조회 가능 여부를 응답한다. 만료 후 중복 방지 보장은 하지 않는다. 시청 seek처럼 최신 의도가 우선인 명령은 중복 억제와 세대 검사로 처리하며 렌더 작업 중복 정책과 혼합하지 않는다.

## 값·오류·이벤트

ID는 의미 없는 문자열이며 제품명·파일 경로·URL에 종속하지 않는다. RequestContext의 request_id/idempotency_key는1..128 Unicode code points이고 byte나 grapheme 수가 아니다. request_id는 C0/C1 control을 허용하지 않는 공개 correlation 값이며 비밀을 담지 않는다. 정상 typed Request의 context validation 오류는 유효 ID를 보존하고 invalid/empty/oversize/control ID는 cli로 대체한다. invalid raw JSON/unknown field에서 ID를 부분 복구하지 않는다.

자막·제작 구간의 시간은 원본 PTS 기준 정수 마이크로초이며 범위는[in,out)다. `playback.status.position_us`와 `playback.seek.position_us`는 해당 native 엔진의 재생 시간이다. 원본 프레임 PTS와 동일하다고 추정하거나 프레임 정확도를 보장하지 않는다. JSON wire에서 `*_us`, revision과 sequence는 십진 문자열로 직렬화하여 정수 정밀도를 보존한다. SDK 내부는 검증된 정수 새 타입을 사용한다. 벽시각은 UTC RFC3339, 소요 시간은 monotonic elapsed다. path는 사용자 입력 문자열이고 정규화·검증은 storage adapter가 수행한다.

오류 공통 필드: `code,message,retryable,details,remediation,request_id`; code는 안정 영문, message는 표시 언어에 따라 변경 가능. typed ApiError/ErrorCode의 원 권위는 [media-contracts/src/errors.rs](../../../../../packages/media-contracts/src/errors.rs)이며 root API는 이를 재사용한다. ErrorCode::ALL과 schema catalog는 invalid_argument, unsupported, needs_input, not_found, revision_conflict, permission_denied, resource_exhausted, source_expired, engine_failed, cancelled, internal, timeout을 공유한다. backend 원문·서명 URL·쿠키·HF 토큰·셸 명령·비밀 payload는 details에 넣지 않는다.

이벤트는 `event_id,stream_id,sequence,emitted_at,request_id?,job_id?,session_id?,kind,payload`. 진행 이벤트는 합쳐 전송할 수 있으나 상태 전이·최종 결과는 보존한다. 느린 소비자는 bounded queue와 gap 표식을 받고 snapshot으로 복구한다. watch 재연결은 cursor 재개 가능 기간을 알려주며 지원하지 않으면 명시적으로 snapshot부터 시작한다. 출력 EOF를 작업 성공으로 취급하지 않는다.

## 버전과 인수

SDK SemVer, wire api_version, DTO schema_version, project schema_version, 모델 revision을 별개로 관리한다. 클라이언트와 호스트는 handshake로 공통 major와 capability를 확인한다. 같은 major에서 선택 필드 추가는 허용하고 기존 필수 필드 삭제·의미 변경은 major 변경. 알 수 없는 이벤트 kind는 보존/건너뛰고 알려진 상태를 훼손하지 않는다. enum unknown fallback과 필수 버전 차이는 별개다.

GUI/CLI는 같은 application/domain과 typed Request를 사용한다. 명시 host의 durable jobs·native playback·취소/정리·회차 전환은 아래 현재 지원표와 연결된 표본 근거 범위에서 구현되어 있다. 전체 플랫폼/명령/제공자 인수, GUI 시각·입력·원본/후보 비교 및 사용자 AI 품질 승인은 별도 미완료다. 접수/완료/저장/사용자 승인을 구분하고, 새로운 소스 조합이나 바이너리에 이전 성공을 자동 귀속하지 않는다.

관련: [로컬 런타임](../../infrastructure/ipc/local-runtime.md), [CLI](../../presentation/cli/automation-contract.md), [제품 식별](../../domain/product/identity.md), [결정](../../../../adrs/player/adr-009-shared-sdk-gui-cli.md).



