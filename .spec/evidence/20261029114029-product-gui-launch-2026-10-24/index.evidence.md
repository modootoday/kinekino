---
id: evidence:kinekino/20261029114029
title: 기본 제품 launcher typed plan 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 기본 제품 launcher typed plan 검증

기준 root `a7a9ca4cc3a72ffc76a9a9f55422a1428cb356d4`, library `b532fe0eed06d8526b755b11bc36416070244ef7`, Cargo.lock `8F2C1D56C8A2C22D36D9E57A285D727C7EE9C7D2D1160902B66CC6630B042C67`. private effective7 `542A097136DE0857A912CA47DEA68A73A2C88C7582DA35C5D7DBB54F30AE9E9D`를 검증했고 제품 canonical 채택은 PM 소유로 남긴다. [계약·변경 경로](../../../plans/player/presentation/product-gui-launch-contract.md), 실제 명령/원문/보호 정본은 root의 `runtime/verification/product-gui-launch-r1/`다.

## 결과와 범위

Windows featureful 일반 실행의 mode를 NativeViewer로, 기능 없는 일반 실행을 기존 Player로 결정하는 DesktopLaunchPlan을 작성했다. 명시 native Unsupported/충돌, browser/demo/proposal 의미와 validation placement의 기존 Player 경로를 보존했다. 프로젝트 선택을 한 번 해석해 typed 값으로 assembly에 전달한다. 기존 configured SDK·단일 LocalHost·owner/cleanup과 명시 root 정책을 재사용한다.

실제 exact 실행47회가 각각1pass이며 고유 이름26이다. 신규 integration은 default6/native10=16회·고유10, 기존 launch/native_viewer/project/native_proposal 옵션은 양구성15씩30회·고유15, 실제 원 LocalHost가 필요한 기존 caller scoped1이다. 기본 구성의 기존 revision inline early-return 통과는 revision 본문 검증과 구분하고 native 본문/신규 pinned 시험을 근거로 삼았다. 실제 목록/명단/각 test case·receipt는 `11-lists-summary.json`, `expected47.json`, `12-registered47.json`, `actual47.json`이다. 전체 suite를 실행한 것은 아니다.

| 검사 | exit | ms |
| --- | --- | --- |
| S overlay cargo fmt --all -- --check | 0 | 5789 |
| default all-target strict Clippy | 0 | 56587 |
| native all-target strict Clippy | 0 | 88909 |
| default lib+integration no-run JSON | 0 | 150945 |
| native lib+integration no-run JSON | 0 | 366407 |

첫03 fmt는 작업 디렉터리를 root로 잘못 지정한 실행으로 원 receipt를 보존하고 후보 검증에서 제외했다. 올바른 S cwd의04 fmt를 위 결과로 사용한다. 코드 수리·빌드 실패·시험 실패·자동 재시도는 없다. 61개 원 receipt는 모두 terminal/exit0이며 이 수에는 첫03 fmt와 목록8도 포함된다. 시험47과 receipt61을 혼동하지 않는다.

## 고정 BIN과 실제 등록

두 Cargo JSON에서 root target/profile.test/executable을 식별해 별도 고정 BIN4를 만들었다. 모두 fresh=false이며 실제 producer와 copy SHA/FileID는 `08-default-fixed2.json`, `10-native-fixed2.json`에 있다.

| 구성·target | bytes | SHA256 |
| --- | --- | --- |
| default lib | 101566976 | 560ADE19639D9D6B8CB053D5C1EF5D9AB6B53A8ADA58B1265D1EBD47FA89557D |
| default integration | 1370624 | 7724C33182BED450A2109A0EC82D9E7A871B824875235BC0FBC8E793941A0387 |
| native lib | 176894976 | 7714E8A20D8EC054FAC148920D3CB654ABA0E57A0003B60FED43847F19FAF98E |
| native integration | 9235968 | 399B7EE44F39EB87E76B9EA83AE277F614D265F04A805F3458FDD740AE453744 |

