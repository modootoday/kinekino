---
id: plan:kinekino/20261029113629
title: 다운로드 공통 executor와 CLI 연결 최소 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 다운로드 공통 executor와 CLI 연결 최소 제안

2026-10-23 읽기 전용 제안. 현재 실행 권한은 execution-leases.md를 따른다. 새 제품 코드·Cargo·S/T·네트워크·다운로드 실행0이다. 선행 입력은 typedcancel effective8 `4B6E2EED3AD68C14CD66529D13890C9BDDCD747221B289FB0AFF72F264E41842`의 실제 인수이며, 현 sourcecancel33 실패를 해결했다고 간주하지 않는다.

## 현재 코드에서 확인한 연결점

- `src/application/download/mod.rs`는 ports/transfer만 등록한다. owned_session은 실제 시험의 path mount이고 생산 등록은 없다.
- `src/infrastructure/download/mod.rs`는 StagedDownloadOutput만 공개한다. 실제 probe/원 source의 제품 조립·입장이 없다.
- `src/infrastructure/browser_provider_jobs.rs`의 기존 registry는 subtitle/catalog/episode resolve executor만 보유한다. 원 BrowserHost와 jobs가 같은 RetainingProviderJobsHost에 있고 부분 시작 실패·shutdown carrier가 이미 있다.
- `src/bin/player-cli/provider_host.rs`는 이 원 host의 JobsApi와 catalog를 같은 PlayerSdk에 연결한다. `jobs.submit/get/watch/cancel`은 이미 SDK와 CLI의 공통 경로다. 별도 CLI 다운로드 정책/새 standalone worker/새 RPC transport는 필요하지 않다.
- jobs capability/schema는 실제 executor operation 목록에서 생성된다. 최상위 `production.download_media` 명령은 현재 unsupported이고 생산 route가 없다. jobs operation 등록과 직접 명령 지원은 구별해야 한다.

## 중요한 source 수명 의존성

현재 `BrowserEpisodeResolveJobs::run`의 BrowserDocumentSession은 함수 범위를 벗어나면 Drop에서 원 ticket을 취소·release하려 한다. 저장된 resolution artifact/URL/SHA로 그 원 live request의 권한과 소유자를 복원할 수 없다. 과거 resolve 결과를 받아 다음 job에서 BrowserOriginalRequest를 다시 만들어 성공시키는 설계는 배제한다.

실제 provider 다운로드는 같은 원 download attempt 안에서 resolve→admit→read→verify→publish가 진행되거나, 명시 보존된 동일 원 source owner가 있어야 한다. 기존 BrowserDocumentSession Drop의 bounded 시도는 원source 물리 ACK가 아니다. 실제 source opener는 core가 소유한 뒤 원 browser/source를 시작하고 부분 open 실패·취소·unwind에서도 동일 owner를 보존해야 한다. 이 의존성은 최소 조립을 미루는 이유가 아니라 실제 positive CLI 인수에 필요한 명시 조건이다.

## 최소 후속 변경 후보

아래 exact 경로는 제안이며 새 편집 배정이 아니다. 실제 구현 전에 preimage와 source manifest를 별도 동결한다.

| 경로 | 책임 |
|---|---|
| 신규 `src/application/download/request.rs` | strict typed request descriptor·정규화·source/destination 식별. URL/직렬화 token/tool path는 실행 권한이 아님 |
| 신규 `src/application/download/job_executor.rs` | 기존 JobExecutor와 OwnedAttemptExecutor를 연결하고 DownloadOwnedExecutor에 위임. source blob SHA/원 scope binding을 확인, legacy execute는 명시 거절 |
| 기존 `src/application/download/mod.rs` | 기존 owned_session과 위 2책임의 모듈 등록만 |
| 신규 `src/infrastructure/download/product_host.rs` | 신뢰된 source preparation/probe admission/output factory를 원 host에 연결. 미입장 상태는 None/Unsupported, 임의 JSON으로 factory를 생성하지 않음 |
| 기존 `src/infrastructure/download/mod.rs` | 실제 probe와 product 조립의 한정 등록. 미채택 HTTP 전체를 자동 합성하지 않음 |
| 기존 `src/infrastructure/browser_provider_jobs.rs` | 동일 BrowserHost를 소비하는 승인된 download factory가 있을 때만 기존 ExecutorRegistry에 추가. 원 부분 시작 실패·shutdown 순서와 carrier 보존 |
| 신규 `tests/download_cli.rs` | 실제 CLI transport의 request/response·job terminal·저장/취소/오류 관측. 아래 fixture와 생산 host 결과를 별도 계상 |
| 신규 `tests/support/download_cli_host.rs` | Rust test-only source/host 주입과 원 owner 관측. 원 실행 파일/기존 CLI 조립 경로를 소비하며 정책 복제·항상 성공 mock 금지 |

