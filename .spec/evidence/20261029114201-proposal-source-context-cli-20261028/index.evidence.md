---
id: evidence:kinekino/20261029114201
title: 실제9 source 컨텍스트 SDK/CLI 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제9 source 컨텍스트 SDK/CLI 검증

2026-10-28 중앙 PM. R67은 source375 후속 private overlay이며 canonical 채택이 아니다. 기존 소유권·WIP·원 실패·원본 증거·보류를 유지한다. [계약](../../contracts/production/proposal-source-context.md)에 따라 `proposals.inspect_source_context`와 typed `inspect_proposal_source_context`를 연결했다.

## 고정 입력과 생산자

- mirror: `runtime/development/source-sdk-integration-r1`, snapshot `runtime/verification/input-r67`.
- SOURCE.md SHA256 `E70A4917333E00B103B780EBEE9A9D1489F33EC030EDB080E01B3AB610B447F6`; inventory `95E6F6542C4B4AA2FF198AF089D5EC3DDE5B9B68B67ED9A87F156F576D0B7324`, 2313파일.
- Cargo 생산 근거: root `runtime/verification/source375-datalab-b0-r1-logs/1544-source-sdk-r67-context-no-run.stdout`, terminal0/126906ms. Native feature의 세 integration target을 생성했다.
- 정상 CLI SHA256 `92C75A064B19F4F6B453285AC7CC74854EE12FF541978E0EB8756C62326D66D8`, 37226496bytes. 시험·CLI 고정 사본은 `runtime/build-cache/native/source-sdk-integration-r1/fixed-r67`, 목록은 `runtime/verification/source-sdk-r67-fixed-artifacts.json`이다.
- preimage는 mirror의 `runtime/verification/preimage-r65-context-wire`다. 첫 R66 lint 실패1541(field_reassign_with_default, exit101)와 당시 시험 소스를 보존하고 초기화 구문만 수정했다. 제품 정책 완화는 없다.

## 실제 관측

아래 번호의 stdout/stderr/receipt는 위 root 로그 디렉터리에서 보존한다. 실제9 원자료는 고정 historical verifier로 읽고 시험용 사본만 수정했다. 새 추론·다운로드·수동 GUI는 실행하지 않았다.

|로그|관측|결과|
|---|---|---|
|1543 native-strict|Native 모든 target strict|exit0/27830ms|
|1546 context-cli|새 실제9 CLI/typed 조회: 24 CLI 요청, typed 양성9·잘못된 timeout 거절|선택1 통과/20593ms|
|1547 actual9|기존 실제9 import·수정·판정·저장·재열기·거절 103 CLI 요청|선택1 통과/41625ms|
|1548 bare-discovery|기본 구성 발견98·schema/status 대조|선택1 통과/32632ms|
|1549 configured-discovery|project/proposal/job 저장소 구성·schema 대조·저장소 무변경|선택1 통과/32275ms|
|1550 model-discovery|catalog 조회·발견98·선언 보존·admission 없음|선택1 통과/37619ms|
|1551 default-strict|기본 구성 모든 target strict|exit0/27494ms|
|1552 fmt|cargo fmt --check|exit0/2958ms|
|1553 final-custody|고정 입력 content/identity 재검증|exit0/41542ms·2313동일|

각 C2 시험은 원 process exit0·family Empty·nonforced·stdout/stderr EOF·원 소유권 complete를 확인했다. C2 manifest 자체를 Cargo 생산 증거로 대체하지 않고1544 생산 로그와 입력 snapshot을 함께 보존한다. 실제 시험 출력은 root `runtime/verification/actual9-context-cli-r67-r1`와 `actual9-current-cli-r67-r1`, C2 outer는 `source-sdk-r67-*-r1/outer`다.

## 입증 범위와 남은 작업

새 조회는 동일 state/context를 반환하고 verifier/source/candidate/reference를 확인한다. 카드/하이라이트6개×cue7의 선언 시간과 교정3개의 빈 시간 목록을 구분한다. canonical cue 시간은None이며 timing_authority=declared_request_only, media_binding=not_verified, quality_approved=false다. verifier 없음·추가 payload·잘못된 context·증거 누락은 거절하고 조회 전후 proposal/project와 원 입력 inventory를 보존했다.

2MiB response 제한과 typed 불변조건은 구현되어 있으나 최대 경계/escaping 전용 시험, 모든 adapter 경쟁/fault 행렬을 완료한 것으로 주장하지 않는다. 원 실제9는 모두 NeedsRepair이고 사용자 품질 승인0/9다. 실제 media/시간축·재생권한 연결, GUI 원문/후보/초안·수정·명시판정·이력 연결과 시각 인수는 남는다. 원 audio clip107~132초와 cue끝133.8초 차이를 자동 보정하지 않는다.

R67 전체190/정상 GUI·CLI bins gate는 이 선택 검증으로 대체하지 않는다. 원 R63의190/2bins 근거는 해당 입력에 유지한다. 최종 custody 및 후속 전체 gate는 별도 실제 결과로 갱신한다.

적용 스킬: modootoday/software-qa/test-fixture-and-mock-build SHA256 A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100; code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B. 사본 fixture/실제 SDK·CLI/사용자 품질과 역사/현재 입력의 증거를 구분하는 데 적용했다.
