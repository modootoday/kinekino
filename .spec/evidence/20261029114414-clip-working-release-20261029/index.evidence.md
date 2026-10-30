---
id: evidence:kinekino/20261029114414
title: 원 작업 입력의 실제 해제 관측 — R201/R202
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 작업 입력의 실제 해제 관측 — R201/R202

`ClipWorkingFileRelease`는 원 핸들과 실제 파일 식별·root/ancestor guard를 유지하는 carrier다. 원 핸들 close 성공 후에도 reader가 남거나 원 파일이 관측되면 pending이다. 같은 경로의 다른 실제 FileID는 보존하고 완료로 관측하며, 경로 삭제·재귀 정리 API는 사용하지 않는다.

`PendingClipRenderRelease`는 해제 중인 MP4/PNG carrier와 원 candidate context를 함께 보관한다. 원 Jobs의 발행 성공 및 실패 경로가 이를 구동한다. 실패 원인·publication_committed/manifest 정보를 보존하고 정리 전 완료 ACK를 반환하지 않는다. 기존 부분 산출물은 삭제하지 않고 partial_cleanup_supported:false를 유지한다. Core owned-session/permit 연결은 여전히 미완료다.

안전 판단: [Rust FromRawHandle](https://doc.rust-lang.org/std/os/windows/io/trait.FromRawHandle.html)는 열려 있는 owned handle을 요구한다. [IntoRawHandle](https://doc.rust-lang.org/std/os/windows/io/trait.IntoRawHandle.html)과 [CloseHandle](https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-closehandle)을 대조해, R202는 OS close 오류 뒤 핸들 유효성을 가정해 File을 재구성하거나 재차 close하지 않는다. 불확실한 원 값·FileID·namespace guard를 보관하고 실제 이름/식별 상태를 관측한다. R201의 복원 분기는 실행 검증 전 대체했으며 그 원 파일과 입력을 보존했다. 시험의 close 오류 주입은 OS 호출 전 실패이며 실제 OS 오류 재현과 구분한다.

- 수정 전 `preimage-r200-working-release/`, R201 release 원본은 `preimage-r201-uncertain-close/`에 보존.
- R2012458파일 inventory SHA256 `2BFF420DAC9B9B84D3D6AE4D36B5BFE57E20B454FB200F9CABFE500858829884`, SOURCE.md SHA256 `1A41751B26D2C786D60BD8F7EB32911C7B092ABA37A487E1669B444FA7B3F92D`. fmt만 수행했고 성공 검증 근거로 계상하지 않는다.
- R2022458파일 inventory SHA256 `433A212CF60C99CC435CB2B88A202E04810F93863D68DB0C92C5D87C6E37D816`, SOURCE.md SHA256 `28B2C08D619163BA3A1633529CD93404D92FB1473DB31CEB8AFD128E336462DC`.
- 로그 루트 `runtime/verification/source375-datalab-b0-r1-logs/`. `2805`fmt/`2806`native strict/`2807`default strict/`2808`lib no-run/`2813`fmt check 종료0. 고정 lib exe/PDB는 mirror `fixed-r202/native-core/`, exe SHA256 `C0C1EBD37596CB3507E2B2AB2A588ED0803BD225DB2727D3EFE1F8E16F7750DE`.
- `2809-source-sdk-r202-working-release` 실제1시험 통과. OS 호출 전 close 오류 주입 뒤 원 owner·쓰기 거절이 유지됐고, 실제 원 handle close 후 reader가 남은 동안 pending·읽기 가능함을 확인했다. 마지막 reader 뒤 생성한 다른 FileID의 replacement는 보존하고 완료로 관측했다. 일반 원 파일의 실제 부재·poll 반복도 확인했다. root `runtime/verification/source-sdk-r202-working-release-r1/`의 input-facts/verification 및 보존 replacement 경로를 기록했다. 실제 OS close 오류 분기를 재현한 근거는 아니다.
- `2810` 원 Jobs 선택 자막 성공/발행/재사용/재열기, `2811` 자막 없는 원 Jobs, `2812` 취소·정상 재기동·수동 attempt2/부분 증거 보존 각1회귀도 통과했다. root `runtime/verification/source-sdk-r202-jobs-subtitles-r1/`, `source-sdk-r202-jobs-plain-r1/`, `source-sdk-r202-jobs-retry-r1/`. `2814-source-sdk-r202-custody`는 원 실행 receipt의 terminal=true/exit_code=0, stdout의 2458파일 content/identity unchanged로 종료를 확인했다. 원 handle은 이후 조회 시 소멸해 있었으며 재실행하지 않았다.
- 정상bins/190 통합 검토 빌드는R197이다. read-only lease의 모든 OS close 오류·pre-verification 실패의 전체 임시 자원·Core 전체 cleanup/중단/ACK 인수로 확대하지 않는다.
- 기존 QA fixture/변경 검증 skill 적용을 이어받아 실제 OS 관측과 fault 주입을 구별한다. Rust/ownership/private overlay·보류 및 원 증거를 유지한다.
