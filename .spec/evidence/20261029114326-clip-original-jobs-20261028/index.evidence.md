---
id: evidence:kinekino/20261029114326
title: 숏폼 원 Jobs 연결 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 숏폼 원 Jobs 연결 검증

R161 첫 CLI 실제 실패 보존:2472 no-run 종료0/3분35초,2473 정상 bins 종료0. fixed-r161에 exe/PDB를 보존했다(CLI SHA A94BBD12B82D52807E5222679450F475D050783219FD97A3E496535CE2046480,GUI SHA670085F18AE692E684589072FBB2827C290091AB63449C6AA1625D8E8416C79B).2474 실제 시험은10.58초/실패다. CLI capability/schema/프로젝트 조회·submit→실제 Completed·get·동일 key replay는 통과했으나 wrong-project-pin의 실제 source_expired 종료8을 시험이5로 기대해 중단됐다. typed SDK 제출 전이므로 전체 시험 성공으로 계상하지 않는다. 원 host는 강제 종료 없이 EOF/pipe·sdk joined/completed_total1로 정상 종료했지만 시험은2를 기대해 secondary 오류를 기록했다. evidence runtime/verification/source-sdk-r161-cli-r1/clip-render와 실패 로그·첫 manifest91b12208fda5587c0ebaad762f56623a7445aabc1d0fcaa0ce1e09b4a8fb4cf1을 보존한다. preimage-r161-cli-exit에 원 시험을 보존하고 기대 종료값만8로 수정했다.2475fmt 종료0/2476R162snapshot 진행 중, 새 실제 성공은 아직 없다.

R161 입력 고정·strict 통과:2470 snapshot 종료0,2429파일/inventory D5757C6861A9CF6190DBA5BDF2AADA9C2818A4523BDA52DDC8871561401C8F52, SOURCE SHA C0AF4DFC0C621BB875B14FF06873DEF9E0735FC9100F7CAF4A6F7A5114DED477.2471 기본 all-targets strict 종료0.2472 lib/Jobs/CLI no-run은 exec session42212에서 실행 중이다. 원 handle로 종료를 확인하기 전 source 편집·중복 producer를 시작하지 않는다. 다음은 tests/jobs/clip_render.rs의 clip_render::configured_cli_and_typed_sdk_render_through_original_jobs_and_owned_eof ignored 실제시험이다. 기존 CLIP_RENDER_* 도구/source/inventory 환경과 CLIP_RENDER_PROJECT_ROOT=C:/workspaces/com.kinekino.player/runtime/verification/source-sdk-r159-jobs-r1/actual/projects, 새 JOB_EVIDENCE_DIR, 고정한 새 정상 JOB_CLI_EXE/JOB_CLI_SHA256을 공급한다. 입력/도구 핀은 R159 관측과 같지만 이번 CLI 실행 결과는 별도 증거로 기록한다.

R160 첫 strict 실패 보존:2467 snapshot은2429파일/inventory2CE029764482B81A1348AE9516C6B0A52A9BBEE9C293F0EFD651A049D6FE0316, SOURCE SHA2A55D1B2D8A3E42E092E764609D89E452B68C08C5DEE14DEBA4A28224C441B0C. 2468 strict 종료1은 새 CLI 시험의 field_reassign_with_default 1건이다. preimage-r160-typed-context에 원 시험을 보존하고 RequestContext 구조체 초기화로 수정했다. 2469 fmt 종료0,2470 R161 snapshot 진행 중. CLI 실제 실행/성공 근거는 아직 없다.

R160 CLI host 연결 WIP: preimage-r159-cli-host에 기존 api/client/schema/job request/host/jobs test를 보존했다. strict 로컬 host 설정 loader·기존 자막/card/clip registry·공유 clip store·typed submit_clip_render·jobs.submit schema와 실제 CLI/typed 시험을 추가했다. 2464~2466 fmt 종료0,2467 snapshot 진행 중. R159 실제 원 Jobs/전체gate를 이번 CLI 성공으로 계상하지 않으며 새 strict/no-run/실제 실행 전이다. QA 스킬의 fixture/원 worker/CLI 실행 범위 분리 판단을 유지한다.

R159 최종 사후 검사 완료: 2463 종료0, 2427파일 내용·identity 동일. 원 Jobs 실제1·strict/no-run/fmt·지정190·정상 bins·사후 custody가 모두 같은 R159 입력에서 완료됐다. 원 Jobs의 이 정상 여정만 인수 근거로 삼으며 제품 CLI 등록/전체 실패·취소·재시작·ACK·색·품질 승인은 계속 미완료다.

