---
id: sot:kinekino/rust-2-2
title: 기능 맥락별 Rust 패키지 경계 조사
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 기능 맥락별 Rust 패키지 경계 조사

상태: **리서치·제안**. 2026-10-19 09:00–09:07 KST 작업 트리의 정적 관측이다. 패키지 추출·이동·workspace 변경을 실행하지 않았다. SDK/UI 담당이 동시에 구현 중이므로 아래 경로와 지원 상태는 실행 착수 전에 다시 확인한다. 이 문서는 기능 완료나 리팩터링 승인 기록이 아니다.

## 기준과 조사 범위

[루트 규칙](../../../AGENTS.md), [.spec 규칙](../../AGENTS.md), [공통 SDK 계약](../player/application/api/sdk-contract.md)을 따른다. 모든 개발 코드·새 검증 도구는 Rust다. 각 패키지 안에서도 domain/application/infrastructure의 책임을 구별하며 GUI `player.exe`와 CLI `player-cli.exe`는 동일 유스케이스를 사용한다. 제품 표시 이름과 안정적인 프로토콜·데이터 ID를 분리한다.

architecture 스킬의 단순 대안·비용·재검토 기준과 api-patterns 스킬의 공개 계약·버전·오류 일관성 원칙을 적용했다. HTTP API 도입을 가정하지 않는다. 현재 소비자는 Rust 호출, JSON CLI, 명시 named-pipe client다. 새 외부 검색·다운로드·추론·빌드·앱 종료·모니터 변경은 없었다.

조사 근거는 루트 `Cargo.toml`, `src/lib.rs`, 각 계층의 mod.rs, `application/api/{mod,client,capabilities,queries,events}.rs`, 실제 기능 모듈, `.gitmodules`, `packages/meari`의 workspace 및 core/native Cargo manifest다. 텍스트 검색은 직접 호출의 일부 증거이며 완전한 자동 dependency graph나 컴파일 결과로 취급하지 않는다.

## 현재 구조에서 확인한 사실

| 관측 | 근거 | 의미 |
| --- | --- | --- |
| 루트는 `kinekino-player` 단일 package, lib와 player/player-cli bin이며 `[workspace]`가 없다 | `Cargo.toml`, `src/lib.rs` | packages 디렉터리 생성만으로 workspace 멤버가 되지 않는다 |
| 공개 lib가 application/domain/infrastructure와 Windows presentation을 모두 노출한다 | `src/lib.rs` | 현재 공개 Rust 표면이 넓다. 새 facade를 만들 때 기존 소비자의 import 호환이 필요하다 |
| serde/sha2/url/adblock/png는 일반 의존성, eframe/wry/WebView2/windows/tokio 등은 Windows 의존성이다 | 루트 Cargo manifest | Windows의 SDK/CLI 빌드도 UI 기술 의존성이 연결될 수 있다. 실제 감소량은 추출 후 측정해야 한다 |
| packages/meari는 별도 Git submodule이며 그 내부 workspace에 packages/meari와 packages/meari-native가 있다 | `.gitmodules`, `packages/meari/Cargo.toml` | 루트 workspace에 무조건 재귀 포함하지 않는다. 기존 nested workspace·lockfile·소유권 보존이 필요하다 |
| PlayerSdk가 ports를 주입받고 ApplicationClient가 GUI/CLI 공통 typed facade를 제공한다 | `application/api/{mod,client}.rs`, `src/main.rs` | 이 경계를 유지해 기능 내부를 추출한다. GUI/CLI별 유스케이스 복제는 금지한다 |
| proposals와 model_review application이 api의 Request/ApiError를 사용하고 api는 기능을 dispatch한다 | `application/proposals/mod.rs`, `application/model_review/mod.rs`, `application/api/mod.rs` | api와 기능을 각 crate로 그대로 옮기면 순환한다. transport handler와 typed usecase를 먼저 분리해야 한다 |
| proposals validation이 model_review의 valid_id/valid_sha256을 가져온다 | `domain/proposals/validation.rs` | 검토 기능 자체에 의존시킬 이유가 없는 값 검증이다. 작은 값 계약으로 옮기거나 맥락별 다른 규칙임을 명시해야 한다 |
| presentation은 Browser/Mpv/VideoSurface/WindowActivity/WindowMetrics/NativeControlBinder 구체 타입을 참조한다 | `presentation/player_app.rs`, `presentation/mod.rs` | 폴더 이동만으로 계층 규칙이 충족되지 않는다. application 포트와 조립 책임 정리가 선행된다 |
| workspace_read는 ReadOrigin::Live/DesignSample을 구별한다 | `application/workspace_read/mod.rs` | 화면 read model 존재를 jobs/projects/AI 실행 구현으로 세지 않는다 |

