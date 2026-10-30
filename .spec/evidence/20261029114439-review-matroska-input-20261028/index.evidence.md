---
id: evidence:kinekino/20261029114439
title: Matroska 검토 입력 분류·원 custody
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Matroska 검토 입력 분류·원 custody

최종1634 default strict exit0/18564ms,1635 fmt exit0,1636 custody exit0/41636ms·2320입력 content/identity 동일을 확인했다. 아래 작성 시 진행 중 표기는 이 실제 종료로 보완한다. root AGENTS SHA40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C도 재확인했다.

2026-10-28 중앙 PM. [컨테이너 헤더 계약](../../contracts/production/review-content-classification.md)에 따라 bounded Matroska 분류기와 video/x-matroska 대응을 추가했다. 제품 preparation과 원 session launcher는 기존 ISO BMFF와 새 Matroska를 분기하는 LocalReviewContentClassifier를 공유한다. 실제 header를 읽기 전에 MIME만으로 형식을 인정하거나 runtime을 요청하지 않는다.

R73 private mirror snapshot은 `runtime/development/source-sdk-integration-r1/runtime/verification/input-r73`: SOURCE SHA36A28383E75AC6B096292B0B0659B7DA8871675CB8082757CFED0B23AFB9CA5C, inventory92265B80BE16A7322F16565EBA498986200D70ADE9A74544E42637B797D090F7/2320파일이다. 원 변경5파일 preimage는 mirror `runtime/verification/preimage-r72-matroska`에 보존하고 parser와 실제 입력 integration2파일을 새로 추가했다. canonical/source375/원 Media seal을 변경한 것이 아니다.

## 실행 결과

원 로그 root는 `runtime/verification/source375-datalab-b0-r1-logs`다.

|로그|범위|실제 결과|
|---|---|---|
|1627|fmt 정렬|exit0|
|1628|Native all-targets strict|exit0/26322ms|
|1629|Native lib/실제입력 integration no-run|exit0/199904ms|
|1630|입력 snapshot|exit0/70703ms|
|1631|헤더 분류·잘림/중복/버전/doctype/크기 거절|2pass/exit0/11ms|
|1632|공통 input/runtime 경계·기존MP4·MIME불일치·MKV lease|4pass/exit0/16ms|
|1633|실제 Sintel Matroska 입력 보유|1pass/exit0/698ms|

같은 소스 lib 시험 실행파일은 kinekino_player-37bec848ae83f5da.exe, integration은 review_matroska_input-e551bac2a9ee53b2.exe다. 원 exe/PDB 사본은 `runtime/build-cache/native/source-sdk-integration-r1/fixed-r73`, SHA/bytes는 `runtime/verification/source-sdk-r73-fixed-artifacts.json`에 보존했다. lib 전체1564를 실행한 것이 아니며 선택2/4만 실제 인수했다.

1633은 R72 새 대상 `runtime/verification/actual9-media-cli-r72-r1/fresh`의 gemma270-highlights case를 직접 소비했다. packageB1E2BE1056E044CD3A8F9E41D16D768B6292DE81C2467890D79EEE844268E390, source request658E5FBD…8093, mediaF12C070E295B38CFC94EBD61AC3357C3BAC82D1015985F1E8DA93A4C6496C46D/681285280bytes·declared video/x-matroska를 확인했다. ReviewFiles의 전체 content 검증과 raw prefix 분류 뒤 원 GuardedReviewMedia를 보유하고 check_current를 확인했다. 외부 디코더·mpv·GUI·모델을 생성하지 않았다.

## 범위와 미완료

읽은 표준은 [RFC8794](https://www.rfc-editor.org/rfc/rfc8794.html)의 EBML/VINT/header 의미와 [RFC9559 §4.3](https://www.rfc-editor.org/rfc/rfc9559.html#section-4.3)의 Matroska 제약이다. 실제 지원은 계약의 bounded header 범위이며 body·codec·CRC checksum 검증·DocTypeExtension·모든 버전/컨테이너 전체 호환을 주장하지 않는다. 공개 source class는 additive matroska_header이며 기존 iso_bmff_header wire는 유지한다. 원 producer trace는 source identity 전체를 대조하므로 형식 선택만으로 기존 token/근거를 재사용하지 않는다.

이 결과는 준비된 실제 파일의 source custody와 분류 경계를 입증한다. actual9의 원 모델 실행과 같은 미디어/clock을 증명한 것, Verified association/새 producer/fresh runtime admission·engine 재생·GUI 인수가 아니다. 원9 range=None·사용자품질0/9는 그대로다. 다음은 새 실제 producer의 bounded timeline/PCM/capture 근거와 정확한 package/case/association 재생 연결이며 기존 기술 sprite의 성공을 이 MKV 성공으로 승계하지 않는다.

작성 시 default strict/fmt/최종custody는 후속 진행 중이다. 이번 첫 strict/compile/시험 실패는 없다. 지정190/정상2bins는 원R67, CLI100/103/24회귀는 원R72 입력에 유지한다. R73에서 해당 전체 gate를 재실행했다고 주장하지 않는다. 원 소유권/WIP/선택소스/실패/원증거/보류 유지, 새추론/다운로드/수동GUI/WARP/push0, goal active.

적용 스킬: modootoday/software-qa/test-fixture-and-mock-build SHA256 A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100, code-change-review-verification9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B. 바이트 분류·원 guard/retained input·runtime grant·actual producer·품질을 분리했다. C-only/heavy1/CPU2/BelowNormal/locked/offline/-j1 유지,1630 read-only snapshot만 no-run compile과 겹쳤고 소스 mutation은 없었다.
