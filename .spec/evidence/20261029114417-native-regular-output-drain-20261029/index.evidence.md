---
id: evidence:kinekino/20261029114417
title: 원 native 일반 파일 출력의 EOF — R203/R205
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 native 일반 파일 출력의 EOF — R203/R205

Core owned-session 연결의 선행 공백: 기존 `NativeProcessProgress`는 원 child exit/family Empty/receipt만 관측하고 일반 파일 출력의 EOF를 관측하지 않았다. `pipe_eof_observed:false`는 계속 사실이며 이를 true로 바꾸지 않는다.

private overlay의 `frame_capture/process/owned/output_drain.rs`는 원 `RawFile::open_read`의 실제 파일 식별 검사를 거친 독립 reader를 유지한다. 원 supervisor의 physical closure 및 최종 receipt 성공 후에만 stdout/stderr를 각각 poll당 최대 64KiB 읽는다. 각 reader의 실제 0-byte read를 모두 관측해야 `regular_files_eof`가 된다. 오류는 최초 오류를 보존하고 원 Session/reader를 유지한다. receipt 복구도 같은 owner와 reader에서 이어가며 deadline을 갱신하지 않는다. 파이프 EOF·worker JSON·PID 추론으로 대체하지 않는다.

원 자식이 없는 Prepared/실패 상태의 `not_started`와 실제 실행 후 EOF를 구분한다. 원 NativeProcessOperation은 EOF를 확인하기 전 완료 결과나 해제 가능 관측을 반환하지 않는다. 기존 immutable receipt 스키마는 변경하지 않으며 EOF는 parent typed 관측이다. Core의 `OwnerObservation`으로 아직 전달하거나 executor를 opt-in하지 않았다.

## 입력과 검증

- 수정 전 mod/operation/tests는 mirror `runtime/verification/preimage-r202-native-eof/`에 보존했다.
- R203 입력 2459파일 inventory SHA256 `A851E9CD924AAEB334CEFA8DD8F076B65B90D8806FA07FAA3B4B950DC325240B`. 로그 `2815` fmt, `2816` snapshot 종료0. `2817` native strict 첫 실패(E0063): 기존 test Session 초기화 2곳의 새 필드 누락. 원 로그·입력을 보존했다.
- R204는 두 초기화를 수정한 snapshot까지만 수행했다. 테스트 의미를 새 비동기 EOF 계약에 맞추기 위해 R205에서 receipt 복구 후 최초 poll pending/다음 poll 완료·EOF를 명시했다. R204를 시험 성공으로 계상하지 않는다.
- R205 2459파일 inventory SHA256 `F69CDFBD933452350029E9A29EF2347D68C393E12A903D260B4FD7D03E55CA01`, SOURCE.md SHA256 `E147A3FF96396307B90BD4F0A176E08D327FFF860ED052A15AC6BA408A18BE70`.
- 로그 루트는 `runtime/verification/source375-datalab-b0-r1-logs/`. `2820` fmt/`2821` snapshot/`2822` native strict/`2823` default strict/`2824` native lib no-run/`2833` fmt check 종료0. 외부 blitz-paint deprecated 경고1은 기존 상태이며 소유 WIP를 변경하지 않았다. 고정 lib exe/PDB는 mirror `fixed-r205/native-core/`, exe SHA256 `29C315334CBC1870F8697958543D3275B9A93D841A4D90BF8C4325BCD943F043`.
- `2825` 원 일반 파일 65537바이트 stdout/별도 stderr를 poll당 청크로 읽고 마지막 실제0 read까지 pending, 반복 poll 및 독립 reader의 전체 bytes 보존을 확인했다. 파일 시험이며 자식 종료 증거와 구별한다.
- `2826` 원 NativeProcessOperation 정상 child/단일 launch/EOF/단일 결과, `2827` 원 child receipt 실패·명시 same-owner 복구 뒤 EOF/최초 오류 보존, `2828` callback panic 후 정리/동일 payload, `2829` 실제 stdin 소비 후 취소 정리 각1시험 통과. 마지막 두 root는 `runtime/verification/source-sdk-r205-native-unwind-r1/`, `source-sdk-r205-stdin-cancel-r1/`이다.
- `2830` 원 Jobs 자막 없는 렌더/발행/재사용/재열기, `2831` 선택 자막, `2832` 취소/재기동/수동 attempt2 각1 실제 회귀 통과. root `runtime/verification/source-sdk-r205-jobs-plain-r1/`, `source-sdk-r205-jobs-subtitles-r1/`, `source-sdk-r205-jobs-retry-r1/`. `2834` 사후 custody 원 실행 종료0, 2459파일 내용·식별 동일성을 확인했다.

## 남은 범위

Core owned session의 원 source/media/encoder/output 전체 소유권·publication 단일 CAS·candidate release 통합은 미완료다. 전체190/정상 bins의 최신 완료 입력은 R197이며 이번 부분 변경으로 대체하지 않는다. 제작 전체 인수0/5, GUI 시각 인수·AI 사용자 승인·기존 추론/다운로드/수동 GUI/push 보류를 유지한다.

기존 QA fixture 및 변경 검증 skill의 적용 판단을 유지한다. 실제 파일 EOF 시험과 실제 자식 종료 시험을 분리하고 컴파일/선언을 실제 기능 인수로 올리지 않는다. 정확한 skill 출처·해시는 기존 native-process-operation 정본에 연결한다.
