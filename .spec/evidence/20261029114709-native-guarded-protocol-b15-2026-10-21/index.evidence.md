---
id: evidence:kinekino/20261029114709
title: Native guarded protocol B15 r3 검증 · 2026-10-21
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native guarded protocol B15 r3 검증 · 2026-10-21

B15 한정 Rust gate를 완료하고 heavy/shared snapshot을 PM에 명시 반환했다. 신규24/전체 고유67pass·0fail 및 default/native all-target strict/fmt0이다. 실제 native seek·제품 전체 완료가 아니다.

기준 root47e3825bc3397dfd43e7081b32e86989e8b56835/Meari98f6a5c4c72ad07a40ac0eae7f7f113926609fae다. source13=0315FFE6B35CFEAB2729A82670C29FD2ABD051B0D77B847BBDF8EBB99B8068CC와 application2=1003EF5DFBD7371FF8AF7DEAA3E4D913CB8B51A71CCCD99445152FDC7A7992FD만 합류했다. 최종 integrated15는005695C87393D933D84CD7260BCCBD3C8938A9376F12A955B7529E68E00CFDE8이다. additive patch check/apply0→CRLF-only 대조→frozen bytes/현재 mtime 동결 순서와 원 before기존8/부재7을 보존했다. harness/CLI wiring/C source는 미포함이다.

| 검사 | 실제 결과 |
| --- | --- |
| 신규 | protocol/fence23 + observation 신규1 = 고유24pass |
| default 관련 | protocol23/observation9/control7/A7 guard6/closed IPC1/projection3 = 고유49pass |
| native 관련 | protocol23/observation9/router12/binding6 = 고유50pass |
| 구성 중복 제거 | 공통32 제외, 고유67pass/0fail |
| 실제 default/native compile+protocol test |58346/101241ms0, 원 Compiling·실명 stdout 보존 |
| 양 all-target strict |38072/32471ms0, 원 Checking 보존 |
| workspace fmt --all -- --check |2871ms0 |

검증 파일은 runtime/quality-review/sdk-guarded-b15-r3/다. 실제 default libtest SHA9A68B7490E4D15257A3C4381F3BA7C51BAA849D9D5CD1EE750211DE9F0435447/94523904bytes, native SHA822A29EF06B81B36F4AA4C1AD7DB5D4BCD2D62072370A4BFD290A40C1D2F641B/155293184bytes다. 캐시 CLI 식별은 이번 CLI 실행 근거가 아니다. 전체 suite/A7 원48과 중복 합산하지 않으며 전체 suite 반복0, source 수리/실패/retry/0selected0이다.

고정 runner EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D의 conservative2/affinity3/BelowNormal/descendant Job에서 절대 manifest/target·kinekino-player/offline/locked/j1/thread1을 사용했다. 원 handle 종료 회수→terminal receipt 확인→다음 실행의 순서를 유지했다.

최종 root949/snapshot1306/UI7/LocalHost6/A7 비중첩5/Meari405/source15/integrated15/default binary delta0, receipt13 terminal/exit0·기록26PID 부재/owned[]·root index empty다. 원 snapshot을 최종 B15 입력으로 보존하고 추가 Cargo0으로 반환했다.

기존100ms tick으로 legacy/초기 구독/guarded pinned write와 pending deadline을 처리하는 generic seam 회귀다. deadline 순간 중단·기록 bytes 취소·실제 named pipe/OS/mpv 인수는 아니다. production issuer와 C 실제 owner 연결이 없으며 같은 호출 Observe→core→원 guard Submit 공백은 그대로다. authorized=false/guardNone·submit/result Unsupported를 유지한다. GUI/미디어/모델/다운로드/VM/DPI 실행0이다. 상세 제한과 root 적용 방법은 같은 디렉터리 handoff.md와 [단일 설계 계약](../../../plans/player/native-guarded-observation-contract.md)을 따른다.
