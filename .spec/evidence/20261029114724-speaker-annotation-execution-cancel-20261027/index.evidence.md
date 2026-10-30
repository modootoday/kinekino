---
id: evidence:kinekino/20261029114724
title: 화자 편집의 원 SDK 실행 취소와 원자적 저장
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 화자 편집의 원 SDK 실행 취소와 원자적 저장

2026-10-27 중앙 PM. 기존 [화자 annotation 계약](../../player/application/analysis/manual-speaker-annotations.md)에 원 PlayerSdk/SDK execution의 외부 control을 연결했다. source375 private overlay·기존 담당/WIP·원본 증거·보류를 유지한다. 외부 추론이나 GUI를 실행하지 않는다.

## 입력과 변경

710 snapshot SOURCE311E2EE6F275D1C324B34129020179AFDCFEB1D6972DB3DE8BC14ADB131E411B, inventory68B8F4DF868BF21251DABD5B44DC3B1C87C89ADE8F6AE98297B27ACF11F1DD39/2241은 runtime/verification/source375-speaker-cancel-input-r1이다. 원3파일은 source375-speaker-cancel-preimage-r1에 보존했다. api-mod SHA3B36F9264BDA7F1DA1219AC9AC2443480CE7DB098695A7D52732C0CDA126C91D, speaker-application96B0DED062763DDA864BD1485A303675E1AEE4E9B160C184C4BC6FFC1D385F04, speaker-tests94C13AC4C80B2B0CA7B94136988931E32FC920684771A651ACD11805C6169053다.

PlayerSdk의 speaker dispatch가 execute_controlled로 원 operation control을 전달한다. 기존 직접 execute는 None 경로를 유지한다. Deadline은 기존 timeout과 원 control을 합성한다. read/edit와 LocalSpeakerFiles의 저장 정책·immutable original·원 content SHA·create-only commit은 변경하지 않았다. capability는 같은 프로세스 취소와 wire_cancel_ack=false·cancel_after_publication actual_success_retained를 분리한다.

708 기본 all-targets strict·709 fmt·711 Native all-targets strict는 같은 입력 source에서 통과했다. 로그는 runtime/verification/source375-datalab-b0-r1-logs/708~에 보존한다. 기존 모델 취소의 성공을 이번 화자 파일 저장 시험으로 대체하지 않는다.

## 실제 실행 결과

712 Native lib/jobs/speaker no-run은 exit0/132537ms다. 713 speaker_annotations_cli6/6(3.15초),714 sdk_execution19/19(0.05초)를 통과했다. speaker6은 기존4+새2이며 기존 normal CLI14개 실행과 새 두 실제 filesystem/SDK execution 사례를 구분한다. 새로운 사례를 CLI 원격 취소 시험으로 계상하지 않는다.

원5개 root는 runtime/build-cache/tmp/speakers-cli-36772-{1793073049465094200,1793073049474893200,1793073049494639000,1793073049511925700,1793073050812558700}에 유지하고 `runtime/verification/speaker-cancel-execution-r1/{direct-cancel,after-publication,before-publication,negative,edit-reopen}`에 복사했다. 원 증거와 복사본의 FileID를 같다고 주장하지 않는다. 두 gate EXECUTION.json은 cancellation_requested=true/queued_cancelled=false/worker_closed=true를 기록한다. before는 published_before_cancel=false/output_exists=false/error=cancelled, after는 published_before_cancel=true/output_exists=true/response.ok=true다. 원 input 쓰기 차단과 반환 뒤 재개방·원 bytes 및 after output SHA 불변까지 assertion으로 확인했다.

715 Product145/145,716 Core18/18,717 Native26/26,718 자동OS1/1을 직렬 실행해 이 입력의 선택190/190을 통과했다. Product는712 새 Cargo JSON, 변경 없는 독립 Meari는622/624의 기존 artifact를 사용해 실제 시험을 다시 실행했다. Meari 재컴파일이나 전체 CLI 인수로 확대하지 않는다. 719 정상 bins2개 test=false/fresh=true도 통과했다. player SHAF79557BF29FB8C4080DF45B71225317FAAE4EF32A9637817A0AA37F3CE051832, CLI59243A7E80A0C923F128DD0C9F0AE3423E8424233340AD633BB88CEC98644AA1다.

## 새 실제 파일 시험의 범위

Rust helper tests/speaker_annotations_cli/cooperative_execution.rs는 Windows LocalSpeakerFiles를 실제로 사용하고 원 execution handle로 취소한다. 독립 임시 저장소마다 bounded gate/RAII release를 두고 원 input 쓰기 잠금·작업 종료 뒤 재개방·worker ACK/close를 확인한다. 주입 gate만 시험용이며 파일 read/write/sync·hash·publication은 실제 adapter다.

저장 전 사례는 .export 임시 파일 생성 뒤 final checkpoint에서 대기한다. 원 handle.cancel→release 후 실제 Cancelled terminal·target/임시 파일 부재·source bytes 불변·lease 해제가 기대 결과다. 저장 후 사례는 실제 publication이 끝나 output이 존재한 뒤 대기한다. late cancel→release 뒤 원 성공 응답·revision2·원 output SHA 동일·source 불변이 기대 결과다. 반환 후 fallible checkpoint나 rollback을 추가하지 않는다. EXECUTION.json은 각 request/actual response/취소 요청/출판 시점/원 input SHA/worker 종료를 보존한다.

합성 JSON의 수동 편집이며 실제 오디오·모델·media PTS·AI 결과/사용자 품질·GUI 인수가 아니다. 원 SDK가 아닌 wrapper/endpoint의 control 전달·원격 취소 ACK·강제 선점·crash/전원/OS IO fault 전수는 미완료다.

modoo.today 검색의 defect-triage/PR-triage는 새 자원 fixture 작업과 맞지 않아 미채택했다. test-fixture-and-mock-build 본문 SHAA393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100을 읽고 원 file resource 보유/실패 시 release·반환 뒤 재개방·mock/실제 경계 구분을 적용했다. 성능 개선/실측은 하지 않았으며 새 개발·시험은 Rust만 사용했다. WARP·추론·다운로드·수동GUI·push 보류를 유지한다.

최종720 verify-current는 exit0/40246ms로 전체2241 content/identity 동일을 확인했다. index는 비어 있고 stage/commit/push·canonical 승격·추가 설치0이다. 이전 원 실패/미실행과 R1/R2 증거를 보존하고 기존 C NVMe 문맥·TMP/TEMP·jobs1/CPU2/BelowNormal·heavy1을 유지했다. 실제 원 후보9 NeedsRepair/사용자 품질0/9·GUI 미완료와 전체 goal은 유지한다.
