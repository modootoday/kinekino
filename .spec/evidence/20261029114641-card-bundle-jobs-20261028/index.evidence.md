---
id: evidence:kinekino/20261029114641
title: 카드 묶음의 실제 Jobs 실행·발행·취소 경계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 카드 묶음의 실제 Jobs 실행·발행·취소 경계

최종1989 default all-target strict0,1990 fmtcheck0,1991 custody0: current source content/identity unchanged files=2356. R109 고정 입력과 실제 시험 뒤 입력이 동일하다. R109 전체190/정상2bins는 미실행이다.

2026-10-28 R108→R109 중앙 PM. [실행/발행 계약](../../player/application/production/card-bundle-jobs.md), [실제 자산 adapter](../../player/application/production/card-bundle-file-adapter.md). 기존 media-jobs JobExecutor/SourceBlobStore/ExecutionControl/retaining host를 사용해 명시 Rust composition의 카드 묶음 실행기를 구현했다. 기본/CLI host 등록은 아직 하지 않았으며 production.export_cards의 직접 CLI 지원으로 계상하지 않는다.

## 첫 실패와 수정

R108 SOURCE.md SHA256 7E31A5202AEA9616EC2EDEF013C0068014A1260AD28BD7A70C59B739F88FD147, inventory96EB73D117715387A7C240B2A9DA49DD6F979CFEDA74F5F6FD9C204D48C185FA/2356. 1977 카드 구성 strict/1978 no-run은 성공했다. 1979 실제 native 발행 race1도 통과했다. 1980 첫 실제 Jobs 완료 시험은 NeedsInput/InvalidArgument/"Verification requires publication"으로 실패했다. 원 begin_verification을 발행 전 staging 검사 시점에 호출한 것이 원인이다.

원 Jobs worker/state/소유권 계약을 바꾸지 않고 R109에서 staging 검증→reserve_publication→원 directory handle rename→begin_verification→발행 후 FileID/hash 검증으로 수정했다. 발행 전 취소와 발행 후 취소 시험을 각각 원 gate 위치에서 실행하도록 나눴다. 원 입력/preimage-r108-card-jobs-gate/fixed-r108 exe/PDB·source-sdk-r108-card-bundle-jobs-r1/jobs-complete-28100의 실패 JobRecord/부분 staging/attempt.json/shutdown을 보존했다. 원 shutdown receipt는 error=null, completed0/cancelled0/interrupted0이며 원 worker는 종료했다. output=null/publication_started=false, 최종 출력은 발행하지 않았다. 최초 코드 preimage는 preimage-r107-card-jobs다.

## 현재 입력과 실행

R109 SOURCE.md SHA256 38A2F323EE304338F886B42C3744F3344248E615BDC90FD86615C1BFBF37F1FE, inventory80D30E0F0B519FCF3426F301201B263B72ECE2F0051494C30B4ED9FD1F94FC89/2356. mirror/runtime/verification/input-r109에 고정했다. 기존 source375/canonical 채택이 아닌 private 후보이며 root WIP·원 미디어/AI9·독립 Meari·기존 Jobs/download 소유 코드는 수정하지 않았다. 새 코드는 Rust이며 Cargo lock/toolchain/global 설정 변경·설치 없음.

로그 루트 runtime/verification/source375-datalab-b0-r1-logs:1981fmt0,1982snapshot0,1983 카드 구성 all-target strict0,1984 lib no-run0(1분10초),1985 실제 완료/재조회/재열기1통과(2.32초),1986 발행 전 취소1통과(2.14초),1987 발행 후 취소1통과(2.19초),1988 native no-overwrite race1통과. 각 선택 시험은1604개를 filtered했으며 전체 lib/전체190 인수가 아니다. CPU2/BelowNormal/heavy1·C-only/cache/tmp·locked/offline/-j1 및 각 원 handle terminal 확인 후 다음 작업 배정을 유지했다. transitive blitz-paint deprecated 경고는 별도다.

