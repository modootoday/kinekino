---
id: evidence:kinekino/20261029114339
title: 검증용 작업 입력 — R185 진행 근거
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 검증용 작업 입력 — R185 진행 근거

## R186 PNG 입력 소유권 확장

- 수정 전 mirror `runtime/verification/preimage-r185-png-input/` 보존. 새 PNG를 파일 경로별로 발행하지 않고 64MiB 이하 PNG sequence를 단일 봉인 작업 파일로 만들고 원 encoder의 읽기 전용 stdin으로 전달한다. filter split/select로 원 run 순서와 표시 구간을 연결한다. overlay recipe는 `/2`, transport는 `owned_regular_stdin_png_sequence`로 구분한다.
- 고정 입력 `input-r186`: 2451 files, inventory SHA256 `E3004D546E0559E3270BBE5CD6611310311FFB955B9D7144F4A85B24F4E055A5`. 고정 lib exe/PDB는 mirror `runtime/verification/fixed-r186/native-core/`, exe SHA256 `9415673DA5DCAEA8E14E4D8D65575098C83802FFA51F7AB1B871250EEA74A581`.
- `2671-source-sdk-r186-fmt`, `2672-source-sdk-r186-native-strict`, `2673-source-sdk-r186-core-no-run` 종료 0.
- `2674-source-sdk-r186-overlay-pixels`: 실제 1시험 통과. 기존 고정 도구·공통 폰트로 24프레임의 [6,12), [18,24) 및 straight alpha 픽셀을 확인하고 overlay 해제 후 새 PNG sequence 파일 부재를 assert했다. root `runtime/verification/source-sdk-r186-subtitle-video-r1/`.
- `2675-source-sdk-r186-render`: 실제 1시험 통과. MP4 typed metadata/full decode/발행·손상 거절 회귀 및 verifier owner 해제 후 새 MP4 입력 부재를 assert했다. root `runtime/verification/source-sdk-r186-render-r1/`.
- `2676-source-sdk-r186-default-strict`, `2677-source-sdk-r186-jobs-no-run`, `2678-source-sdk-r186-normal-bins` 종료 0. 정상 exe/PDB는 mirror `runtime/verification/fixed-r186/native-bins/`. CLI SHA256 `473DDAF2E2B22891DD197DB967AF37BAE7EBCCBD3FA9A4CCA73CA5870C98C3CB`, GUI SHA256 `407B607FC7D6E051BE8528D5055F4E27AA0E6E9CFDDD6EA53332046ADD34E0AF`. GUI 수동 실행은 하지 않았다.
- 첫 실패 `2679-source-sdk-r186-subtitle-cli`: style 환경 핀 누락으로 렌더 전 종료 1. 로그와 `source-sdk-r186-cli-r1/`을 보존했다. 소스 변경 없이 명시 style 핀을 추가해 새 증거 root `runtime/verification/source-sdk-r186-cli-r2/`에서 재실행했다.
- `2680-source-sdk-r186-subtitle-cli-r2`, `2681-source-sdk-r186-plain-cli` 각각 실제 1시험 통과. 선택 자막16CLI 및 기존15CLI, 각각 typed catalog/source/submit와 원 Jobs2완료·정상 EOF를 확인했다. 선택 자막 manifest SHA256 `aa256e36f4ed69600803ec182a1c79ddec21fc91b1597a7c699493ec061b1806`. caption은 manual fixture이며 사용자 품질 승인으로 계상하지 않는다.
- `2682-source-sdk-r186-combined-no-run` 종료 0. `2683` Product145, `2684` Core18, `2685` Native26, `2686` OS1은 각 group-return의 execution_result와 producer_integrity가 모두 Ok이며 같은 SOURCE.md SHA256 `CB53E23E4491AC846D53DAA800627405516660301E1021322D0906285677AD68`을 참조한다. Meari/OS는 기존 고정 producer를 이번에 실제 재실행한 근거이며 새 컴파일로 표시하지 않는다. `2687-source-sdk-r186-fmt-check` 종료 0.
- `2688-source-sdk-r186-custody` 종료 0, 2451파일 content/identity 동일 확인. 전체 Jobs 취소/cleanup/복구/ACK 및 사용자 인수는 여전히 완료가 아니다. 기존 임시 파일과 원 증거를 소급 삭제하지 않는다.

