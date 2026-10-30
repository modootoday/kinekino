---
id: sot:kinekino/live-timeline-custody
title: Live timeline custody 발행 — 구현 목표 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Live timeline custody 발행 — 구현 목표 계약

후속1E2B3767에서 필수 revision/역할 계약·Rust producer pin을 갖는 live_2_1과 fresh publisher token 기반 v2 reader를 실제 application retain까지 검증했다. [경계](live-timeline-reader.md), [실제 인수](../../verification/application/timeline-live-reader-20261027.md). 초기 live_2 및1CD5 첫 ID 실패 자료는 소급 인수하지 않는다. SDK/정상CLI 저장·재생·startup/재시작과 fault 전체는 남는다.

후속 DA8C42F7에서 기존 package 새 버전의 실제 발행/재열기·원본/판정 보존·취소 미출판·동일 package replay를 검증했다. [발행 근거](../../verification/application/timeline-package-publication-20261027.md). 아래 미완료 중 publisher 기본 경로는 구현됐으나 v2 reader/입장·SDK/CLI/제품 연결과 fault 전체는 여전히 남는다. commit 기록을 Verified 권위로 승격하지 않는다.

2026-10-27 중앙 PM의 현재 구현 공백을 닫기 위한 설계 선택이다. 현재 구현 완료나 사용자 품질 승인을 뜻하지 않는다. [기존 v1](timeline-producer-bundle.md)의 archive reader·동일 item asset 선언을 보존하고, [live producer](timeline-production.md)의 실제 원 실행을 연결한다.

## 저장 경계

새 asset registry나 caller manifest 권한을 만들지 않는다. 기존 ReviewPackage/EvidenceAsset 및 ReviewFiles의 packages/objects 체계 안에 **명시적인 새 package 버전**을 발행한다. 원 review root/package/asset/decision/user association은 수정하지 않는다. 발행 대상 root는 trusted host가 고정한 별도 private root이며 원본·user transaction·runtime·도구 root의 겹침을 거절한다.

새 package는 원 package의 모델 조건·기존 출력·기존 자동검사·known issues를 보존하고, 선택 item에만 새 기술 증거 asset과 기술 검사 항목을 추가한다. 새 package SHA는 새 ReviewCase identity다. 원 case/draft/association/품질 판정은 새 identity로 자동 이전하거나 복제하지 않는다. 모델 실험의 과거 호스트/성공은 과거 사실로 보존하며 새 timeline 기술 관측을 새 모델 실행 성공으로 표시하지 않는다. 새 item의 source-request와 media ID/SHA는 원 선택 case와 같아야 한다.

발행된 기술 검사의 Passed는 timeline 일관성 범위만 입증한다. actual9의 NeedsRepair·사용자 품질 승인0/9는 바뀌지 않는다. source/candidate/baseline 자산과 모델 pin을 임의 교체하지 않는다. 형식·개수·package512KiB·asset 크기 등 기존 경계를 초과하면 명시적 ResourceExhausted로 거절하며 전역 한도를 넓히지 않는다.

## v2 발행과 입장

기존 schema1의 observer_accepted·WAV·과거 CLI trace를 새 원 Job receipt로 소급 변환하지 않는다. 별도 strict v2 manifest가 live source/Job/command/input/출력·raw PCM·origin/끝점 trace를 지정한다. 작은 manifest와 큰 raw 관측은 책임을 분리하며 같은 선택 item에 모든 asset을 선언한다. live BUNDLE.json의 내용 검증만으로 Verified 입장을 만들지 않는다.

v2 authority reader를 연결할 때 mapping_revision과 역할별 원 실행 계약은 명시 필수 필드여야 한다. 누락된 값은 default/역사값으로 채우지 않는다. DA8C42F7의 초기 live_2는 로컬 custody 발행 시험 자료이며 아직 authority reader가 인수한 안정 형식이 아니다. 필수 필드를 추가한 후계 형식/입력은 새 epoch로 검증하고 초기 자료를 소급 승인하지 않는다.

publisher는 원 ProducedTimelineBundle과 source case lease가 살아 있는 동안 원 asset과 파생 파일의 SHA/size/일반 파일 경계를 대조하고 create-new/원자 출판으로 새 객체와 package를 발행한다. 원 tool과 실제 producer 식별·원 PID/creation/exit/Job empty·강제 여부를 실제 기록대로 보존한다. numeric exit와 과거 observer_accepted를 섞지 않는다. 근거에 없는 CLI 실행을 만들지 않으며 새 v2의 원 producer는 실제 Rust producer다.

publisher만 생성할 수 있는 Rust-only opaque 발행 token을 사용한다. token은 source case→새 case 관계·명시된 manifest·source/tool pins와 발행된 asset guard를 결합한다. JSON/SDK 요청·임의 파일 검색에서 token을 재구성하지 않는다. 재시작 후 이미 발행한 자료를 지명하는 trusted host nomination은 별도의 명시 입장이며 자동 복원 권한이 아니다.

resolver는 기존 application TimelineEvidenceResolver/TimelineEvidenceLease 계약을 구현한다. case/media/reference 불일치는 파일 조회 전에 거절하고, 새 package의 item 선언과 실제 source/tool/trace/PCM/PNG를 확인한다. 원본 case/asset 관계는 새 package에 선언한 원 package 기록과 비교한다. 원 모델 조건/출력 보존을 확인하지 못하면 입장을 거절한다. 새 case의 원 세션과 asset guard를 lease가 유지한 후에만 기존 application retain과 Verified 저장전이를 사용한다. 저장된 Verified만으로 playback_ready를 true로 만들지 않는다.

## 실패와 검증 순서

객체 준비 전 취소·입력 변경·예산 초과는 package 미출판이다. 객체만 남은 부분 실패는 orphan이며 case로 등록하지 않는다. package 원자 출판 뒤의 ACK 유실은 committed identity를 보존하며 재시도에서 덮어쓰지 않는다. publisher 오류·취소·deadline과 원 process 물리 종료를 구별한다. 원 Job이 남아 있으면 source/tool/input owner를 유지하며 출판으로 우회하지 않는다.

구현 순서는 live 관측 동시 보유/공통 domain 검증 → publisher의 기존 package 새 버전·원본 보존 → v2 resolver와 opaque token → 실제 SDK/정상 CLI 저장/재열기/재생 → 제품 startup/전체CLI 인수다. 실제 publisher/원자 출판/ACK/CAS·v2 reader·Send/Sync 경계·재시작·diskfull/root교체는 아직 미검증이다. GUI·AI 품질 인수는 CLI 선행 조건 이후이며 수동 GUI·추론·다운로드·WARP·push 보류를 해제하지 않는다.