R159 통합 결과: 2457 Product145/2458 Core18/2459 Native26/2460 OS1의 각 group-return에서 execution_result=Ok(())/producer_integrity=Ok(())와 동일 SOURCE pin을 확인했다. 2461 정상2bins/2462 fmt check 종료0. Meari는 기존 고정 producer의 재실행이며 새 컴파일과 구별한다. 정상 exe/PDB는 fixed-r159/native-bins, CLI SHA 2E733117E8140919CF9B8D1B3A1E9BF35537293C52767609C37A50E4448EF785, GUI SHA 1B2971A3B3E3F7AA4171FF93F2AA4CB166885302AB26E1AE7C53139F81FF028D. 2463 최종 custody 진행 중. 아래는 당시 진행 기록으로 보존한다.

2456 R159 사후 custody 종료0:2427파일 내용·identity 동일. 이 검증 뒤 소스 변경 없이2457 지정 Product145를 시작했다. 정상 원 Jobs 성공은 CLI제품 노출/모든 실패행렬/사용자 품질 인수를 뜻하지 않는다.

R159 실제 원 Jobs 성공: 2454 no-run 종료0(4분), fixed-r159/native-core exe/PDB 보존, lib SHA C2157A3A6AC762058AC6E9F726D35DB00FB0999FF2CD76D0D5343BDC4B8FCD36. 2455 original_jobs_capture_render_publish_replay_and_reopen 실제1시험이 31.57초/종료0이다. 저장 프로젝트→engine-free capture→틀린 expected pin 거절→원 worker 실제 source/encoder/경계/렌더/출력검증/발행→Completed→동일 key 재사용→원 owner 정상 종료→저장 job/manifest 재열기를 확인했다. 실제 evidence는 runtime/verification/source-sdk-r159-jobs-r1/actual이다. job-c3e34b4f489f4a9ebfe6c5ec0d469ac4 attempt1, input SHA 5a105223f4fb01fd05f7ce69cb79c606b09b8bedffd6d13516139c9bdd0d6c1f, source blob cfe040c128f9dbfc7e867f320736f05a8a0d96ce7fa24439aab4d301d3d3b4c5, manifest 5fc00804924ec4b317a75d5b6e336d9b8e7a32c657b696899d335e27ff971fd6/1259bytes. shutdown receipt completed_total=1/error=null/secondary_count=0. CLI/색/품질은 false이며 전체 worker 취소·실패/attempt2·재시작/ACK 행렬은 아직 아니다. 2456 사후 custody 진행 중, 최신 전체190/정상 bins 기준선은 R157이다.

2453 R159 all-targets strict는 원 영수증 terminal=true/exit_code=0,40.688초로 통과했다. 2454 lib/Jobs/CLI no-run은 exec session 72375에서 계속 실행 중이며 원 handle 재관측을 유지한다. 종료 전 재시작/소스 변경을 하지 않는다. 다음 실행은 새 CLIP_JOBS_EVIDENCE_DIR 아래 original_jobs_capture_render_publish_replay_and_reopen ignored 시험이다. 환경은 기존 CLIP_RENDER_SOURCE/PROBE/PROBE_SHA256/ENCODER/ENCODER_SHA256과 CLIP_RENDER_INVENTORY_SHA256=2841eb1fa5abeff7bd342e6cbfa6b20eb1565388ffa0df1ed1bc532ca4629317을 사용한다. 새 전체190/정상 bins/custody는 아직 실행하지 않았다.

현재 R159 입력은 2427파일/inventory 695BA1BD130BBDE4D1D5CA7E28C385F18CB67AAE33D40549799D4F0B4BAC93A0, SOURCE.md SHA 1A862BFCE212D9BFA8F12DB445B8EE9C02B7948FF213EC01A9B25CA2EE406801이다. 2452 snapshot 종료0,2453 기본 all-targets strict 진행 중. 이 입력 고정 이후 source 편집 없이 검증한다.

2026-10-28 중앙 PM. [계약](../../player/application/production/clip-original-jobs.md)에 따른 private overlay WIP다. R157 전체 gate/검토 발행은 기존 근거이며 원 Jobs 성공으로 계상하지 않는다.

R158은 기존 infrastructure/mod.rs와 clip_render/mod.rs를 preimage-r157-jobs-bridge에 보존한 뒤 engine-free ClipRenderRequest/capture와 기존 JobExecutor/실제 retaining host 시험을 추가했다. 2447/2448 fmt, 2449 snapshot 완료:2427파일/inventory E0F331B2B0DA0A10A4DBCE03F563FF362D0BDC3000A319ED7722B0019F4B8313. 2450 strict는 실패했다. publish의 실제 반환값 RetainedClipArtifact를 PublishedClipArtifact로 오인해 필드 접근/Serialize 3오류(E0609/E0277)가 발생했다. 첫 로그/input-r158을 보존했고 preimage-r158-publication-return에 원 코드를 보존했다.

R159는 guard 객체를 원 수명 동안 유지하고 receipt()로 내부 영수증만 사용하도록 수정했다. 2451 fmt 종료0, 2452 새 snapshot 진행 중. 실행시험·SDKCLI 지원·제품 host 등록·새 전체 gate는 아직 미완료다. 원 소유권·WIP·source375/private overlay·C-only/heavy1·모든 기존 보류·AI 품질0/9를 유지한다.
