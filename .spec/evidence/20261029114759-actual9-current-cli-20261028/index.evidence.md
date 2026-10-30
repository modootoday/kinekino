---
id: evidence:kinekino/20261029114759
title: 실제9 결과의 현재 제품 CLI 검증 — R64
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제9 결과의 현재 제품 CLI 검증 — R64

최종1530 terminal/exit0·입력2312 content/identity unchanged 확인. 아래 작성시진행중 표기를 실제종료로보완한다. 원패키지/manifest 입력은 시험끝의 개별SHA 재확인으로 별도 보존을 확인했다.

2026-10-28 중앙 PM. 기존 Gemma270/Llama1B/Qwen0.6 × correction/cards/highlights의 실제9 결과를 최신 정상CLI A7C1894E로 재검증했다. 모델 실행·원문/후보 재생성·사용자 품질 승인은 수행하지 않았다. 원 source375/root canonical 미적용 private 후보이며 원 실패·소유권/WIP/보류를 유지한다.

시험 harness의 HISTORICAL_INPUT_WORKSPACE만 추가했다. 격리 source workspace에서 원 private .spec 패키지/manifest를 명시 읽기 입력으로 선택하고, 새 evidence/workspace 사본에만 수정·변조·누락을 주입한다. 미지정 시 원 CARGO_MANIFEST_DIR 경로를 유지한다. 제품 import/SDK/verifier 정책은 변경하지 않았다. preimage-r63-actual9-input/harness.rs에 원본을 보존했다.

입력2312 SOURCE3752D6C4A0C2C31FE828AAEB9318DCEC96DCC814995F0BD086C6C85D40BF984D,inventory39B02F7D640551DA342DF4C16723F054B9F5A15C003EEE5FD7EDC5CB6C70A1FB. AGENTS40C7F5A6 핀과 이전R63 gate1522 terminal0를 재확인했다. 원 historical companion SHA2567A7CD05EAFA6294E846C8080C976600EA135AA91E9DA315AC9F770E45F624B10/16566bytes, 기존 Rust verifier E00C1726603EF7D0167C6D58DEB79E694071044C20A2723DE8ABE8A504F45E41/1127424bytes를 확인했다. verifier는 기존 고정 도구의 새 실행이며 새 빌드로 주장하지 않는다.

|로그|범위|실제 결과|
|---|---|---|
|1523|fmt write|terminal0|
|1524|Native historical test/정상CLI no-run|terminal0/10677ms|
|1525|snapshot|2312 고정|
|1526|actual_nine_binding_fresh_cli_accept_retry_and_project_capture|exact1pass/0filtered/30.68초,runner0/42379ms|
|1527|Native all-target strict|terminal0/1071ms|
|1528|기본 all-target strict|terminal0/773ms|
|1529|fmt check|terminal0|

103개의 CLI request별 receipt/stdout/stderr/PID/exit/reaped를 runtime/verification/actual9-current-cli-r64-r1에 보존했다. C2 바깥 receipt는 child0/Empty/nonforced/physical_closed=true다. 원 package 모든 파일과 세 모델 결과 manifest·AGENTS를 시작에 pin하고 끝에서 SHA 동일을 확인했다. 후보/generated raw와 historical binding을 보존한 9개 import/fresh inspect는 schema3/revision1/NeedsRepair다. 이후 기술적 fixture 수정/채택/재요청·판정 이력·프로젝트 재열기와 source/candidate/companion 누락·변조/부분publication 거절은 작업 사본에서만 검증했다. 기대 오류 응답을 정상 기능 성공으로 세지 않는다.

원 source/후보/draft 비교는 같은 proposals.inspect 응답의 document.source/generated/draft로 가능하고 accepted output target/criteria와 판정 이력은 별도 계약이다. 기술적 accept와 합성 수정 텍스트·declared verdict는 실제 의미 수선/사용자 품질 승인이 아니다. 실제9 원 상태는 모두 NeedsRepair, 사용자 승인0/9를 유지한다. 원 range=null을 임의 타임라인이나 verified media 권위로 보충하지 않았다. R61/R63의 technical sprite 재생 성공도 actual9 media 재생 성공으로 계상하지 않는다.

고정 test fixed-r64/actual9-cli-tests.exe SHA2568ABEEA76B83307D5E898524C64A8D928433EFCA2CE64F4A7CEDA208B8171C6B5/2400768bytes, PDB11EF5851CF88C1FEA81E3B6C8377B22BBEC8C768031772300D787C5AAB3841FC/424824832bytes 및 정상CLI A7C1894E1915B35C7A8F96BF67EA3DD86EAADAD7F7B8E0B3A2A7F1784FBA691D/37023232bytes 사본을 보존했다. index는 runtime/verification/source-sdk-r64-actual9-index.json, raw는 source-sdk-r64-actual9-r1/outer 및 source375-datalab-b0-r1-logs/1523~1530*다.1530 최종custody는 작성시진행중이다.

다음은 [실제9 GUI 연결 준비표](../../../plans/player/actual9-review-ui-readiness-20261028.md)의 비교/수정/명시판정/이력과 실제 미디어 재결합 공백이다. GUI 구현·실행시각·전체CLI/AI품질 인수는 아직 미완료다. R63 지정190/정상bins는 원입력 결과이며 R64 재실행으로 승계하지 않는다. C-onlycache·heavy1/CPU2/BelowNormal·locked/offline/-j1, 새 추론/다운로드/수동GUI/WARP/push 없음.

modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B) 적용: 실제보존결과/기술적fixture/품질승인·현재실행/과거도구·원본/사본의 경계를 구분했다.
