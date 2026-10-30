---
id: evidence:kinekino/20261029114246
title: 원 Jobs 숏폼 취소 — 진행 근거
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 Jobs 숏폼 취소 — 진행 근거

## R190/R191 CLI 취소·재시도

`tests/jobs/clip_cancel.rs`는 고정 기존 host JSON을 SHA로 확인한 뒤 새 artifact/캐시 tmp 경로만 지정한다. 기존 프로젝트 publication2와 원 renderer/Jobs를 사용하고 실제 정상 CLI는 owned request process/영수증 경로로 실행한다. host는 같은 시험 프로세스의 SDK LocalHost이며 별도 정상 CLI host의 중단·재기동 시험은 아니다. 공통 시험 wrapper는 원 execution_identity를 전달하도록 보정했다.

R190 `input-r190`2453파일 inventory SHA256 `B55055D88BA1466DFD6AF11B99068BD03A2EEC1E6C3A230D4F10FC7625D71BDB`, SOURCE.md SHA256 `24469F99DC7CA9416518ABCD661E9B40D0BD513E11F2FF3C4C96D4FFDF39D703`. `2703`fmt/`2704`native strict/`2705`Jobs no-run 종료0. 고정 Jobs exe SHA256 `9BC9FC910612D284E466976F31EB08CCA19B88BDBE5C5AA390CE19CE78C076FD`, mirror `fixed-r190/native-core/`에 exe/PDB 보존.

`2706-source-sdk-r190-clip-cli-cancel` 실제1시험 통과(24.96초): 실제 CLI 7요청으로 submit/취소 requested+cancelling/최종 cancelled get/같은 submit 재전송의 created=false/stale retry 거절/수동 attempt2/완료 get을 검증했다. root `runtime/verification/source-sdk-r190-cli-cancel-r1/clip-cancel-retry/`를 보존한다. 원 verification.json의 cli_calls=8은 계수 메타데이터 오류이며 실제 owned request는7개다. 이를8요청 통과로 사용하지 않는다. `2707-source-sdk-r190-existing-cli-cancel`은 기존 자막 변환/범위 변환의 취소·발행 경쟁4경로 실제1회귀 통과다.

R191은 `preimage-r190-cli-retry-ack/` 보존 후 수동 retry 응답의 동일 revision 재전송을 여덟 번째 실제 CLI 요청으로 추가했다. `input-r191`2453파일 inventory SHA256 `B639A9D214DF2898D99FEE6E1009572F3F9393E6C8FA51C95E6FEB710226FFA2`, SOURCE.md SHA256 `E2DED82125548AC46D751D2C2C69D80B84D4C9B116B475A52F2CFD67F4355DF0`. `2708`fmt/`2709`native strict/`2710`Jobs no-run/`2712`default strict 종료0. 고정 Jobs exe/PDB는 mirror `fixed-r191/native-core/`, exe SHA256 `7205270A93D0337CDD3D9BBCE554D4D3B41D870D18F8D00062E0B67FAE900C6A`.

`2711-source-sdk-r191-clip-cli-cancel` 실제1시험 통과(24.49초). root `runtime/verification/source-sdk-r191-cli-cancel-r1/clip-cancel-retry/`에서 submit/cancel/cancelled-get/submit-ack-replay/stale-retry/manual-retry/retry-ack-replay/attempt-two-get의 실제 owned request8개를 확인했다. retry 재전송은 revision_conflict로 거절되고 attempt2가 완료됐다. 새 입력 부재·attempt1 증거 보존·원 worker 종료도 확인했다. `2713-source-sdk-r191-custody` 종료0,2453파일 content/identity 동일 확인. 실제 전송 ACK 유실은 주입하지 않았고 host는 in-process SDK fixture다. 제품 정책 변경은 없고 정상 제품 bins/전체190의 최신 근거는R186이다.

## R189 수동 재시도 확장

원 worker 정상 종료 후 같은 프로세스에서 다시 host를 열고, 취소된 작업이 자동 재시도되지 않았음을 확인한 다음 원 retry API로 attempt2를 요청하는 시험을 추가했다. 오래된 revision의 재시도 거절, 동일 입력/source SHA, attempt1 부분 증거 보존, attempt2 manifest와 작업 재열기 및 새 작업 입력 부재를 검사한다. 예기치 않은 crash·새 OS 프로세스·ACK 유실 복구를 입증하는 시험은 아니다.

수정 전 `preimage-r188-job-retry/` 보존. `input-r189`는2452파일, inventory SHA256 `D0471EB72AE79A161F6055AAF5112D43B556FA3108170D7A9930B1EA5B717705`, SOURCE.md SHA256 `CB4A896F4F530CE75718C423D7593C98F6DC838C3A4CF827438B748AB7DE7CD7`. `2697-source-sdk-r189-fmt`, `2698-source-sdk-r189-native-strict`, `2699-source-sdk-r189-lib-no-run` 종료0.