고정 root lib exe/PDB는 runtime/build-cache/native/source-sdk-integration-r1/fixed-r109/kinekino_player-a56df110f8c592d1.exe이며 SHA256 7B2D3598D6CF7FB91F9D222890B5B3A0459BD49D96859E4F6F07FB6BE3547FDA. R108 첫 실패 바이너리는 fixed-r108의 SHA F01B2B8196DE8C5254B88A2C521466EC18188077646A0B2B7200F100F466939D로 별도 보존했다.

## 실제 증거와 판정

runtime/verification/source-sdk-r109-card-bundle-jobs-r1 아래:

- jobs-complete-36668: 실제 retaining Jobs worker가 선택한 immutable project/PNG/폰트/template을 고정하고 001-cover.png·카드 provenance·raw frame PNG·caption.txt·alt-text.json·project.json·attempt.json·manifest.json을 새 디렉터리에 발행했다. manifest/output SHA와 manifest 각 파일 SHA를 읽어 대조하고 raw frame/편집 프로젝트를 재해석했다. 동일 idempotency key의 동일 요청은 created=false/같은 job으로 재조회됐고 원 프로젝트 발행은1 그대로다. host 종료 후 readonly Jobs 저장소 재열기로 같은 결과 SHA를 확인했다. shutdown error=null, completed1/cancelled0/interrupted0이다.
- jobs-cancel-5316: 모든 staging 작성/검증 뒤 원 reserve_publication 직전 gate에서 실제 jobs.cancel을 보냈다. 원 gate가 취소를 관측해 rename 없이 Cancelled/output=null로 끝났고 최종 출력은 없다. private staging의 원 job/attempt.json은 보존했다. shutdown error=null, completed0/cancelled1/interrupted0이다. 부분 파일 삭제 성공은 아니다.
- jobs-late-cancel-36372: 원 rename 뒤 begin_verification 진입 직전 gate에서 실제 취소를 보냈다. 원 Jobs가 PublicationInProgress로 거절하고 cancel_requested=false를 유지했다. 원 작업을 계속해 발행 후 검증·Completed로 끝났다. shutdown error=null, completed1/cancelled0/interrupted0이다. 늦은 요청을 취소 ACK로 위장하지 않았다.
- jobs-race-32652: 합성 control의 reserve 시점에 다른 출력 디렉터리/원 sentinel을 생성했다. 원 native directory handle의 ReplaceIfExists=false rename이 RevisionConflict로 실패하고 기존 sentinel과 원 staging/FileID recovery 정보를 보존했다. 이는 native race 주입이며 실제 Jobs 취소 인수와 구분한다.

산출물 state=review_artifact/quality_approved=false다. 합성 프로젝트/원 PNG를 사용했고 실제9·사용자 품질·수동 GUI를 시험하지 않았다. 여러 번의 successful render나 원 worker 종료를 cleanup/템플릿 승인으로 승격하지 않는다. 출력 파일/원 attempt·실패 logs 모두 private runtime에 보존했다.

## 남은 범위

기본/명시 CLI host의 등록·typed 제출·직접 명령의 지원 상태/schema 연결과 실제 CLI 여정은 다음 단계다. 전체 feature 인수는 미완료이며 부분 성공 카드 재사용/실패만 retry·발행 후 ACK 재조정·강제 crash/power-loss·disk/permission fault·부분 staging의 명시 cleanup·여러 canvas/빈 template/역할별 디자인·GUI 시각/사용자 AI 품질은 남는다. 보존된 partial을 자동 삭제/채택하지 않는다. R109 전체190/정상2bins는 미실행, 최신 완료 통합gate R104·정상2bins R107을 현재 입력으로 승계하지 않는다. 기본103/별도host10·원9 NeedsRepair/승인0/9·WARP/추론/다운로드/수동GUI/push 보류 유지.

기존에 읽은 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100), code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용했다. 원 gate의 실제 실패를 보존하고 mock control race·실제 retaining Jobs·원 shutdown receipt·CLI/전체 gate/품질을 구분했다.
