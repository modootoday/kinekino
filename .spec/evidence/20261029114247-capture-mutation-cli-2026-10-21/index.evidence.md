---
id: evidence:kinekino/20261029114247
title: Capture origin·원자 mutation·실제 CLI 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Capture origin·원자 mutation·실제 CLI 검증

2026-10-21 root `3ebb4498655547aaa4f065fe0c34880e03e75c3a`, Meari `4331ff3ea5cad71a7159bdb9e3f86fbfccb16220` 기준 후보를 실행했다. 원 증거는 `runtime/verification/capture-mutation-cli-r1/`이다. SDK는 root 제품 소스·index를 변경하지 않았고, 반환 뒤 PM이 최종35를 `541b4c83abe0da1596683db9dca03196312ccfdc`에 채택했다. 후속 library 변경에 이 BIN의 실증을 승계하지 않는다. [설계 결정](../../../adrs/player/adr-capture-origin-and-project-v2.md)과 [공통 계약](../../player/application/api/capture-mutation.md)을 구별한다.

## 입력과 최소 수리

producer source20 `16D02EA6…`, guard source10 `965DDBE7…`, API source12 `B4155D23…`를 명시 override4/3 순서로 합친 원 union35는 `B4E16D551238FB7D2E7E614F3C871767D67B59EDD3C17FED885D57BB3677818D`다. 모든 before/원 raw42를 대조했다. 기존 S source mirror와 native T cache만 사용했으며 lock `721586F9C83ADA7CF504E5A5AE0CB168F66B017C3956CD84387D794E4EAE2528`을 offline/locked로 유지했다. 기존 windows dependency에 FileSystem feature 하나를 추가했고 dependency/version/lock 재생성·다운로드0이다.

원 all-target strict는 explicit-argument helper의 inherited private consumer 미사용 경고로 exit101/15736ms였다. r1은 기존 run_process도 같은 helper를 실제 호출하도록 수리했다. r1 strict는 새 독립 시험에 기존 shared self-test의 부모 hash hook이 없어 exit101/21107ms였다. 그 의존성을 제거한 r2 시도는 기존 다른 caller hook을 미사용으로 만들어 exit101/7435ms였으며 채택하지 않았다. r1 helper를 복원하고 새 독립 시험에 기존 digest hook을 추가한 r3만 채택했다. 기존 Rust digest를 위임하며 suppression·skip·authority/CAS 완화0이다. 세 원 실패와 raw/receipt를 보존했다.

최종 입력35(기존18/신규17)는 `final-source35.json` SHA256 `90714B052FC7ACD38518B9587E7320F9515599DC87462E62D255ADA8D29B9408`와 raw `final-source35/`, `before35.json`에 고정했다. 원 후보는 불변이며 최종 수리는 시험2파일뿐이다. 상세는 gate의 REPAIRS.md다.

## 실제 검사·시험과 바이너리

최종 cargo fmt --all -- --check 4200ms/0, default workspace all-target clippy -D warnings 7944ms/0, 최소 root lib/capture integration cargo test --no-run 173691ms/0을 확인했다. 검사와 빌드는 canonical resource-runner conservative2/mask3/CPU2/BelowNormal/descendant Job/offline/locked/-j1로 실행했다.

| 고정 artifact | SHA256 | bytes | 실제 범위 |
|---|---|---:|---|
| root libtest | `F289DE9FE7B03A3DC06BE1046210452AAC94E59F0912AA02C9B37C3DA0FA9293` | 99191296 | 실제 등록666 중 새22 exact pass |
| capture integration test | `178CDBDCBEA51644BBB6CA04E48D595262FB36185E241697A30255EBB899B300` | 4285952 | 등록9 중 ignored actual1 명시 pass, 수리 직접 회귀2 pass |
| player-cli.exe | `4919B5B2200544219C3D8AC28C7FEE3219F9DB9315CFC4A4B5D65AC990C9B6C3` | 30661120 | 고정 원본·사본에서 실제 child12 |
| player.exe | `2F39A12EF1CE359A6541FD9B4830A59A2AADA2C607578A454093C685D15B4CD3` | 80636928 | integration 빌드 출력, 실행0 |

