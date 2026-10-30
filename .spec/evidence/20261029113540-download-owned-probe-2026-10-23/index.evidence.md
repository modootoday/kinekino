---
id: evidence:kinekino/20261029113540
title: Owned download/probe의 원 stdin·수명·core CAS 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Owned download/probe의 원 stdin·수명·core CAS 검증

2026-10-23. 원 jobs retaining host/core가 발급한 AttemptScope와 실제 Windows child를 사용하는 한정 checkpoint **actual23**을 통과했다. root integration20 및 owned-process의 신규 input-file3을 고정 BIN에서 각각 exact1회 실행했다. synthetic child는 실제 inherited regular-file stdin의 seek/내용/EOF를 검사하지만 실제 ffprobe/디코더가 아니다. 공급자 다운로드·HLS·실제 미디어·제품 CLI 지원 인수는 아직 아니다.

중앙의 명시 슬롯 반환 확인 후 최종 source12는 root `bddeca87a129e89633833337ee86f6ce8934a6d5`에 채택됐다. 이 채택은 아래 한정 검증 범위이며, 전체 root broad check나 공급자/제품 CLI 인수를 추가로 입증하지 않는다. index 비움·push0은 중앙 채택 보고와 구분해 유지한다.

## 입력과 실행 경계

gate는 `runtime/verification/media-download-owned-r1/`, source는 `runtime/development/media-download-checkpoint-r4/`다. root `cce969547f2011e13d30badf2c5f6cc23a0a376f`, library HEAD `0b9ea031ce5c08a5bf99ed9852b668cf2bfce038`, lock `E469A7204B9DC2AC82AC729E36FD8AC41449E1C382E9167A28A00A45C45789A2`를 유지했다. 원 source12 manifest `2F2D0EC5AADC116E58AF7D52A5B11F4088F6E2C7A2E4526D5639F409D14168D6`와 원11/helper r2/module r3를 보존했다. 최종 `final-source12.json` SHA256 **A836BA7E9C7EF47820DDFA127824D537505BEA7B415D04A801A54CF9D2E63EAC**, `final-before12.json` **2F8B2C03350D93879BBF5A36E9C14B276884357C6FD973972419F9165229E01E**다. 기존5/신규7, unchanged borrowed guard2, dependency diff0이다.

중앙 배정 뒤 fresh RAM29.86GiB/VRAM6152MiB/C120.67GiB/writer0을 확인했다. 기존 canonical 제품 S/T와 runner `EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D`의 conservative2/CPU2/mask3/BelowNormal/descendant Job/j1/thread1/offline/locked를 사용했다. root 소스·index·등록·Cargo는 변경하지 않았고 library mixed/JOB launch/UI 후보를 합성하지 않았다. GUI/네트워크/모델/WSL/새 target/clean은0이다.

## 실제 결과

- canonical rustc1.99.0으로 std-only fixture를 새로 빌드했다. normal/held 두 파일은 SHA256 `0A3A8DE1CAE5ABD8D502CC92BA342566B94A3E38CD5C4D080C34B457EF38F2F3`, 209408B로 동일하다. 파일명만 held 동작을 선택한다. 이는 생산 도구 신뢰·라이선스/실제 ffprobe 설치 검증이 아니다.
- 최종 root test BIN `E066C3DD03D8DD7925EA0D78BF3ABA4BA283D4329D60DEE3F6276745B4350AE3`/3411968B: Cargo JSON fresh=false와 실제 list17의20개를 확인했다. normal18 및 root19..37에서 각각1pass, 총20이다. 원 r7 BIN `C57DDAB57D7229E22033FA274F74B226EDDB039B3AD8FBB4D5537E64470D5F7F`/3411456B도 보존했다.
- owned-process BIN `78C8BC7AF094B386015B7BD6CDD65E84D4961FB2FF8BFE19A09C724C6563DDC7`/832000B: 실제 등록5 중 input_file_tests3만38..40에서 각각1pass다. 전체5가 실행됐다고 세지 않는다.
- root20은 원 host/process4, cfg(test) orderly transfer4, Windows lease4, metadata 판정4, helper diagnostic4다. Matroska/MpegTs의 accept/mismatch 순수 판정1을 추가해 원 예상22에서23으로 늘었다. 각 실제 full name은 list17/list13과 selected-root-rest/selected-process에 보존했다. 이전 foundation16/configured19/default190을 합산하지 않았다.

정상 case는 원 sealed writer/ancestor guard를 유지한 채 private read-only stdin handle을 자식에게 전달했다. 자식 raw JSON의 `fixture_original_stdin_seek/eof=true`, 실제1024개 byte7, 길이/hash 일치와 최종1024B SHA256 `A99C07CE93703C7390589C5B007BD9A97A8B6DE29E9A920D474D4F028CE2D42C`를 확인했다. 원 journal5는 queued→running→running(publication_started=true)→verifying→completed이며 false→true 전이1/Completed1이다. 원 core가 publication CAS를 소유하고 후보는 두 번째 CAS를 예약하지 않는다. 별도 기존 target case는 probe 뒤 publish 거절 시 원 bytes와 단일 파일을 보존했다.

