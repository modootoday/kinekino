---
id: evidence:kinekino/20261029114612
title: Live timeline의 로컬 package 버전 발행
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Live timeline의 로컬 package 버전 발행

2026-10-27 중앙 PM. 배포가 아니라 private 증거 저장소의 새 immutable ReviewPackage 버전 발행이다. 기존 packages/objects/decisions 체계를 재사용하며 새 registry·caller JSON 권한을 만들지 않았다. 원 package/source/candidate/baseline·모델 조건·기존 자동검사·known issues·판정은 보존한다. 선택 item에만 기술 증거 검사와 asset 선언을 추가하며 새 package SHA는 별도 case identity다.

## 고정 입력과 실제 결과

SOURCE `DA8C42F7FF3825E78E09F36C56F6DA965668EEA353EB6C12D37FD844347A8A46`/2216파일, inventory `CE36EAFABE0F3032D4183A86D6FB38BD71320E267498C5996940A15E8928CAFE`, runtime/verification/source375-timeline-publication-input-r1. 원375 private overlay·이전5310/B1B4와 구분하며 canonical 승격이 아니다. preimage와 SHA 목록은 timeline-publication-preimage-r1이다.

| 로그 | 결과 |
|---|---|
|493/494|초기 strict 및 기존 BCrypt checkpoint 복사 경로 재사용 후 최종 native-proposal-window/all-targets strict 통과|
|495|정책23통과/5ignored; 고유 content byte 예산·중복 SHA·overflow 거절 포함|
|497|실제 시험 no-run 통과|
|498|실제1/21.64초 통과: live 관측 회귀·발행/재열기·원 package/판정 보존·취소 미출판·동일 package replay|
|499|fmt --check 통과|
|500|verify-current exit0/38932ms·전후2216파일 content/identity 동일·stage 없음|

원 package SHA `46445F350ACEA475927F838CBA0258E8EAC8A277D14D8DAC981615154052323D`에서 새 SHA `A5516F1BFF65BEE2A94AB642656D86DA40ED3BFC88732F5942251DC8DBFF29D3`를 발행했다. item/source-request identity는 유지하고 package identity만 달라졌다. 원본과 새 package에서 추가된 기술 검사만 제거한 전체 구조가 정확히 일치했다. 원 판정은 변하지 않았으며 새 case 판정은0개다. 원 root 발행은 파일 생성 전에 InvalidArgument로 거절했다.

새 manifest ID `live-e3a944dae5b7-manifest`, SHA `4A32DCA8EFF2CA5529334624CA47A3DDCA0756EFD756D75FAB3D08C68EC05A56`; 결합 기록 SHA `E3A944DAE5B7D41D408E9019ABCD1DCC35D73C99CDB58A14AC9049322688FB5C`. 기존 ReviewFiles/ReviewCaseEvidence로 새 case의 선언된 manifest를 읽었고, 원 case에서는 이 새 reference가 거절됐다. 같은 live bundle 재발행은 같은 package identity/replayed=true로 돌아왔다. SDK ACK 유실 전체 시나리오를 시험한 것은 아니다.

첫 object 생성 뒤 취소는 원 Cancelled를 보존하고 packages0개를 확인했다. cancelled-review/objects에는 source-request 이름의0byte orphan을 보존했다. 실패 객체를 덮어쓰거나 자동 삭제하지 않는다. 손상 orphan이 같은 digest 경로에 있으면 후속 시도는 명시 복구를 요구한다.

원 증거는 `runtime/verification/timeline-publication-actual-datalab-r1/RETURN.json`, producer의 원 실행/파일, cancelled-review와 published-review의 packages/objects, source375-datalab-b0-r1-logs/493~500이다. 실제 producer EXE SHA `520A0AE5B366653F269C55AF61478644ADE7ABCB65DB238416F06D3D330FD02D`.

## 구현 경계

model_review/export는 검증된 원 session과 asset file을 보유한다. producer export는 고정 역할/파일명만 선택하며 임의 경로를 열지 않는다. video의 원 stderr도 물리 종료 후 hash/guard를 보유한다. publication의 streaming copy는 기존 BCrypt/64KiB checkpoint를 사용하며 원본/runtime/tool/명시 제외 root 겹침과 고유 content 총량을 검증한다. package는 기존 create-only atomic publication을 최종 commit으로 사용한다. commit 뒤의 후속 취소를 실패/rollback으로 바꾸지 않는다.

이번 시험은 새 복사/검증 단계를 포함해 시작 시 총 deadline60초·cleanup2초를 고정했다. 기존20초 operation을 실행 중 연장한 것이 아니며 actual21.64초는 새 입력의 측정이다. C 캐시·CPU2/BelowNormal·무거운 작업1개·locked/offline/jobs1과 기존 보류를 유지했다.

PublishedTimelinePackage는 constructor-private/Rust-only commit 기록이며 TimelineEvidenceLease나 Verified 권위가 아니다. 원 package 파일을 바꾸거나 기존 판정/association을 새 case로 자동 이동하지 않는다. v2 manifest reader·opaque 입장/원 trace 교차검증·SDK/CLI Verified 저장/재열기/재생·제품 startup은 미완료다. diskfull/권한교체/전원손실·ACK 전체·host 재시작 행렬도 미검증이다. 최신 전체190/정상bins는 변경 전 B1B46C92 checkpoint다. actual9 AI 품질 승인0/9와 GUI 미인수는 변하지 않는다.

modoo.today `modootoday-free/ai-quality-plus/agent-run-postcondition-and-replay-check` 본문 SHA61843099b7d54c3d7ac226218a5ee1b6ccdbe36ef84268762dbc33a68fbb58b5/worked-example SHA80c9fe8a8a79dd8e08d84b36483d7742f273e55fb2769d2e6156a1111fdd5c11의 원 실행/후조건/주장 분리를 적용했다. 기존 QA fixture/code-change·cache correctness 지침을 적용해 content 예산·원본 보존·취소와 commit/replay를 구분했다. 새 설치·수동GUI·추론·다운로드·WARP·push 없음.
