---
id: evidence:kinekino/20261029114513
title: Review package 내부 binary evidence 읽기
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Review package 내부 binary evidence 읽기

2026-10-26 중앙 PM. 실제 timeline resolver 연결 조사에서 ReviewSession은 이미 package/assets SHA와 모든 파일 guard를 보유하지만 읽기 API는64KiB text뿐임을 확인했다. raw audio/frame JSON/PCM을기존custody로소비하기위해내부Rust read_evidence_asset(asset_id,sha,max_bytes)계약을추가했다.별도media registry·경로입력·CLI명령은추가하지않는다.

기존provider의기본구현은Unsupported다.실제ReviewFiles session은선언된asset ID→SHA일치→요청한도(1..64MiB)→기존objects/SHA일반파일→실byte수/SHA재확인순서로읽는다.새객체/판정파일을생성하지않고session의기존guard를유지한다.외부source JSON의경로를해석하거나실행하지않는다.공개reviews.read_text_asset의64KiB/media type/UTF-8정책은그대로다.

193은기존실filesystem custody시험을확장해실행중이다.합성package의NUL/비UTF8 binary일치,원source request원문읽기,존재하는SHA라도foreign asset ID거절,다른선언SHA와의혼합거절,한도초과/0/>64MiB거절,session보유중writer거절을검사한다.실제원미디어/사용자품질시험이아니다.완료전통과계상0.

## 확정 결과

193 exact실파일시험1pass.증거identity오류문구를decision이아닌evidence로분리한최종후계194에서review_source영향25pass/1internal helper ignored(2.38초),별도process복구·capacity행렬포함.새binary membership/UTF8독립·bounds/guard검사통과.원source JSON의Z경로는열지않고원문bytes로만읽었다.

fmt --all -- --check 종료0,195 all-targets strict 종료0/19437ms.소스 SHA는application/model_review/mod.rs549F73F73D4A95F6BE0421B5568661AB887AF5C20E55ADAA663F13B7E4C0E541,infrastructure/model_review/session.rs9CC9C6B0BE21FDCB19533CB9671CCC666E1A009B35870FCC5CF58DE0B1D33758,review_source/case_tests.rs45120A5DDB5DEA62763693EA4B3A2B60C1D0F66F3F7CA66C6ED43F73DBC84AC8.194/strict동안코드변경없음.

실제resolver를위한기존custody내부읽기통로는완료했다.원관측을이통로로읽어source/tool/process·case에결합하는resolver본체는다음개발이며완료로표시하지않는다.외부파일경로reader로기존package정책을우회하지않는다.

이계약은resolver의입력통로다.실제TimelineEvidenceResolver·관측별source/tool/process결합·Verified저장전이·SDK재생clock은계속미완료다.기존396통합190/정상bins를이후계의통과로계상하지않는다.기존WIP·소유권·원자료·heavy1/C캐시·추론/다운로드/수동GUI/WARP/push보류유지.

기존modoo.today test-fixture-and-mock-build/code-change-review-verification의실파일/합성·membership/증거주장구분을적용했다.[원case custody](review-source-case-custody-20261026.md),[PCM adapter](timeline-pcm-custody-20261026.md),[admission](timeline-evidence-admission-20261026.md)에연결한다.
