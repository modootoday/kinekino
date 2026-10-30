---
id: evidence:kinekino/20261029114442
title: 실제9 로컬 미디어 준비·재열기
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제9 로컬 미디어 준비·재열기

최종1581 exit0/42115ms·2316파일 content/identity 동일을 확인했다. 아래 작성 시 확인 중 표기는 완료로 보완한다. 원 actual9-prepared.json SHA256은356B8C6BF0971593EFAFBCA9C9CDB926ED88B72B74723CADD70349626BCE831B다.

2026-10-28 중앙 PM. [로컬 준비 경로](review-media-preparation-20261028.md)를 실제9 원자료와 로컬 원영상으로 실행했다. 원 패키지와 판정·proposal은 변경하지 않았으며 명시 선택의 새 package/case만 별도 root에 만들었다. 실제9와 미디어의 historical 동일성·시간축·사용자 품질을 승인한 결과가 아니다.

## 입력과 실행

R70은 R69 제품 코드를 유지하고 `tests/review_media_preparation.rs`에 opt-in 실제9 시험을 추가한 private 입력이다. SOURCE SHA256 283F4BBE2BA4F8A43500E5B42379D9691C6770BB87CA850907FCD4B9224859EE, inventory949E124199DA00397319678D5CD8033C316D69BB1D0632373B61EA80090132C6/2316파일. snapshot은 mirror `runtime/verification/input-r70`, preimage는 `runtime/verification/preimage-r69-actual9-media`다.

원 package는 `.spec/artifacts/quality-review/sdk-text-results-r3/packages/659b3cb6757259d8e770f6d3c96c4a77dafbab43c6ea540bcc7e0c58e4eb0197.json`, 로컬 선택 영상은 `runtime/ai-spike/source/Sintel.2010.720p.mkv`/681285280bytes/SHA F12C070E295B38CFC94EBD61AC3357C3BAC82D1015985F1E8DA93A4C6496C46D다. 시험이 전후 원 package root 전체와 미디어 hash를 대조했다. 실제 추론·다운로드·GUI/플레이어 실행은 없다.

root `runtime/verification/source375-datalab-b0-r1-logs`의 원 stdout/stderr/receipt:

|로그|범위|관측|
|---|---|---|
|1573|Native integration no-run|exit0/10262ms|
|1574|실제9 준비·재열기·원본 불변|exit0/29743ms·exact1 내부9·3filtered|
|1575|입력 snapshot|exit0/52321ms|
|1576|기존 실파일 준비 회귀|exit0·3pass/실제9 opt-in1ignored|
|1577|Native all-targets strict|exit0/1522ms|
|1578|default all-targets strict|exit0/1100ms|
|1579|fmt --check|exit0|
|1580|PM이 SOURCE 대신 inventory 해시 전달|exit1/27ms·source receipt pin mismatch, 검사 전 거절·원 raw 보존|

올바른 SOURCE pin으로1581 최종custody를 진행했다. 작성 시 그 결과는 확인 중이다. 시험 exe/PDB는 root `runtime/build-cache/native/source-sdk-integration-r1/fixed-r70`, SHA/bytes는 `runtime/verification/source-sdk-r70-fixed-artifacts.json`에 보존한다.1575 read-only snapshot만 실제시험과 겹쳤고 heavy compile/시험은 직렬이었다.

## 실제 결과

Gemma270/Llama1B/Qwen0.6 × correction/cards/highlights의 원9 item ID를 전수 확인했다. 각 item에 새 관계를 추가한 별도 package를 발행하고 기존 ReviewFiles로 완전히 재열었다. 새 NotRun check를 제외하면 원 package 전체와 같으며 원 source/candidate/모델 조건/baseline/기존 checks/criteria/known issues/range를 보존했다. 원 결정 디렉터리도 전후 inventory가 같고 후속 decisions는 비어 있다.

9개 package가 동일한 영상 objects/SHA를 공유한다. 영상을9번 복제하지 않았다. 관계는 원 case/request·원 candidate 전체 SHA와 selected media metadata를 보유한다. 모든 결과가 historical_media_verified=false/timeline_verified=false/quality_approved=false다. 원9 NeedsRepair·사용자 승인0/9는 유지한다.

개별 새 case와 관계 및 원본 inventory의 원증거는 `runtime/verification/actual9-media-preparation-r70-r1/actual9-prepared.json`; 실제 package/object는 같은 디렉터리의 `prepared`다. 이 파일은 개발 시험의 산출물이며 공통 SDK 명령/CLI 발행 receipt나 제품 GUI 인수를 대체하지 않는다.

## 남은 작업

공통 SDK/CLI에서 명시 선택의 준비·관계 조회를 연결하고, trusted assembly가 원 case/선택/root·budget을 보유하게 해야 한다. caller path/JSON을 source 권위로 받아들이지 않는다. 준비된 case를 새 timeline producer→기존 published resolver→association→재생 경로로 연결해야 하며, 원9 선언 시간이나 역사 assets manifest만으로 이 단계를 통과시키지 않는다. correction은 untimed이고 dialogue.wav가 cue 전체를 덮지 않는 한계도 유지한다. GUI 비교/수정/판정/이력·전체 CLI/시각·사용자 품질 인수는 남는다.

R70 지정190/정상bins 재실행은 이번 범위가 아니다. 해당 완료 근거는 원R67 입력에 유지한다. source375 private overlay·기존 소유권/WIP·원 실패·보류 유지, canonical 채택/추론/다운로드/수동GUI/WARP/push0, goal active.

적용 스킬: modootoday/software-qa/test-fixture-and-mock-build SHA256 A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및 code-change-review-verification9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B. 합성 회귀·원9 실제 입력·명시 선택 custody·clock/품질의 증거를 분리했다.
