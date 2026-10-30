---
id: evidence:kinekino/20261029114643
title: 원본 결합 playback.status SDK/CLI 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원본 결합 playback.status SDK/CLI 검증

## 최신 실제 결과

275 최종 감사는 terminal=true/exit0/38635ms로2173파일의 content/FileID/bytes/creation/mtime 동일성을 통과했다. index는 비어 있고 stage/commit/push는 수행하지 않았다.

SOURCE F156BADACAEE12B026A1116DEF776591F1DFA044F6367B38770EB79A1082AD47, inventory D3C828E6E7274A6CC1247A5E6690D62B3E1CF0F3F30BAEFF1171C612D43EBDFB /2173입력에서274 actual1시험 통과(actual4.89초/outer5679ms). 원 Sprite case/media·고정 mpv·실 Job/IPC를 시험 host에 구성하고 정상 CLI5프로세스로 seek2/status2/closed-status1을 실행했다. 직접 SDK의 잘못된 instance 거절도 확인했다.

| 실제 항목 | 결과 |
|---|---|
|seek17초 뒤 CLI playback.status|position_us="17000000", duration_us="629887710", paused=true, has_error=false|
|seek25.5초 뒤 CLI playback.status|position_us="25500000", 동일 instance1, has_error=false|
|SDK foreign instance|RevisionConflict, data=null|
|정상 종료 뒤 CLI playback.status|NotFound, data=null; 이전 clock 재사용0|
|원 process/정리|PID10512/creation134375072178535598, exit0/forced=false/physical_closed/Job Empty/IPC 종료, first_error=None|

status는 engine-time의 기존 microseconds 투영이며 frame-exact/source PTS·Verified·품질 승인이 아니다. 실제 publication age4/9ms는 원 native event age가 아니다. reader/case/media/instance 및 종료 경계는271 synthetic 단위 시험으로도 별도 검증했다. 실제 source와 fake fixture를 같은 증거로 계상하지 않는다.

원 artifact root는 runtime/build-cache/tmp/review-worker-15168-1793033615996019700이다. source/test/CLI/tool/nomination·사전 WITNESS·종료 전 OBSERVATIONS 전후 guard를 확인했다. test producer48D35B73D7AB39E3EEB03611D7F25163B61BB138B315C41658CACF91975862E5, 정상 CLI70B21548E25945A057F35934267A360F1F2466A38691009813B60D038933FD65. 기존 toolF8BF7E8F…/원 media7CFC8B8C…/package46445F35…/requestA518F587…/nominationB1A47D17…를 유지했다.

| 기록 | SHA256 |
|---|---|
|runtime/verification/review-worker-status-actual-datalab-r1/RETURN.json|0244279D605273AE509DBA83A7292B9009F9EB8B33D008E7CA3965E4F48944EF|
|WITNESS.json|4FFF323B237AF4D913D3A565BD2C85744FC8289807C3278342ED5E8A9DB439FD|
|OBSERVATIONS.json|9B97FB3DDA805665DB48ED7DEDA800B5D137159E6630690477767F3CB0CEEC2A|
|evidence/INPUT.json|5F9F9C1302324408C255A5BC787364B0F57D395CD970AAD7124F36EA195BA8B5|
|evidence/PROCESS.json|475738977C0DC6CB2A9C414B54E4648712C71CB044D7C28931B1061D8C7287C6|

270 strict/fmt·271unit2·272no-run·273입력고정·274actual·275입력 재검증을 구분한다. 아래 진행중 설명은 당시 상태이며 이 최신 결과로 갱신한다. 새 입력의 전체190/명시 정상bins는 아직 미실행이고, 이전 F7A32EFE의 완료 checkpoint와 합치지 않는다. public local-open/close·opaque Verified·worker 실패/취소 전체 행렬·actual9/GUI 인수는 계속 미완료다. RETURN의 verified_timeline/actual9_binding/gui_visual_acceptance/quality_approved=false를 유지한다.

271 후속 unit2시험 통과: synthetic source/case/media/instance·closing·physical close·owner 종료/Weak 만료에서 stale clock 거절, prepare 실패에서 child 생성0을 확인했다. 이 fixture 성공을 실제 미디어·SDK/CLI 성공으로 계상하지 않는다.272 actual target no-run을 수행한 뒤 새 source epoch를 고정한다.

2026-10-27 중앙 PM. 최신 AGENTS·소유권·WIP·C cache/heavy1/CPU2·보류를 유지한다. 제품 공개 open/Verified gate를 낮추지 않고 기존 PlaybackStatusPort의 실제 engine clock 연결부터 진행한다.

## 보존한 기준선

SOURCE F7A32EFE…/2172입력의 선택190/정상 bins/정상 CLI3시험은 이전 완료 checkpoint다. 수정 전 application review_playback/mod.rs, review_worker.rs/run.rs, actual test와 test producer/정상 CLI를 runtime/verification/review-worker-r3-baseline에 사본으로 보존했다. 사본 verifier211611F2…/CLI0B527048…는 원 witness SHA와 일치하며 physical FileID는 다르다. 원 Sprite 봉인 bundle·r1/r2 실패·r3 성공의 원 기록은 변경하지 않았다.

## 구현과 현재 검사

worker State는 원 source identity와 publication 시각을 보유하며, status reader는 case/media/engine을 고정하고 Weak로만 관측한다. 다른 case/media/instance, closing, physical close, worker 종료/해제에서 이전 clock을 돌려주지 않는다. 공통 SDK의 기존 checked engine-time 투영을 사용한다. publication age를 event freshness나 source PTS로 오인하지 않는다.

269 strict는 새 actual test의 불필요한 ApplicationClient import에서 실패했다. 원 stderr/receipt를 보존하고 import를 제거했다.270 제품/owned-process all-target clippy -D warnings와 fmt check 통과. 기존 dependency blitz-paint 경고는 담당 WIP로 유지했다.271은 synthetic source/instance/종료/Weak 만료 거절 및 prepare 실패의 unit 검증이며 완료 후 별도 갱신한다.

정상 CLI actual 후속은 같은 source/instance의 seek 뒤 playback.status 관측과 종료 후 NotFound를 검사한다. source/test/CLI/tool WITNESS 및 close 전 OBSERVATIONS를 유지하고 explicit fresh output directory를 요구하므로 이전 RETURN을 덮어쓰지 않는다. 완료 전 실제 성공 계상0이며 새 미디어 실행 전 새 source snapshot을 고정한다.

기존에 읽고 적용한 modoo.today test-fixture-and-mock-build/code-change-review-verification의 synthetic/actual·owner/observer·SDK receipt/원 관측 분리를 적용한다. 정확 출처 연결은 [worker 검증](review-local-worker-20261027.md) 및 [이전 통합](source375-review-worker-regression-20261027.md)을 따른다. 수동 GUI·WARP·추론·다운로드·push는 재개하지 않는다. CLI/GUI/AI 품질 전체 완료로 계상하지 않는다.
