---
id: sot:kinekino/package-extraction-owner-handoff-2026-10-19
title: 담당 세션용 패키지 추출 인계
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 담당 세션용 패키지 추출 인계

상태: **부분 변경·미검증·구현 중단**. PM의 구현 허가와 SDK lease ACK를 받은 뒤 아래 변경을 실행했으나, 후속 사용자 정정으로 서브에이전트는 리서치만 담당하게 됐다. 즉시 코드/Cargo/lock/파일 이동/포맷/빌드/시험 작업을 중단했다. 이 문서는 남은 변경을 실제 구현 담당 desktop 세션에 인계하기 위한 관측 기록이다. 완료나 회귀 통과로 취급하지 않는다. 이후 서브에이전트는 구현을 재개하지 않는다.

기준은 [경계 조사](package-boundaries-research-2026-10-19.md), [후속 계획](../../plans/architecture/package-extraction-2026-10-19.md), [최상위 규칙](../../../AGENTS.md)이다. 사용자 정정 이후 이 인계 문서 외 코드나 기존 문서는 수정하지 않았다.

## 실제 변경

신규 Rust 패키지는 3개, 신규 파일은 20개다. 명시 workspace/path dependency와 최소 기존 공개경로 re-export를 만들었다. 기능 정책 변경을 의도하지 않았지만 컴파일·회귀로 아직 입증하지 않았다.

| 신규 파일 | 원래 책임·변경 내용 |
| --- | --- |
| packages/media-contracts/Cargo.toml | private package, serde/serde_json 의존 |
| packages/media-contracts/src/lib.rs | errors/api_values 공개 모듈 |
| packages/media-contracts/src/errors.rs | 기존 application/api/errors.rs의 오류/exit code 구현 복사 |
| packages/media-contracts/src/api_values.rs | 기존 domain/api_values.rs DecimalU64와 시험 복사 |
| packages/media-file-publication/Cargo.toml | private package, media-contracts 의존 |
| packages/media-file-publication/src/lib.rs | 기존 infrastructure/new_file.rs 복사, 오류 import만 contracts로 변경 |
| packages/media-subtitles/Cargo.toml | private package, contracts/publication/serde/serde_json 의존 |
| packages/media-subtitles/src/lib.rs | domain/application/infrastructure 계층 공개 |
| packages/media-subtitles/src/domain/mod.rs | subtitles 모듈 |
| packages/media-subtitles/src/domain/subtitles/mod.rs | 기존 domain 타입, DecimalU64 import를 contracts로 변경 |
| packages/media-subtitles/src/domain/subtitles/parser.rs | 기존 parser, DecimalU64 import만 변경 |
| packages/media-subtitles/src/domain/subtitles/format.rs | 기존 형식 코드 복사 |
| packages/media-subtitles/src/domain/subtitles/render.rs | 기존 render 복사 |
| packages/media-subtitles/src/domain/subtitles/tests.rs | 기존 단위 시험 복사 |
| packages/media-subtitles/src/application/mod.rs | convert/storage/path_picker 모듈 |
| packages/media-subtitles/src/application/convert.rs | 기존 JSON conversion handler·typed DTO, 오류 import 변경 |
| packages/media-subtitles/src/application/storage.rs | 기존 storage port/limit, 오류 import 변경 |
| packages/media-subtitles/src/application/path_picker.rs | 기존 picker port/TextPathOnly, 오류 import 변경 |
| packages/media-subtitles/src/infrastructure/mod.rs | subtitle_files 모듈 |
| packages/media-subtitles/src/infrastructure/subtitle_files.rs | 기존 local-file adapter, shared publication crate를 호출 |

기존 파일 변경은 7개다.

| 경로 | 현재 내용·영향 |
| --- | --- |
| Cargo.toml | 명시 workspace members/default-members는 루트+위3패키지, resolver3. Meari 및 기존 일부 tools를 exclude. 3개 path dependency 추가 |
| src/application/api/errors.rs | `pub use media_contracts::errors::*;` |
| src/domain/api_values.rs | `pub use media_contracts::api_values::*;` |
| src/domain/subtitles/mod.rs | `pub use media_subtitles::domain::subtitles::*;` |
| src/application/subtitles/mod.rs | `pub use media_subtitles::application::{convert, path_picker, storage};` |
| src/infrastructure/subtitle_files.rs | `pub use media_subtitles::infrastructure::subtitle_files::*;` |
| src/infrastructure/new_file.rs | `pub use media_file_publication::{io_error, publish, publish_before};` |

기존 domain/subtitles 하위 파일과 application/subtitles의 convert/storage/path_picker는 삭제하지 않았다. 현재 mod가 새 패키지를 재공개하므로 원래 하위 구현은 컴파일 대상이 아니다. 검증 후 중복 원문의 처리는 구현 담당이 결정한다. presentation/main, projects, Meari, tests/examples 원본은 이번 변경에서 편집하지 않았다. Request/transport/API_VERSION/schema dispatch가 SDK 내부에 남아 있으므로 모든 기능의 crate 순환이 해소됐다고 주장하지 않는다.