CLI provider_host의 기존 `jobs.capability().operations`와 SDK `jobs.submit` 경로는 그대로 재사용한다. 현재 계획에서는 새 CLI flag/직접 production 명령/API COMMANDS 일괄 supported 변경0이다. 필요해진 ready/schema 표시 변경은 실제 의미 차이를 먼저 제시한다. 기존 `--serve-jobs`의 subtitle/VAD 선택과 `--serve-provider`의 root 선택을 자동 혼합하지 않는다.

source DTO의 실제 필드·최대 크기·timeout 상한은 기존 provider 선택 계약과 도구 입장으로 확정한다. 아직 살아 있는 source owner 없이 직렬화된 새 handle registry를 먼저 만들지 않는다. output은 기존 원 디렉터리 guard/no-overwrite/CAS/verify_published를 소비한다. GUI 소비자는 같은 ApplicationClient::submit_job를 사용하며 기능 정책을 복제하지 않는다.

## 실행 인수와 증거 구분

1. typedcancel gate 실제 인수 후 위 공통 executor/registry의 최소 조립을 구현·검증한다. default host에서 다운로드 operation은 미등록이며 jobs capability/schema/submit의 Unsupported가 일치해야 한다.
2. Rust 통합 fixture가 승인된 원 로컬 media/probe/source를 주입해 **같은 SDK/CLI 조립 경로**를 실행한다. 정상은 원 입력 hash/bytes와 같은 출력·단일 publication/Completed, 손상 입력은 probe 오류·출력0, 취소는 원 expected_attempt/Cancelled와 같은 source close 후 출력0, 기존 대상은 no-overwrite·원 bytes 불변을 실제 관측한다. 에러 code/details/request_id/remediation, idempotency 충돌·원 first error·EOF/family·incomplete cleanup carrier도 보존한다. 이 positive는 test-only source이고 생산 provider 다운로드 지원으로 계상하지 않는다. 테스트에만 있는 wire/env를 배포 CLI 권한 경로에 추가하지 않는다.
3. **생산 CLI positive의 최소 조건**은 원 BrowserHost의 source admission과 구체 physical-close source host가 같은 download attempt에 연결되는 것이다. 고정 실제 provider 입력 하나에서 정상/오류/취소/저장과 원 source/frame backpressure/oversize를 확인한 뒤에만 해당 operation을 제공한다. 원 HTTP response custody·DNS/runtime ACK·full stream 공백은 [기존 후속 계획](ffprobe-original-provider-checkpoint.md)의 별도 계약을 적용한다.

전체 HLS/resume/remux/re-resolve/restart/modes/batch와 원제품 CLI/자막/제작/AI 범위는 그대로 남는다. direct200 또는 로컬 fixture를 전체 완료로 대체하지 않는다. 실제 CLI positive가 불가능한 경우 미지원 원인과 정확한 source host 의존성을 기록하고 조립/독립시험 통과를 생산 지원으로 표시하지 않는다.

## 스킬 적용

modoo.today `modootoday/software-qa/test-fixture-and-mock-build` SKILL SHA `a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100`을 원 실제 계약·오류·소유자 정리와 test-only/생산 증거 경계에 적용했다. 기존 원 jobs API/host와 fixed 입력을 재사용하며 새 언어의 정책·검증 코드는 도입하지 않는다. 현재는 읽기 제안으로 타입/실행/성능 측정0이다.