실제 default lib747/ignored5, integration6/ignored0; native lib1218/ignored93, integration10/ignored0이다. feature별 목록과 native-only 부재를 확인했으며 정확 시험당1pass를 확인했다. 같은 이름의 두 feature 실행을 고유 시험으로 중복 집계하지 않았다.

## 복원·소유 감사와 예외

UI 반환 DB848D00/S5907/T586/carry125 0F04D825에서 시작해 canonical UI15만 S에 동기화했다. 새 admission baseline5912 뒤 launcher7을 overlay했다. 검증 뒤 기존6의 bytes/mtime과 신규1 부재를 baseline으로 복원했다. 최종 S5944=baseline5912+원 scoped PID72324가 만든 runtime evidence32이며 모두 receipt15에 귀속해 보존했다. writer.lock 존재를 active writer로 해석하지 않는다.

root1382/library493/index·manifest·lock6의 bytes/mtime 차이0, 기존 fixed125 bytes/mtime/FileID 차이0이다. 신규 fixed4를 합친 carry129 SHA256 `A5238F92237DE48233C1A4A3DF4E99D5230D457F3B2993B2217D50770B7A56AB`를 인계한다. 최종 T588은 기존586에서 changed4/new2이며 이번 producer로 귀속했다.

감사 중 담당자의 alias 덮어쓰기 복원이 하드링크 producer4를 일시 변경한 오류가 있었다. `RESTORE-INCIDENT.md`와 수리 전후 snapshot을 보존했다. 실제 Cargo09가 명시한 산출물과 원복 전 보존된 사본·동일 FileID로 확인한 exact producer4만 복구하고 alias4를 분리했다. alias6 bytes/mtime은 원본과 같고 alias4 FileID 변경을 명시한다. fixed129는 수리 대상 identity와 교집합0, known2 bytes/mtime/FileID는 불변이다. 과거 known2 미귀속 예외와 이번 alias FileID 예외 때문에 `all_cache_protected=false`다. 모든 cache/alias FileID 불변으로 확대하지 않는다.

최종 own worker/runner/시험 process0이다. 기록 PID 중 재사용된 cmd/node4는 원 terminal receipt 이후의 생성 시각으로 구분해 `recorded-pid-reuse.json`에 남겼고 종료시키지 않았다. 원 scoped case는 run_retaining_case의 원 반환으로 종료했으며 시험 종료 후 추가 owner/cleanup 대기0이다. 실제 GUI·Native 창·CLI 프로세스·AI 실행0, stage/commit/push0이다.

## 미완료와 적용 스킬

SDK/CLI 전체 선행 범위, Native media 기능 동등성, 영속 기본 root 설정, 실제 기본 창 도달성·입력/포커스/접근성·시각 인수는 계속 미완료다. 이번 typed mode·headless 연결 성공을 제품 전체 완료로 표시하지 않는다. NoMedia/Jobs/CLI 정책은 변경하지 않았다. CLI 실행으로 GUI 미지원 기능을 우회하지 않았으며 GUI로 기능 검증을 대체하지도 않았다.

modoo.today `modootoday/software-qa/test-fixture-and-mock-build` 본문 SHA256 a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100, `code-change-review-verification` 본문 SHA256 9f73865f96ec0cefcf3a4023b294400258473640df3b1d2add03dcca8192d64b 및 필수 `references/verdict-table.md` SHA256 9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9를 읽고 실제 adapter/명시 fixture 격리·해제, caller별 검증·미입증 경계, 원 실패와 복구 예외 보존에 적용했다. Rust/architecture/clean-code/파일별 lint 지침 적용 판단은 원 HANDOFF에 있다. 다른 언어 개발 코드·새 검증 도구는 도입하지 않았다.

PM 인수: 원47 stdout/terminal 및4BINsha 대조 후 정확7을 67c6215a1a831967090cd5de6d55077f91995bb9로 채택. 사고예외 유지. 실제기본창/시각 인수는 후속.