- 범위: private overlay `runtime/development/source-sdk-integration-r1`. canonical 적용 전이다.
- 수정 전 증거: mirror의 `runtime/verification/preimage-r184-working-file/`. 기존 증거와 임시 파일은 삭제하지 않았다.
- 현재 입력: mirror `runtime/verification/input-r185`, 2451 files, inventory SHA256 `6BF92609914CEC4E5F13B927B24EA0B66DADF2923CEFDB64628892FA017DF389`.
- 새 작업 파일은 CREATE_NEW 원핸들 및 DELETE_ON_CLOSE로 소유한다. 봉인한 바이트와 실제 파일 식별을 검사하고 읽기 전용 regular stdin을 원 process supervisor에 전달한다.
- 검증용 MP4 probe/decode를 해당 입력으로 연결했다. 새 취소 시험은 부분 spawn 실패에서도 원 프로세스 소유자를 유지해 종료·영수증 처리 경로로 전달한다.
- 로그 루트: `runtime/verification/source375-datalab-b0-r1-logs/`. `2663-source-sdk-r185-fmt` 종료 0, `2664-source-sdk-r185-native-strict` 종료 0. 외부 blitz-paint의 기존 deprecated 경고는 별도 소유권으로 유지한다.
- 추가 근거: `2665-source-sdk-r185-core-no-run` 종료 0. 고정 lib exe SHA256 `0763B4618F2704DB3E40AAF39B453E59960064768B0363F5DA0D77F2EF948E2E`, mirror `runtime/verification/fixed-r185/native-core/`에 exe/PDB 보존.
- `2666-source-sdk-r185-working-file`: 실제 1시험 통과. 마지막 reader 후 삭제, 부분 쓰기 취소·용량 거절 후 새 파일 삭제, 뒤에 독립 생성된 파일과 원 증거 보존. 근거 root `runtime/verification/source-sdk-r185-working-file-r1/`.
- `2667-source-sdk-r185-stdin-cancel`: 실제 1시험 통과. 원 자식이 regular stdin을 소비한 marker를 관측한 다음 취소, 원 family empty·child exit·receipt 기록 이후 새 입력 해제. 근거 root `runtime/verification/source-sdk-r185-stdin-cancel-r1/`.
- `2668-source-sdk-r185-render`: 실제 1시험 통과. 기존 고정 Sintel 및 FFmpeg/ffprobe로 1080×1920/30frames/AAC/1초 출력의 fd 입력 metadata·전체 decode·게시/재열기/손상 거절을 검증했다. 근거 root `runtime/verification/source-sdk-r185-render-r1/`. 출력 SHA256 `4b198a94795d9e85814757994dfb7859df1d24e4757061edce6588e0039bdc2d`, transport `owned_regular_stdin`. 이 시험은 원 Jobs/CLI 전체 인수를 입증하지 않는다.
- `2669-source-sdk-r185-default-strict` 종료 0. native 및 기본 구성 strict가 모두 통과했다. 사후 `verify-current`도 종료 0, 2451파일 content/identity 동일을 확인했다. SOURCE.md SHA256 `B316B20536EDA3A5404EE1A6ED98A78252EECE54CC7287D53A627A926694F36C`.
- 미검증: R185 정상 bins 및 전체190, 원 Jobs/CLI 통합 회귀. R184 성공을 R185 실제 실행 성공으로 계상하지 않는다.
- 전체 PNG 임시 자원 정리·원 Jobs 취소·복구/ACK는 이번 코드만으로 완료되지 않는다. `partial_cleanup_supported:false`를 유지한다. 수동 GUI·추론·다운로드·WARP·push 보류도 유지한다.
- 적용 판단: 기존 test-fixture-and-mock-build 및 code-change-review-verification 적용을 이어받아 실제 원 프로세스/파일 수명 증거와 합성·컴파일 근거를 구분한다. 정확한 skill 출처와 해시는 기존 검증 정본을 유지한다.
