---
id: evidence:kinekino/20261029114108
title: 기본 Native launch 순수17 검증 — 2026-10-24
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 기본 Native launch 순수17 검증 — 2026-10-24

## 관측 범위

root `76287c43e59b98134e8dad127e6d83790dd2e504`, Meari `f85a68df7a6a8fee239d9655895ee15d5eaa5510`, lock `8F2C1D56C8A2C22D36D9E57A285D727C7EE9C7D2D1160902B66CC6630B042C67`. PM 단독 heavy 배정으로 기존 shared S/T와 resource-runner `EEA9DEE6`를 사용했다. 매 실행 RAM16GiB/GPU3072MiB/C12GiB/writer0 입장, CPU2/mask3/BelowNormal/descendant Job·offline/locked/j1/thread1을 유지했다.

UI slot `7051F6B5`의 S6043/T588/carry135 `785786BB`와 fresh 비교 차이0. canonical A14 exact14의 원 공유 preimage를 별도 보존·동기화해 baseline6045를 고정했다. r2 private-parent before를 공유 복원 기준으로 사용하지 않았다. 기존1/new6 overlay와 원 shared 복원 기준은 `runtime/verification/default-native-window-r2/03-overlay-preimage7.json`이다.

## source와 최초 오류

원 r2 source7 `7C3D7B1C`와 expected17 `FC29CAB6`를 보존했다. 원 오류마다 후속 gate를 중지하고 PM 승인 범위 안에서 아래 private 후계를 분리했다.

| 입력 | 원 gate 결과 | 후계 |
| --- | --- | --- |
| r2 | 04 fmt exit1: child module이 `native_product_launch/control.rs`를 찾음 | r3 `0AF44C20`: default_launch.rs exact1에 child5 명시 path |
| r3 | fmt/default strict0, 09 native strict101 E0369: Windows flags BitOr 미구현 | r4 `DE1E907F`: exact1에서 동일 `.0` bit값을 GETFINALPATHNAMEBYHANDLE_FLAGS로 구성 |
| r4 | fmt/default strict0, 13 native strict101: collapsible_if·large_enum_variant | r5 `4F69CEEA`: control let-chain, Observing의 원 Supervisor 단일 Box 이동, 생성 caller exact3 |

최종 r5 effective7 전체 SHA `4F69CEEAED179B1FBFCDFD92BD92F9F429D8BCF47E8604F60BDB8D405F51B6E1`, delta `F3E5CA2B7B9575FFC0577C642666E871CBAA27EDF3C7142E7BB565DDBB28D336`. 다른4와 expected17 불변. clone/leak/새 Supervisor/조기 drop/원 Pin guard·단언·deadline·cleanup 정책 변경0. source/raw/before/delta/HANDOFF는 각 revision 디렉터리에 보존했다.

## 실제 결과

r5의 15 fmt·16 default all-target strict·17 native-observer-test-support all-target strict 모두 terminal0. 18은 `cargo test --offline --locked -j1 -p kinekino-player --test native_product_launch --features native-proposal-window --no-run --message-format=json`이며 terminal0/209973ms다. 전체 suite 반복0이다.

원 compiler-artifact에서 `native_product_launch` producer를 선택했고 독립 사본의 SHA·bytes·FileID·feature/profile을 고정했다. fixed SHA `C60AD456539EEED79B8CB852C26A4F519B07E037090EB1D2F0698AEEFC67F763`. 실제 list25/ignored8이며 동결 expected17 모두 ordinary로 정확1회 등록됐다. 같은 fixed BIN에서 17개를 각 `--exact --test-threads=1`로 실행해 각1pass/0failed/0ignored·원 terminal0을 확인했다. 0-test 성공은 세지 않았다. default 구성 등록0은 이번 별도 producer/list 실행 근거가 없다.

오류 객체 보유·bounded log/overflow·입력/권한·동일 SHA의 identity 비교·NoForce 계약 fake counter가 순수 시험 범위다. 실제 Windows 파일/부모 교체 거절·ancestor reparse·pipe ACL·원 host 생존·정상 N/C·렌더 frame·전체 CLI 기능 인수는0이다. [fileguard 후속 계획](../../../plans/player/presentation/default-native-file-guard-evidence.md)을 유지한다.

## 복원·감사

기존1 bytes/mtime와 신규6 부재를 복원했다. Cargo가 함께 생산한 player/CLI exe·PDB4는 독립 사본을 먼저 보존하고 정확한 alias↔deps 생산자4의 동일 bytes/FileID를 기록했다. 원 백업을 새 파일로 검증한 뒤 exact alias unlink/replace로 복원해 직접 덮어쓰기0, 생산자4 bytes/mtime/FileID 불변을 확인했다. alias4의 원 bytes/mtime는 복원됐고 FileID는4 변경됐다. 과거 alias/known2 예외를 유지하며 all_cache_protected로 표시하지 않는다.

최종 감사: root1383/library497/index·manifest·lock9 원 bytes/mtime 변화0, A14 canonical 신규2는 이전 root 목록 밖이므로 fmt 뒤 supplemental로 관측했고 최종 일치0차이다. 이를 최초 전체 root1385 preimage로 확대하지 않는다. 원 fixed135와 신규 fixed1 bytes/mtime/FileID 일치, carry136 SHA `E9615564A168CD7EFD6E9A97F05AC5E874A1B28AC8EA9CBF71F309CBF9D6D1EA`.

S는 승인 sync baseline6045→최종6045로 비입력 변화·신규 runtime0. T588→589이며 변경2(player/CLI deps producer)·신규1(native_product_launch producer)을 원 Cargo JSON·동일 SHA/FileID 연결로 모두 귀속했다. PDB2도 exact alias↔producer4 기록으로 보존했다. 메타데이터 귀속 최초 시도는 Cargo의 forward-slash 경로와 파일시스템 backslash 경로의 문자열 차이를 거절했다. GetFullPath로 표현을 정규화하고 동일 SHA 조건을 유지했으며 원 메모를 보존했다.

receipt30 모두 terminal이며 exit0 27개와 보존된 최초 fmt/type/Clippy 오류3개를 구분한다. 테스트 실패0, 실제 고유17pass다. 기록 PID60 전부 부재, writer0·pending owner/session0. 명시 반환 receipt는 `runtime/verification/default-native-window-r2/slot-return.json`이다. 이 반환은 순수17 checkpoint이며 실제 OS 인수가 아니다.

## 스킬과 증거

modoo.today `modootoday/software-qa/code-change-review-verification` 본문 SHA `9f73865f96ec0cefcf3a4023b294400258473640df3b1d2add03dcca8192d64b`, 필요 참조 `references/verdict-table.md` SHA `9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9`를 실제 읽었다. 이전 review 주장을 원 오류·call path·동결 순수 시험으로 확인하되 OS 재현과 구분했다. 기계적 Rust 수정은 rust-pro/clean-code/lint-and-validate의 기존 세션 적용을 유지했다.

원 증거: `runtime/verification/default-native-window-r2/`의 번호별 stdout/stderr/resource/receipt, `actual17.json`, `19-fixed-bin.json`, `22-registered17.json`, 각 source revision의 raw/before/delta/manifest. canonical stage/commit/push는0이며 PM 소유다.