Meari submodule은 디렉터리 모양만으로 판단하지 않았다. `git ls-files --stage -- packages/meari`는 mode **160000**, revision `26805a91058a875e0919a3c2368bba11ae551051`을 반환했고 `git -C packages/meari rev-parse HEAD`도 같은 revision이다. 작업 트리는 다른 담당의 수정이 있는 상태이므로 이 commit만으로 현재 dirty source 전체를 고정했다고 주장하지 않는다.

순환 위험의 정확한 경계는 **application↔SDK facade**다. `application/proposals/mod.rs:4`가 api의 ApiError/ErrorCode/Request를 가져오고, 같은 파일 `:248` execute가 Request를 받는다. 반대로 `application/api/mod.rs:153–155`가 proposals dispatch를 호출한다. domain이 api에 의존한다고 해석하지 않는다. domain 측 발견은 별도로 `domain/proposals/validation.rs:2`의 model_review 값 검증 헬퍼 의존이며, `domain/proposals/mod.rs`는 DecimalU64와 자체 모듈·serde를 사용한다. 작은 contracts 추출은 이 두 종류의 결합을 각각 해결하기 위한 것이다.

프로젝트 상태는 조율 정보와 디스크 관측을 구별한다. SDK 담당은 `src/{domain,application,infrastructure}/projects`를 구현 중이다. 09:07 전후 read 시점에는 세 디렉터리가 아직 없고 [projects API 계약](../player/application/api/projects.md)은 **다음 구현 계약**으로 존재했다. 따라서 미완료 기능을 구현 완료로 세지 않으며 이후 source가 생기면 이 조사 표를 갱신한다.

## 권장 경계와 우선순위

아래 `media-*`는 **후보 Cargo 이름**이다. 사용자 제품 표시명 변경과 독립된 기술 ID이며 아직 생성하지 않는다. 초기에 모든 후보를 한 번에 만들지 않는다. 각 기능 crate의 기본 공개 표면은 typed domain 값·application usecase·port다. 외부 기술 adapter는 feature 뒤 infrastructure 또는 별도 adapter crate로 두며, lib의 기본 기능이 Windows UI를 끌어오지 않게 한다.