client-drop/run-unwind case는 실제 child ready와 원 partial에 대한 write open raw OS32를 먼저 관측하고 원 source가 닫히지 않았음을 확인했다. 호출 client를 drop해도 host가 원 session/probe/lease를 유지하고, 같은 owner의 close gate 해제 후 source와 partial을 정리했다. run unwind는 원 probe launch 뒤 주입했으며 core 밖 catch가 원 Box를 유지했다. 정상4 child는 exit0/forced=false, held2는 의도된 cancel/cleanup의 exit124/forced=true다. 원 identity(pid+creation time+exe)를 담은 close receipt6 모두 family=Empty이며, stdin_eof_requested와 별도로 원 stdout/stderr12를 family closure 뒤 bounded read해 EOF를 확인하는 경로를 통과했다. pid 재검색/교체 child/경로 재개방 cleanup으로 대체하지 않았다.

lease4는 seal/append 경계, 유지 중 write/rename/delete 거절, premature abort의 원 guard 유지/NeedsInput, premature publish의 final 부재를 실제 Windows 파일로 확인했다. input-file3은 원 offset0/read-only capability/no copied stdin.raw, 기존 bytes input 교체 거절, write-only reader 거절을 확인했다. diagnostic4는 write/flush/panic을 secondary로 보존하고 동일 completed carrier와 원 오류/원 action payload 우선순위를 검사했다. 이 constructed completed-carrier fixture는 live Pending 증명이 아니며 실제 child 수명 근거는 위 held case다. test-process incomplete boundary의 Box 보존을 cleanup 성공으로 표시하지 않는다.

## 실패와 수리 보존

원 receipt41개가 모두 terminal이며37개 exit0, fmt03 exit1 및 strict05/06·normal14의 exit101 세 개를 보존했다.

| 원 실패 | 별도 revision | 확인 |
| --- | --- | --- |
| fmt03: path-mounted façade 하위 attempt/result가 parent 디렉터리로 해석 | r5-module-path exact1의 명시 하위 path | fmt04 통과 |
| strict05: read capability probe 반환량 미소비 | r6-input-read exact1, 반환량 명시 보관; 빈 reader 허용 유지 | strict06에서 해당 진단 해소 |
| strict06: 한정 private mount의 unused public surface, 현 toolchain API/조건문/미구성 container | r7-strict exact6, 기존 foundation과 같은 public contract mount 및 이유, try_update, 동등 if 정리, 실제 metadata fixture1 | root strict07·process strict08 통과 |
| normal14: fixture jobs root 미생성, host 생성 전 NotFound | r8-fixture-root exact1 mkdir jobs; 원 factory 정책 유지 | 새 BIN/list와 attempts-r8에서 normal18 통과 |

원 stage/manifest, failures와 attempts-r7을 삭제·재사용하지 않았다. 최종 root strict15, process strict08, fmt41이 exit0이다. 한정 target strict/actual23은 전체 workspace cargo test 또는 cargo clippy --all-targets -- -D warnings의 대체가 아니며 광범위 integration check는 후속으로 남는다. 공개 legacy/testdraft Complete/receipt/first_error 선언이 이번 target에서 모든 런타임 경로를 인수했다는 뜻도 아니다.

## 보호와 한계

root tracked1313/library tracked479/index 원 SHA는 변경0이다. 기존 S4655 중 변경은 승인 기존5뿐이고 신규7은 최종 source12와 일치한다. 원 T exe530 중 변경5는 owned-process test1과 normal player/player-cli의 각 원본/alias2쌍이다. Cargo JSON build10/12가 producer이며 정상 두 alias 쌍은 동일 volume/FileID도 일치한다. 정상 GUI/CLI BIN은 부수 빌드됐지만 실행하지 않았다. 새 T exe는 root test1이다. fixed-copy5는 원 해시를 유지했다. 근거는 protection-delta/final-input-S-observation/binary-producers/binary-fileids/fixed-bin-final이다.

known cache4709 미귀속 alias2는 전후 `2D2094C73DC9FA25162BFA1B2466F7C5C5E0390ED79E637914F6E86D6BC6CCAA`로 변경0이며 기존 미귀속 상태를 유지한다. 전체 캐시 안전성이 입증됐다고 표시하지 않는다. 보호된 원 source/BIN과 .spec은 별도이며 source 채택과 중앙 자원 반환도 구분한다.

실제 provider/header/auth/runtime resolver close, 실제 ffprobe tool pin/license/protocol/미디어 품질, HLS VOD 수집·리먹스, Range/객체 동일성 재개, 정식 재리졸브/재시작 복구, batch/modes, production registry/SDK/player-cli 성공·오류·취소·저장 인수는 미완료다. 자세한 후속은 [download 계획](../../../plans/player/production/download-executor-source-admission.md)과 [요구사항](../../player/application/production/downloads.md)을 유지한다. 이 checkpoint를 제품 전체 완료나 GUI 기능 검증으로 승격하지 않는다.

이미 읽은 modootoday/software-qa/test-fixture-and-mock-build SKILL SHA `a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100`을 실제 원 host/실패 knob/격리 디렉터리/teardown에 적용했다. code-change-review-verification SKILL SHA `9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B`, verdict-table reference `9D9619A829AE24D73645EF5F7C8126C8257CAEDDAC80800632866257008AD2A9`를 예상/list/actual·원 실패·새 BIN·보호/한계 구분에 적용했다.
