---
id: evidence:kinekino/20261029114552
title: R72 CLI 최초발행·거절·구성별 회귀
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R72 CLI 최초발행·거절·구성별 회귀

최종1626 exit0/41356ms·2318입력 content/identity 동일을 확인했다. 아래 작성 시 진행 중 표기는 완료로 보완한다. 실제9 root의 원CLI receipt 파일 수103과context24도확인했다.

2026-10-28 중앙 PM. [SDK/CLI 연결](review-media-sdk-cli-20261028.md)의 미실행 항목을 같은 R72 입력에서 추가 실행했다. 코드 변경0이며 SOURCE8BB055789721B9C59E5E0776AA68C24399393E9826464B569BE55963059A8795/inventoryA577365058D695D9C9EC814023C5B97C7939D3024AA2C016B9C77FF6F24F8052/2318파일을 유지한다. 정상 CLI는157C5B7258ECD196F2C933E3E574CE471574D1D693CB5D2A565C042D1DC50DAD/37574656bytes다.

## 원 실행

로그 root `runtime/verification/source375-datalab-b0-r1-logs`에 원 stdout/stderr/receipt를 보존한다.

|로그|범위|실제 결과|
|---|---|---|
|1609|Native discovery/model/historical integration no-run|exit0/31774ms|
|1610|기본CLI100명령·schema/status|exact1 통과/32430ms|
|1611|project/proposal/job 저장소 구성100·무변경|exact1 통과/32640ms|
|1612|모델 catalog 구성100·선언보존/admission 없음|exact1 통과/38054ms|
|1613|기존 실제9 import/사본 수정/판정/저장/재열기/거절103요청|exact1 통과/39786ms|
|1614|빈 대상에 실제Sintel media package 최초발행|exit0/2026ms·replayed=false|
|1615|같은 exact package 조회|exit0/537ms·replayed=true|
|1616|request의추가 source_path|invalid_argument·예상거절|
|1617|미등록 selection ID|not_found·예상거절|
|1618|inspect timeout context|invalid_argument·예상거절|
|1619|기본SDK 미구성|unsupported·예상거절|
|1620|settings SHA 불일치|revision_conflict·예상거절|
|1621|settings flag pair 누락|invalid_argument·예상거절|
|1622/1623|새 두명령의 명시 media 구성 schema|exit0·각supported/strong_cancel=false|
|1624|명시 media 구성 capabilities|exit0·두명령supported/read·write=true·clock/품질false|
|1625|실제9 source-context CLI24요청/typed9·거절/무변경|exact1 통과/19869ms|

C2 1610~1613/1625는 모두 원 child exit0/nonforced/familyEmpty/ownership complete/stdout·stderr EOF를 확인했다. baseline은 `runtime/verification/source-sdk-r72-cli-regression-index.json`의 바이트pin 검증이며 compilation proof가 아니다. Cargo1609 생산 로그·기존R72 snapshot·고정 exe/PDB hash를 함께 연결한다. 고정 사본/목록은 기존 fixed-r72 및 source-sdk-r72-fixed-artifacts.json에 갱신했다. 예외 fixture 예상 오류의 CLI 프로세스 exit1을 회귀 실패나 예상하지 못한 제품 결함으로 계상하지 않는다.

## 최초발행과 원본 보존

CLI 최초발행은 실제 gemma270-highlights case와 명시 local Sintel로 별도 `runtime/verification/actual9-media-cli-r72-r1/fresh`에 실행했다. 설정은 fresh-settings.json/SHA763AD125BB03B13475491E251D158E07DC10C48C60370948D919D03CA0AE7068이다. 새 대상의 package SHA는 B1E2BE1056E044CD3A8F9E41D16D768B6292DE81C2467890D79EEE844268E390이며 기존R70과 같은 결정적인 내용이다. 이 대상에는 처음 쓰였으므로1614 replayed=false이고1615 조회는true다. 원9 전부를 이번 새 대상으로 최초발행했다고 확대하지 않는다. 나머지8의 최초발행은 원R70 근거에 유지한다.

PowerShell의 native Get-ChildItem/Get-FileHash/Compare-Object로 원R70 inventory와 현재 원자료 root 전체21파일을 비교해 추가/누락/hash차이0을 관측했다. 처음 경로 구분자 표기 차이로 나온 비교 차이는 Path.GetFullPath 정규화 후 사라졌으며 내용 불일치가 아니었다. 검사 로직을 새 제품 코드나 저장된 개발 스크립트로 만들지 않았다. 실제 비교 입력은 `runtime/verification/actual9-media-cli-r72-r1/original-files-after-18-requests.json`와 `original-files-after-regression.json`에 보존한다. 이 native metadata 관측은 별도 Rust CLI 전후 inventory 시험을 새로 실행했다는 뜻은 아니다. 로컬 영상 SHA F12C070E295B38CFC94EBD61AC3357C3BAC82D1015985F1E8DA93A4C6496C46D도 재확인했다.

실제9 회귀 산출물은 `runtime/verification/actual9-current-cli-r72-r1`, context는 `actual9-context-cli-r72-r1`, C2 outer는 source-sdk-r72-{bare-discovery,configured-discovery,model-discovery,actual9,context-cli}-r1/outer다. CLI 신규 fixture와 설정·원 request는 actual9-media-cli-r72-r1에 유지한다.

## 남은 경계

이 결과는 actual9와 선택 영상의 historical clock 동일성·재생 지원·GUI/품질 인수가 아니다. 실제 원9는 NeedsRepair·사용자 승인0/9다. strong_cancel=false 유지, 실행중취소·deadline/권한/공간/부분발행·ACK/재시작 전체 행렬·설정 중복/remote혼합 CLI의 전수는 남는다. 지정190/정상2bins도 원R67 입력의 근거이며 이번 회귀로 R72 통과를 주장하지 않는다. 기존 R72 양strict/fmt는 원1605~1607 완료 근거를 재사용하고 코드 변경 없이 재실행하지 않았다.1626 최종 custody는 작성 시 진행 중이다.

다음 구현 전에 MKV 경계를 닫아야 한다. 현재 infrastructure/review_content_classification.rs의 IsoBmffReviewClassifier는 ftyp 브랜드를 가진 ISO BMFF만 인정하고 실제 선택 영상은 Matroska다. MIME/hash 선택 성공을 이 classifier 통과로 바꾸지 않는다. 실제 컨테이너의 bounded header 분류·공통 content class/adapter 계약과 원 source custody·새 timeline producer·재생의 실제 지원을 연결해야 한다. 포맷이 다르다는 이유로 caller MIME를 신뢰하거나 검사를 생략하지 않는다.

source375 private overlay·기존 소유권/WIP·원 실패/증거·보류를 유지한다. 새추론/다운로드/수동GUI/WARP/push0, canonical 적용0, goal active. 적용 스킬은 기존 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100)와 code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)이며 현재 실행/원 성공·예상 거절/실패·metadata 관측/제품 인수를 분리했다.