| 순서·후보 | 실제 파일 / 현재 상태 | 공개 책임·입력/출력 | 의존 방향·플랫폼 | 유지 대비 분리 판단 |
| --- | --- | --- | --- | --- |
| 기반: media-contracts | `domain/api_values.rs`, `application/api/{commands,errors,events}.rs` 일부, `domain/product.rs` | DecimalU64, 요청 문맥, 오류/이벤트 envelope, 안정 ID·digest 값. 제품 식별은 한 책임의 별도 모듈 | serde 중심, UI/기능/OS 의존 없음 | 순환 해소에 필요한 최소 값만. 모든 DTO·도메인을 모으는 common/utils crate 금지. 제품 상수는 재사용 패키지에 하드코딩하지 않고 조립에서 전달 |
| 1: media-subtitles | `domain/subtitles/**`, `application/subtitles/{convert,storage}.rs`, `infrastructure/subtitle_files.rs` | SRT/VTT parse/render, 손실·speaker policy, ConvertSubtitles/ConversionResult, SubtitleStorage | contracts에만 의존. parser는 OS 비의존. 파일 adapter의 경로/신규 확정 별도 책임 | 가장 작은 실재 수직 기능으로 시범 추출 적합. 화자 분리/ASR 모델이 구현됐다는 뜻은 아님. path picker는 desktop 소유 |
| 2: media-proposals | `domain/proposals/**`, `application/proposals/**`, `infrastructure/proposals/**` | 불변 원문·generated, typed draft, CAS revision, create/inspect/edit/accept, StoredProposal/receipt, ProposalStore | contracts. api Request dispatch/schema는 SDK 측; 파일 adapter의 Windows guard는 선택적 infrastructure | S1/S2와 실제9건 안정화 후 추출. 모델 검토 승인과 분리된 사용자 편집 bounded context. 새 패키지로 기존 저장 bytes/JSON 형태를 바꾸지 않음 |
| 3: media-model-review | `domain/model_review/**`, `application/model_review/**`, `infrastructure/model_review/**` | 고정 package/assets 검증, 표본 결정·read_text_asset, ReviewStore/Session | contracts. filesystem adapter Windows 조건부. proposals에 의존하지 않음 | 품질 판정과 채택이 다른 책임임을 crate로 강화. 공통 ID 검증을 proposals가 이 기능에서 빌리지 않게 선행 정리 |
| 4: media-playback | `domain/{playback,playback_control,media,local_video}.rs`, `application/{native_playback,playback_control,playback_status,playback_route}` | session/engine generation, typed control, 상태·접수 receipt, control/status ports | contracts. mpv 프로세스/IPC·surface는 adapter 측, wry/egui 금지 | CLI/GUI 동일 제어를 강화하나 legacy facade·명령 완료 ACK 정리가 필요. accepted/unobserved를 적용 완료로 승격 금지 |
| 5: media-catalog | `domain/{episodes,navigation}.rs`, `application/{episode_queue,episode_transition,session}.rs` 일부 | EpisodePage/선택·이전다음/오래된 generation 거부; 향후 CatalogPort | contracts 및 최소 media source 값; 재생 제어는 조립 orchestrator가 연결 | UI 내부 동작은 존재하지만 catalog.search/episodes API 지원과 다름. source provider URL 정책과 일반 회차 정책을 분리한 뒤 추출 |
| 6: media-browser | `infrastructure/browser/**`, `infrastructure/adblock.rs`, `assets/{filters.txt,*observer.js,clean-page.js,media-control.js}` | navigation/resource filter, 제한된 browser observation, 웹 호환 playback adapter | catalog/playback ports 및 명시 provider policy. Windows wry/WebView2는 이곳에 격리 | 보안 경계가 뚜렷하므로 별도 adapter package 가치 큼. 기존 JS는 Rust 이관/최소화 추적 유지. 브라우저가 임의 셸·실행 경로를 제공하지 않음 |
| 7: media-production | `domain/frame_capture.rs`, `application/frame_capture.rs`, `infrastructure/frame_capture/**` | 우선 trusted local source→PNG capture port, bounded timeout/검증. 향후 export usecase | contracts 및 명시 media 값. FFmpeg/FFprobe는 Rust argv adapter | 현재 실제 기능은 capture 중심. 카드/short/download/2D·3D를 빈 facade로 한꺼번에 지원 표시하지 않음. 사용 가능한 기능부터 내부 하위 모듈화 |
| 후속: media-projects / media-jobs | catalog 명령·schema와 설계는 존재; projects API 새 계약과 담당의 구현 진행은 확인, 독립 source 디렉터리는 조사 시점 아직 없음. jobs 독립 구현도 미확인 | ProjectId/Revision/source/artifact refs/CAS; durable job state/cancel/watch | contracts에 각각 의존. job이 project engine을 가져오지 않고 immutable input ref 사용 | 현재 담당의 구현 우선순위를 막아 패키지화부터 강제하지 않음. typed port가 실제 생긴 뒤 분리. 큐 receipt/렌더 성공/산출물 검증 구분 |
| 후속: media-inference | 제품의 모델 실행 port·adapter는 목표; 실제 스파이크는 `tools` 계열과 별도 engine source에 존재 | task/revision/license/precision/model availability, bounded inference adapter | jobs/application port 구현. HF/engine/native GPU 의존은 adapter, domain에 노출 금지 | 스파이크 코드를 제품 SDK로 복사하지 않음. 새 추론은 사용자 지시대로 기능 완성 뒤. 모델/엔진 확정과 crate 분리 결정은 별개 |
| 조립: media-sdk / media-desktop / media-local-host | api facade / presentation·main / application·infrastructure local_host | 기능 dispatch+capabilities/schema, GUI, named-pipe framing·동일 client | SDK→contexts/contracts; desktop→SDK+adapters; local-host→client contract. 역방향 금지 | 최종 조립 단계에서 분리. 초기 루트가 호환 facade/두 bin을 유지해도 됨. CLI는 desktop package에 의존하지 않음 |

## 의존 규약과 순환 방지

권장 DAG는 `GUI/CLI/LocalHost → SDK facade → 기능 application/domain → 최소 contracts`이며 외부 adapter는 자신이 구현하는 기능 port에만 의존한다. 조립 entry가 facade에 adapter를 주입한다. `domain → application`, `기능 → SDK facade`, `SDK → presentation`, `meari → 제품 기능` 역참조는 허용하지 않는다.

