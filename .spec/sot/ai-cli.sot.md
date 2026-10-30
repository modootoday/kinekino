---
id: sot:kinekino/ai-cli
title: AI 친화 CLI 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# AI 친화 CLI 계약

기준: 2026-10-25. `player-cli.exe`와 GUI `player.exe`는 [공통 Rust API/SDK](../../application/api/sdk-contract.md)의 Request·application/domain 정책을 사용한다. 명령 목록이나 schema 존재, adapter 주입에 따른 capability, 실제 표본 실행, 사용자 품질 승인은 각각 다른 판정이다. 2026-10-18의 “파일 작업 외 전부 unsupported/endpoint 예약” 문구는 현재 계약이 아니다.

## 권위와 현재 지원 경계

명령 이름은 `src/application/api/capabilities.rs::COMMANDS`, 요청·오류는 `api/{commands,errors}.rs`, 실제 dispatch와 capability 합성은 `api/mod.rs` 및 기능별 schema, CLI 조립은 `src/bin/player-cli/runner.rs`와 각 host 어댑터가 권위다. GUI 정책을 CLI에 복제하지 않는다. `capabilities`/`schema`는 **호출한 해당 SDK/endpoint에 주입된 port** 기준으로 해석한다. 기본 SDK의 unavailable과 명시 host의 지원을 혼동하지 않는다. 고정 지원 개수는 완료율이 아니며 수동으로 유지하지 않는다.

| 표면 | 현재 계약 | 실제 근거와 제한 |
| --- | --- | --- |
| version / capabilities / schema | 공통 discovery. unsupported 명령도 schema/name이 있을 수 있음 | 실제 실행의 adapter·자원·입력을 보장하지 않음 |
| subtitles.convert / subtitles.export_range, production.capture_frame / production.export_snapshot | 명시 파일/캡처 어댑터·저장소에 조건부 | 기존 [SDK 검증](../../../verification/application/sdk-contract-fixes-2026-10-18.md)과 기능별 정본 범위. ffmpeg 캡처와 native instance 캡처는 서로 다른 조립 |
| catalog.search / episodes.list / catalog.read_raw / episodes.inspect_resolution | 명시 catalog/archive 또는 provider jobs. 카탈로그 discovery는 experimental/site_verified=false 유지 | [카탈로그 계약](../../application/api/catalog.md). 검색어/장르/정렬 또는 전체 제공자 지원을 뜻하지 않음 |
| jobs.submit/get/list/watch/cancel/retry, episodes.resolve | 명시 job store/worker 및 browser provider host. watch는 bounded event page | [jobs 계약](../../application/api/jobs.md). 모든 작업·모델을 실행할 수 있다는 뜻이 아님 |
| playback.open/engine/status/close, 제어, previous/next·transition observe/commit/release | native/combined host 또는 원 GUI endpoint의 공유 SDK·원 engine/scope가 필요 | [실제 3category+인접 왕복](../../../verification/application/product-review-next260-r3-actual-airing-2026-10-25.md). 각 첫 표본과 next/previous 왕복의 원 SDK/CLI·native 관측이며 전체 제공자 보장 아님 |
| projects.*, reviews.*, proposals.* | 각 명시 store와 기능별 port에 조건부 | [프로젝트](../../application/api/projects.md), [표본 검토](../../application/api/model-review-contract.md), [제안](../../application/api/proposals.md). 저장·채택·품질 판정은 별도 유스케이스 |
| AI/모델/제작의 그 밖의 명령 | 이름만으로 실행 지원을 추론하지 않음 | 해당 instance의 capabilities/schema와 명시 port/권한 확인. 모델 추론·다운로드의 중앙 보류 유지 |

최신 실제 재생 근거는 source1448 선택 조합에 귀속된다. 후속 ManualRepair 수정 조합의 CPU 통과/실패를 재생·전체 CLI 인수로 승격하지 않는다. 최근 정상 GUI의 ManualRepair 진입은 실패 기록이 있으며, 초기 mount·편집·원본/후보 비교·사용자 판정 인수는 별도 미완료다. 실제 CLI/SDK 판정 성공도 사용자 AI 품질 승인이 아니다.