## 보존 자료와 실행 상태

`runtime/quality-review/package-extraction-r1/before/`에 실행 직전 현재 dirty source의 src/tests/examples 전체와 Cargo.toml/lock을 복사했다. 이 사본은 기존 r3 bin의 컴파일 source와 동일하다는 뜻이 아니다. initial-pins.txt는 당시 root Cargo/lock과 기존 `proposal-sdk-r3/final-bin/player.exe`, `player-cli.exe` hash 조회다. 기존 bin을 새 폴더에 복사하지 않았으며 actual-nine-r1 두 store도 수정하지 않았다.

| 파일 | 관측 SHA256 |
| --- | --- |
| before/Cargo.toml | fcfd38c71f56d9799636f2523dc6ae841eca1bc30987d01e3f53b2394412f2ad |
| current Cargo.toml | 15cfae5a1c5ce3959e65bd42af3edc4f0b3187d8f3c8cbd0cd3a09fda9847115 |
| before/Cargo.lock 및 current Cargo.lock | 5c665f6c6ac47e7fa0e9056c4d733d9206e3241fad46f258a2295d54f9d8265f |

현재/before lock hash가 같으며 이 agent는 lock을 명시 편집하지 않았다. 위 hash는 인계 직전 관측이지 미래 담당 변경 이후 값은 아니다. 전체 파일별 SHA manifest는 아직 생성하지 않았다.

실행한 검사는 **cargo fmt --all --check 1회**, terminal **exit1**이다. 로그는 `runtime/quality-review/package-extraction-r1/logs/fmt-initial.log`에 있다. 보고된 차이는 media-subtitles/src/application/convert.rs의 import 순서다. cargo fmt 쓰기 모드, build/test/clippy는 실행하지 않았다. 따라서 새 패키지의 컴파일 성공도 미확인이다. 마지막 read-only CIM에는 agent 소유 cargo/rustfmt가 없었고 기존 R4 cargo16440(PP72080)만 있었다. 그 작업은 수정·중단하지 않았다.

## 구현자 재개 기준

1. PM에게서 공유 Cargo/API/source lease와 제품 검사 슬롯을 인수한다. 현재 파일과 이 목록을 대조하고 다른 담당 변경을 보존한다. 중단된 agent가 프로세스를 계속 실행한다고 가정하지 않는다.
2. 세 패키지의 공개 경로/타입 identity, workspace 명시 members/exclude, 모든 실제 tools의 독립 workspace, include/asset 경로를 검토한다. Meari nested workspace/submodule은 변경하지 않는다.
3. 기존 fmt 실패 원문을 보존하고 소유 범위 포맷을 수정한다. source manifest를 고정한 뒤 workspace fmt/test/clippy all-targets-Dwarnings 및 GUI/CLI bins/examples를 한 슬롯에서 실행한다. 신규 의존이 생긴 lock은 그 전후를 보존한다. 비 Windows 지원·경량화 성과를 추정으로 주장하지 않는다.
4. 기존 tests/subtitle_conversion.rs의 실제 CLI 왕복/한글/원문불변, 손실 needs_input→명시 허용, overwrite/hardlink 거부, 경쟁 publication, typed ApplicationClient worker, resource/encoding/format/context 거부를 유지한다. 외부 Sintel corpus ignored 시험은 실제 자산 존재와 의도된 gate를 확인한 경우만 별도 실행한다.
5. shared publication은 subtitle뿐 아니라 frame_capture와 model_review/session에서도 소비한다. timeout/nooverwrite/error mapping·부분 임시파일 처리·검토 저장 경계를 전부 회귀한다. 새 패키지 때문에 production이 subtitle에 의존하지 않게 현재 publication 경계를 유지한다.
6. S1 저장 receipt readback, S2 docref/원문SHA/JSON pointer/LF 정책, 기존 actual9 immutable input/technical store를 보존한다. 새 bin pin으로 별도 출력 디렉터리에서 기존 Rust importer를 실행해 needs_repair·명시fixture수정·채택·fresh CLI receipt와 원문/raw SHA를 대조한다. 이것은 모델 품질 승인이나 신규 추론이 아니다.
7. 고정 GUI bin과 fixture를 UI 담당에 인계한다. 실제 main monitor 조작·자막 conversion·손실 확인·저장/오류·기존 제안 재조회는 UI 담당의 실제 증거로 확인한다. 이 agent는 GUI/모니터를 변경하거나 조작하지 않았다.
8. 통과한 범위와 남은 범위를 verification/SOT/plan/ADR에 반영한다. 사용자 정정으로 실제 구현은 desktop 담당 세션만 지속한다. 이 부분 변경을 검증 없이 완료로 표시하거나 reset/삭제로 숨기지 않는다.