각 BIN 원본/사본의 종료 후 SHA가 일치했다. root actual22는 producer·v1/v2 호환11, domain/guard9, API2다. exact/thread1로 각1pass/0fail/0ignored이며 나머지665개는 매번 제외했다. actual CLI1은 9305ms/0, 1pass/0fail/0ignored/8filtered다. 수리 회귀2는 inherited digest hook·invalid-budget-before-spawn 시험이며 새 guard22와 합산하지 않는다. 전체 cargo test suite 통과 주장은 하지 않는다.

root22는 원 root handle FileIdInfo, 같은-volume rename/복사 metadata 거절, 부분 metadata 보존, 실제 writer 경합, saved origin 재사용, v1 frozen bytes/canonical SHA·v2 mixed·old strict snapshot reader, 같은 writer의 mutation/명시 continuation/exact accept retry·legacy 거절을 일반 파일로 확인했다. 실제 다른-volume 복구와 미지원 filesystem, 과거 제품 binary 실행은 검증하지 않았다.

## 실제 CLI 관측

시험은 CLI로 제안을 생성하고 live capture를 저장했다. 새 프로젝트는 schema2/origin Some이었다. 실제 CLI는 같은 공통 application 유스케이스를 호출했다.

| 호출 | 관측 |
|---|---|
| 03 최초 edit → 04 continuation edit | revision2 → 3 |
| 05 명시 accept | revision4, reused=false |
| 06 원 pre-accept payload exact retry | 같은 state/continuation revision4, reused=true |
| 07 stale edit / 08 forged expected / 09 wrong project hash / 10 foreign backend | 모두 revision_conflict·exit5 |
| 11 origin 없는 과거 capture | needs_input·exit3 |
| 12 원 project revision 재개방 | 최초 응답과 전체 동일, 자동 project save0 |

원 child12 전부 원 creation identity/handle 기반 receipt의 family=Empty, forced=false, stdin EOF requested이며 stdout/stderr가 완전 보존됐다. stderr bytes0, stdout JSON 한 줄이다. 재시도는 기존 receipt만 먼저 반환하지 않고 원 pre-state 전체 해시와 결정적 post-state를 동일 writer 아래 대조했다. 입력은 고정 synthetic cue/잘못된 generated JSON이며 실제 모델·미디어 품질 또는 자동 적용의 근거가 아니다.

## 소유 회수·보호와 남은 범위

원 runner34 receipt 모두 terminal이며 성공31/보존 strict101 3이다. 원 exec handles가 반환됐고 소유 live process0이다. PID68 부재는 보조 관측이다. 원 CLI child12 receipt/EOF·raw, 일반 파일 fixture14 사본을 남겼다. root1175+extra11/library454+docs2/양 index/부모42/S 비입력1168/S library456 및 고정 BIN 사본의 보호 차이는0이다. 정상 CLI5DB→4919와 player3195→2F39 두 target 교체는 의도한 새 build artifact이며 원 5DB 사본·과거 실행 증거를 보존했다. 과거 실증을 새 BIN에 승계하지 않는다. 이전 native CLI unit7A 원본/사본도 일치했다.

heavy/shared 슬롯을 PM에 명시 반환한 뒤 이 정본을 기록했다. UI3 adapter의 type/runtime/원 host dispatch·close·시각 검증과 제품 readonly 해제는 별도 후속이다. 같은-process Windows writer 경합 외의 동시 다중 CLI 경쟁, 전체 사용자 흐름, 실제 미디어/worker/model/GUI/VM/DPI는 이 gate에서 실행하지 않았다. 프로젝트 latest와 제안 간 cross-root transaction도 보장하지 않는다. 전체 제품 goal은 active다.
