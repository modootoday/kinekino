---
id: evidence:kinekino/20261029114730
title: 숏폼 출력 검증·발행 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 숏폼 출력 검증·발행 검증

R157 통합 gate 완료: 2440 Product145, 2441 Core18, 2442 Native26, 2443 OS1의 group-return에서 execution_result와 producer_integrity가 모두 Ok(())이며 동일 R157 SOURCE pin이다. Meari는 기존 고정 producer 산출물의 실제 재실행이며 이번 새 컴파일로 계상하지 않는다. 2444 정상 player/player-cli bins, 2445 사후 custody(2424파일 내용·identity 동일), 2446 fmt check 모두 종료0. 정상 exe/PDB는 fixed-r157/native-bins에 보존했다. CLI SHA B7E31C85D9E266BDC4D27AB7195E87F916D712A6C0853AABAC2514A8797BF797, GUI SHA 0B361651FFBC42F2FF63091D5B6EFD641BDF118D8C86C3ADAA7DA387E1F59C5D. 외부 blitz-paint deprecated 경고1은 기존 소유권 범위로 유지한다. 전체 제품/GUI/AI 품질 완료를 뜻하지 않는다.

R157 실제 결과: 2437 no-run은 원 영수증 terminal=true/exit_code=0으로 확인했다. 고정 lib SHA는 322D51D6984E9E5CCA87627112694EE28195D37D68BBDC4C5F60CA3AD8D06379이며 exe/PDB를 fixed-r157/native-core에 보존했다. 2438 movie/metadata 2시험과 2439 실제 후보·typed 출력 검증·불변 검토 발행 1시험이 종료0이다. 실제 1080×1920/30frames, 영상·AAC·컨테이너 각 1000000us, movie 1000ticks/1000timescale, 전체 decode 무결성을 확인했다. 후보 SHA 4b198a94795d9e85814757994dfb7859df1d24e4757061edce6588e0039bdc2d, first manifest SHA 4b70451ba3aa4a14aaec57838a7a5b491f9391098cf2a97f0433b6b1798f6faf. 근거는 runtime/verification/source-sdk-r157-output-r1/actual의 typed-output-verification.json, publication-receipt.json, verification.json이다. 변조/다른 pin/덮어쓰기 거절·고정 pin 재열기·취소/늦은 취소·commit 후 timeout 조정을 포함한다. publication_control_is_fixture=true, jobs_or_cli_verified=false, color_verified=false, quality_approved=false를 유지한다. 2440 지정 Product145 회귀는 진행 중이며 전체190/정상 bins/custody는 아직 R157 완료로 계상하지 않는다. 아래 draft/진행 기록은 당시 관측이며 최신 결과는 이 문단을 기준으로 한다.

R157 입력 고정:2435 snapshot 종료0,2424파일/inventory33304D8CCBC7D1967FCD695F171A0C376066C6910C07F8F428B545F45274FAC7, SOURCE.md233267750332B5F50F2D46E0BC2EB8BA40388209C107B71EF660F4D6C16391BD.2436 기본strict 종료0,2437 no-run 실행 중이며 새 실제시험 전이다. underlying root/tool guard의 일반 local-Disk 제한은 바꾸지 않았다.

R156/r1 첫 실제 실패:2431 no-run/2432 movie·metadata 2시험은 종료0. 고정lib SHA249680B177F1B37EDC7B052608794956727155AC7CF043B6E9FB4D1455A75122와exe/PDB를fixed-r156-r1에보존했다.2433실제시험은verifier.open에서PermissionDenied로실패했다. strict ordinary-Disk root API에이미canonical인verbatim tool 경로를재입력한fixture연결과tmp/output root의동일재입력문제를확인했다. 원입력/로그/후보를보존하고preimage-r156r1-path-context에기존코드를보존했다. R157은guard정책을넓히지않고configured 일반경로를재open용으로유지하며canonical은내부I/O에만쓴다.2434fmt완료/2435snapshot진행중이고재검증전이다. 첫실패뒤발행성공으로계상하지않는다.

2026-10-28 중앙 PM. [계약](../../player/application/production/clip-output-verification-publication.md)의 원 MP4/metadata/전체decode 검증과 새 검토 산출물 발행을 구현한다. 원소유권/WIP/private overlay·보류를 유지하며 새 worker나 취소 arbitration을 만들지 않는다. 기존 mod/tests는 preimage-r154-output-verifier에 보존했다.

R155 draft는2423/2424fmt·2425snapshot까지만 실행했고 실행시험은 하지 않았다. R156 draft는 commit 뒤 cancel과 deadline을 구분하고2426fmt/2427snapshot까지 보존했다. 실행 전 검토에서 fixture의 '만료 시 now 반환'을 고정된 과거 deadline으로 고쳤으며 원 tests는 preimage-r156-fixture-clock에 보존했다. 현재 입력은 input-r156-r1이며2428fmt check/2429snapshot/2430기본all-targets strict가 종료0이다. 2431기본lib/Jobs/CLI no-run 실행 중이다.

현재2424파일/inventory92744D38546FF258BC35C84F3D17FFB924A8E665F16AFF3D638FEEDDE6A67B5B, SOURCE.md CA75258C0BE0AE1036E2B798446F64A22297A370345DB1C306A1069413B9E17C. 원AGENTS SHA40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C 확인. 새 movie/metadata2시험·확장 실제후보/검증/발행시험은 아직 미실행이며 최신 전체 gate는R154다.

예정 실제 범위는 원Sintel 후보 생성 후 byte변조 거절·typed검증·spool쓰기차단, manifest commit·전체파일해시/lineage·재열기·덮어쓰기/다른pin/파일변조 거절·원복 재검증, reserve전취소의partial보존·예약후늦은취소의완료보존·commit후deadline실패의manifest pin조정이다. 발행 control은 fixture이며 원Jobs/CLI 인수로 계상하지 않는다. 품질/색 정확도 승인·자막/구간crop·전체입력/디스크/재시작/ACK는 남는다. 기존 QA 스킬 출처·해시는 공통계약에 기록했다.