## 호출과 wire

```text
player-cli version --json
player-cli capabilities --json
player-cli schema <command> --json
player-cli --input request.json --json
player-cli --input - --json
player-cli --input request.json --endpoint <explicit-current-user-endpoint> --json
```

`--input FILE|-`는 context/command/payload의 full Request JSON이다. positional 명령 또는 context flags와 중복하지 않는다. 최대 입력 크기는 `local_host::MAX_REQUEST_BYTES`를 공유한다. unknown flags/fields를 묵시적으로 우선하지 않는다. compact `--json`과 pretty `--human`은 같은 envelope/정책을 사용한다.

RequestContext는 request_id/api_version/expected_revision?/idempotency_key?/timeout_ms를 갖는다. request_id/idempotency_key는 1..128 Unicode code points이며 요청 ID는 C0/C1 control을 거절한다. 정상 typed validation 오류는 유효 correlation ID를 보존하고 invalid raw JSON에서 ID를 부분 복구하지 않는다. 비밀 payload·endpoint·원문을 오류에 반사하지 않는다. 각 명령이 revision/idempotency/timeout을 실제 지원하는지는 해당 schema로 확인한다.

응답·오류·십진 문자열 정수·exit mapping은 SDK 계약을 공유한다. 안정 ErrorCode는 invalid_argument/unsupported/needs_input/not_found/revision_conflict/permission_denied/resource_exhausted/source_expired/engine_failed/cancelled/internal/timeout이다. exit의 원 권위는 [media-contracts 오류 계약](../../../../../packages/media-contracts/src/errors.rs)이다. 매핑은 invalid_argument→2, unsupported 또는 needs_input→3, not_found→4, revision_conflict→5, permission_denied→6, resource_exhausted→7, source_expired 또는 engine_failed→8, cancelled→9, internal→1, timeout→10, 성공→0이며 출력 실패도1이다. 접수 receipt, 엔진 적용 관측, 작업 완료, 저장, 사용자 승인 상태를 구분한다.

## 로컬 host와 수명

Windows의 명시 `--endpoint`는 현재 사용자 LocalHost로 요청을 전달한다. 예약 unsupported 표면이 아니다. 전송 deadline/단절 뒤 원 요청이 실행됐을 수 있으며 자동 fallback/retry하지 않는다. 로컬 root 옵션과 remote endpoint를 혼합하지 않는다. [전송 계약·실행](../../../verification/application/local-host-transport-2026-10-19.md), [CLI 조립](../../../verification/application/local-host-cli-wiring-2026-10-21.md), [권한 경계](../../../verification/application/local-host-privilege-boundary-2026-10-19.md)를 따른다.

명시 `--serve-jobs`, `--serve-provider`, `--serve-native`, `--serve-playback`은 서로 다른 의존성을 조립한다. runner/help와 해당 host의 Rust validation이 요구하는 절대 root·전용 profile·trusted mpv path/SHA·예산을 지킨다. 자동 host 시작·브라우저 클릭·광고 차단 해제로 미지원 경로를 우회하지 않는다. 원 stdin EOF/정상 close·worker join·잔여 소유자·오류를 별도 확인하며 단순 EOF/child 생성은 기능 성공이 아니다.

`jobs.watch`는 bounded durable event page이며 지속 NDJSON stream이나 `--wait`/detach 지원을 의미하지 않는다. cancel은 원 attempt, retry는 expected_revision 등 jobs schema를 따른다. 수명·재시작 Interrupted·취소 ACK와 GUI 창 닫힘을 혼합하지 않는다. [정리 계약](../../../verification/application/local-host-shutdown-2026-10-21.md)을 따른다.

전체 무필터 회귀·모든 명령/엔진·플랫폼·제공자·모델의 인수는 아직 완료되지 않았다. 이 문서 갱신을 위해 CLI/host/모델을 새로 실행하지 않았으며, 정본은 코드 계약과 링크된 기존 증거 범위만 반영한다.

