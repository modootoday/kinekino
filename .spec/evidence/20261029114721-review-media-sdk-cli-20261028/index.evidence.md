---
id: evidence:kinekino/20261029114721
title: 명시 미디어 준비·관계 조회 SDK/CLI 연결
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 명시 미디어 준비·관계 조회 SDK/CLI 연결

최종1606 default strict exit0/27005ms,1607 fmt exit0/2828ms,1608 custody exit0/42105ms·2318파일 content/identity 동일을 확인했다. 아래 작성 시 진행 중 표기는 이 실제 종료로 보완한다. 이번 실제 compile/시험/CLI/strict/fmt/custody의 첫 실패는 없다. 실패한 문서 apply_patch는 문맥 불일치로 전체 적용 전 거절됐고 올바른 제목으로 재적용했으며 제품코드/시험 실패가 아니다.

2026-10-28 중앙 PM. [계약](../../contracts/production/review-media-preparation.md)의 `reviews.media.prepare`와 `reviews.media.inspect`, typed `prepare_review_media`/`inspect_review_media`를 공통 application/PlayerSdk에 등록했다. 고정 선택 adapter와 해시로 보유하는 CLI 설정을 연결했다. source375 후속 private overlay이며 canonical 채택이 아니다.

R72 입력은 mirror `runtime/verification/input-r72`: SOURCE SHA256 8BB055789721B9C59E5E0776AA68C24399393E9826464B569BE55963059A8795, inventory A577365058D695D9C9EC814023C5B97C7939D3024AA2C016B9C77FF6F24F8052/2318파일이다. 관련 SDK/source API/client/capability·기존 integration 시험·CLI runner 원본은 mirror `runtime/verification/preimage-r70-media-api`에 보존한다. 이름이 같은 두 mod는 review-source-mod.rs/sdk-mod.rs로 별도 보존했다. infrastructure-mod-after-module.rs는 신규 module 선언 후 캡처이며 변경 전 원본으로 계상하지 않는다. 해당 원본 식별은 R70 inventory에 유지한다.

새 request는 case와 selection_id만 받는다. expected_revision/idempotency_key를 거절하고, prepare의 bounded timeout을 publication 전 control checkpoint에 적용한다. inspect는 timeout도 거절한다. 응답은 exact selection ID/원 case/새 case 관계와 historical media·timeline·품질false를 typed client에서 재검사한다. 기존 generic source service와 기본 SDK는 새 adapter를 자동 생성하지 않는다. discovery는100으로 늘었지만 완료율이나 지원100을 뜻하지 않는다.

고정 adapter는 application의 준비 body를 계산하고 기존 ReviewFiles의 content-addressed package/object 검증을 사용한다. 저장 완료 조회는 외부 영상 path가 없어져도 가능하되 원 case와 고정 선택 metadata는 검증한다. 설정 파일은 기존 RetainedReviewStartupSettings의 SHA/64KiB/guard로 보유한다. CLI flag는 두 설정flag+local review/user roots를 함께 요구하고 remote endpoint와 혼합하지 않는다. SDK payload 경로는 거절한다.

## 원 실행

로그 root는 `runtime/verification/source375-datalab-b0-r1-logs`다.

|로그|범위|결과|
|---|---|---|
|1583|R71 SDK/adapter 구현 초기 Native strict|exit0/35494ms·CLI 설정 후계 전|
|1585|R72 Native integration compile·시험|exit0/114370ms·4pass/1실제9 opt-in ignored·0.11초|
|1586|R72 snapshot|원 terminal receipt 보존|
|1587~1604|정상 CLI 실제9 prepare/inspect 각9|18요청 모두exit0·ok=true·replayed=true|
|1605|R72 Native all-targets strict|exit0/29426ms|

integration은 실제 filesystem adapter를 typed SDK로 소비했다. 새 prepare·inspect·capability100, unknown selection NotFound, request 추가 path InvalidArgument, readonly prepare Unsupported와 외부 media rename 뒤 readonly inspect 성공, inspect timeout InvalidArgument, 기본SDK Unsupported를 확인했다. 기존 원본 보존/replay·잘못된 case/media/budget/선행 취소·partial object 무덮기3시험도 같은 R72에서 통과했다.

정상 CLI는 SHA157C5B7258ECD196F2C933E3E574CE471574D1D693CB5D2A565C042D1DC50DAD다. 설정·18개 원 request JSON은 `runtime/verification/actual9-media-cli-r72-r1`; settings SHA6A785CA0CAB2A33E1B40AD06506107932DAE73A90A7CF58BBA0DC89F06579C39다. 선택은 실제 local Sintel/681285280bytes/F12C070E295B38CFC94EBD61AC3357C3BAC82D1015985F1E8DA93A4C6496C46D이며 원9/source request/candidate 참조를 응답에서 전수 확인했다. 18개 모두 R70의 기존 prepared package hash와 같은 exact replay/inspect다. 이번 CLI에서 처음9개 media package를 새로 발행했다고 주장하지 않는다. 신규 발행은 R70 실제9와 이번 합성 SDK 시험의 별도 근거다.

CLI18 응답은 item·원 case·새 package·선택 ID가 각각 대응하고 historical_media_verified/timeline_verified/quality_approved=false였다. 개발 시험 사본의 기술 판정을 사용자 승인으로 바꾸지 않았고 원9 사용자품질0/9·GUI 미인수 유지다. 이번18요청 뒤 원9 전체 inventory를 다시 검사한 별도 Rust 시험은 아직 없으므로 R70의 원본불변 시험을 이번CLI 전후 검증으로 승계하지 않는다.

CLI/시험 exe/PDB 고정 사본은 `runtime/build-cache/native/source-sdk-integration-r1/fixed-r72`, SHA/bytes는 `runtime/verification/source-sdk-r72-fixed-artifacts.json`에 보존한다. Cargo의1585 생산 로그와 R72 snapshot을 함께 사용하며 복사본 FileID를 원 생산자 식별로 취급하지 않는다.

## 미완료

작성 시 default strict/fmt/최종custody는 후속 진행 중이다. 정상CLI의 새 destination 최초발행·설정변조/누락/중복flag·기한/취소/저장fault·readonly 구성 전체, discovery100의 기본/저장소/모델 schema 전수, 기존103 CLI 회귀는 R72에서 아직 미실행이다. 관련시험의 기대수100 수정은 실행 통과를 뜻하지 않는다. strong_cancel_available=false를 유지한다.

새 실제 media producer→published resolver→association·재생, historical clock 관계의 실검증·GUI 비교/수정/판정/이력과 시각 인수는 남는다. 지정190/정상2bins는 원R67 결과에 유지하고 R72로 승계하지 않는다. WIP·소유권·원 실패·원 증거·보류 유지, 새 추론/다운로드/수동GUI/WARP/push0, goal active.

적용 스킬: modootoday/software-qa/test-fixture-and-mock-build SHA256 A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100, code-change-review-verification9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B. fixture/실제9·SDK/정상CLI·신규발행/replay·역사/현재 검증 범위를 분리했다. C-only heavy1/CPU2/BelowNormal/locked/offline/-j1,1586 read-only snapshot만 compile과 겹쳤고 후속 mutation은 없었다.