고정 exe/PDB는 mirror `runtime/verification/fixed-r189/native-core/`, exe SHA256 `D60F73999122A2A792FB3D6A6F8806F88B3743FC6DB4512A28C478F413B6FCB1`. `2700-source-sdk-r189-job-retry` 실제1시험 통과(20.43초). root `runtime/verification/source-sdk-r189-job-retry-r1/`에 취소·재시도·두 번의 원 worker 종료와 원 산출물을 보존한다. attempt2 manifest SHA256 `fc4600b816f1af4efde91eeb3de13d7f84756036d0747e7fe31bc20315845f4a`. 자동 재시도 부재·stale revision 거절·원 입력/source SHA 일치·attempt1 증거 보존·새 입력 부재와 attempt2 출력 재열기를 확인했다. `2701-source-sdk-r189-custody` 종료0,2452파일 content/identity 동일 확인.

범위는 실제 자막 렌더·검증을 마친 후 원 publication CAS 직전 취소다. 인코더 실행 도중 취소·ACK 유실·재시작 및 전체 정리 인수와 구분한다.

- private overlay `runtime/development/source-sdk-integration-r1`. 기존 제품 실행 정책은 변경하지 않았으며 실제 원 executor를 감싼 Rust 시험에서 publication 호출 직전만 정지한다. 원 execution_identity·deadline·취소 요청·reservation·verification을 그대로 전달한다.
- 원 Jobs client로 cancel을 요청하고 정지를 해제한 다음 원 worker의 최종 상태 및 종료를 확인한다. 전용 새 임시 루트는 C build-cache tmp 아래에 생성한다. 살아 있는 PNG/MP4 입력 2개와 최종 해제 후 부재를 확인하도록 작성했다. 원 부분 산출물과 첫 실패는 삭제하지 않는다.
- 수정 전 mirror `runtime/verification/preimage-r186-job-cancel/` 보존. R187 입력 2452파일 inventory SHA256 `8F5515EDE1D6984D0CEDEB44A9EBD8BFEEF69DE3F98EB183311EB26FE1B3DF51`.
- 첫 실패 `2691-source-sdk-r187-native-strict`: 시험의 DecimalU64 직접 Display 사용으로 E0277. 실패 파일은 `preimage-r187-cancel-format/`에 보존하고 숫자 필드 접근으로 수정했다.
- R188 입력 2452파일 inventory SHA256 `8F4107140A022B54CD4E2517A1A8338F28EEB7A8CABDF8756F32CBEC3F969375`, SOURCE.md SHA256 `7F307BBD74FA7348C6F68366F7952AC632AF8459DAA930237644E2A2F9E2B659`.
- 로그 루트 `runtime/verification/source375-datalab-b0-r1-logs/`. `2692-source-sdk-r188-fmt`, `2693-source-sdk-r188-native-strict`, `2694-source-sdk-r188-lib-no-run` 종료 0. 고정 exe/PDB는 mirror `runtime/verification/fixed-r188/native-core/`, exe SHA256 `410D48819A73DAB63462E7C5081035B1D635E47B20419B8E39401256E53157BA`.
- `2695-source-sdk-r188-job-cancel`: 실제 1시험 통과(10.38초). 원 Jobs `job-c157521da324431883bd5043785a4ba0` attempt1에서 자막 렌더/출력 전체 decode 후 원 CAS 전 취소 요청, 원 worker 종료 및 Cancelled 재열기를 확인했다. 새 입력 2개 해제·manifest 미생성·부분 clip/render/verification 증거 보존을 assert했다. root `runtime/verification/source-sdk-r188-job-cancel-r1/`의 live-inputs-before-cancel.json, cancel-request.json, last-job.json, shutdown.json, verification.json 및 원 artifacts/jobs를 보존한다.
- `2696-source-sdk-r188-custody` 종료 0, 2452파일 content/identity 동일 확인. 인코더 실행 도중 원 Jobs 취소·CLI 취소·ACK 유실·attempt 재시작은 이번 시험이 입증하지 않는다. `partial_cleanup_supported:false`를 유지한다.
- test-fixture-and-mock-build 및 code-change-review-verification 적용을 이어받아 실제 engine/원 Jobs와 시험 정지 장치를 구분한다. 기존 skill 출처·해시 정본을 유지한다.
- 현재 검토용 정상 bins/전체190의 최신 실제 근거는 R186이다. R188 시험 변경의 컴파일 성공을 제품 전체 인수로 계상하지 않으며 private overlay·전체 goal·보류를 유지한다.