현재 기능 application의 `execute(Request)`/schema 처리는 JSON 경계다. 이것을 SDK command adapter에 남기고 기능 crate는 typed create/edit/accept 등을 노출한다. 오류는 작은 공통 계약 또는 기능 고유 오류를 facade에서 ApiError로 매핑한다. 두 방식 중 첫 추출 시 하나를 택해 ADR로 기록한다. Request 전체를 모든 domain API에 전달하지 않는다.

`media`, `revision`, `digest`를 이름이 같다는 이유로 동일 타입으로 합치지 않는다. 동일 의미·직렬화·불변식이 입증된 값만 공유한다. cue source revision, project revision, engine generation, job sequence는 별도 newtype 또는 명시 필드로 유지한다. source time이 없는 문서/ASR에 가짜 cue/time을 부여하지 않는다.

Meari는 범용 UI IR·Rust/TSX 변환·native/Web adapter의 별도 소유 패키지다. 제품 테마·아이콘·화면·SDK 모델을 이 저장소에 넣지 않는다. 제품은 향후 안정 공개 API를 소비할 수 있으나 현재 egui UI를 곧바로 Meari로 교체하는 결정을 이 조사에서 내리지 않는다. Meari의 nested workspace를 루트로 평탄화하지 않는다.

## Workspace·공개 표면·버전

최초 추출 시 루트에 명시 `[workspace] members`를 추가하되 `packages/*` glob을 즉시 사용하지 않는다. 별도 소유 `packages/meari`는 제외 대상으로 검토하고 path 의존은 실제 하위 crate manifest를 향하게 한다. root Cargo.lock과 Meari Cargo.lock은 그 소유 workspace에 남긴다. 기존 tools의 독립 workspace를 일괄 편입하지 않는다.

새 crate는 우선 `publish = false`와 명시 export만 사용한다. crate 이름·dir·workspace version, 외부 API `api_version`, 저장 schema version, 제품 표시명은 별개다. 단순 경로 이동으로 JSON 명령·오류·capability·schema·파일명·직렬화 bytes를 변경하지 않는다. 기존 `kinekino_player::...` 사용자는 루트 호환 re-export 또는 명시 변경 목록으로 보호한다. Rust API 범위 축소는 실제 외부 소비자를 확인한 뒤 수행한다.

Windows feature matrix를 package별로 분리한다. default core 테스트는 eframe/wry/WebView2/windows/tokio를 요구하지 않는 방향을 목표로 삼되 현 코드의 모든 cfg를 자동 제거하지 않는다. 실제 비 Windows target 지원은 CI/toolchain 확보 후에만 주장한다. dependency tree와 빌드/메모리 시간은 이동 전후 같은 조건으로 비교하며 경량화 수치를 미리 약속하지 않는다.

## 대안 평가

| 대안 | 이점 | 비용·위험 | 권장 조건 |
| --- | --- | --- | --- |
| 현재 단일 crate+기능 모듈 유지 | 진행 중 API/UI 변경 충돌 최소, 가장 작은 변경 | 불필요한 기술 의존과 넓은 공개 표면이 지속 | 지금 기능 검증을 마무리하는 동안 유지 |
| domain/application/infrastructure를 각각 전역 crate로 분리 | 계층 컴파일 경계 명확 | 모든 기능이 공동 변경되며 맥락 경계가 약함, 거대 shared domain 유발 | 이 프로젝트의 기본안으로 비권장 |
| 기능 bounded context별 점진 추출 | 책임·소유권·선택 빌드가 명확, 공통 GUI/CLI 유지 | 순환 해소·호환 facade·추가 Cargo 관리 필요 | **권장: subtitle 시범 → proposal/review → playback/provider** |
| 모든 화면/작은 파일을 crate로 분리 | 작은 개별 단위 | dependency/Cargo/공개 API 관리 과다, 기능 변경에 다수 release 동반 | 비권장 |

리서치 결론은 **현재 API/SDK/UI 사용 가능한 체크포인트를 보존하고 작은 수직 기능부터 점진 추출**이다. 새 추론·모델 실측보다 제품 기능을 우선한다. 실제 이동 전에는 소유 담당과 source freeze·ADR·검증 기준을 확정한다. [잔여 계획](../../plans/architecture/package-extraction-2026-10-19.md)을 따른다.
