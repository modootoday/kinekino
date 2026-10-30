---
id: evidence:kinekino/20261029114536
title: 원 SDK 프레임 실행 중 취소 — R52
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 SDK 프레임 실행 중 취소 — R52

격리 private mirror 입력2306의 SOURCE.md SHA256 AF68B1282EB5E00AD550AA5C615073762B0687AD7D8B3191D233008A66F2F7B8, inventory E0650DEC222409464EFB9AC4EF0CFAD5A09D238659BE6A4EC15404E7BB41B990. 원 root/source375 canonical 채택이 아니다. preimage-r50-frame-control과 preimage-r51-frame-ready에 이전 소스/fixture를 보존했다.

공통 SDK ExecutionOwner의 원 MutationControl을 application frame 포트와 Windows 원 owned-process poll로 전달했다. 일반 포트 기본값은 preflight만 지원하며 capabilities의 실행 중 협력 취소는 file_frame_adapter 범위다. wire cancel ACK를 지원으로 표시하지 않는다. 발행 직전 체크포인트에서 취소하며 발행 이후 실제 성공을 취소로 되돌리지 않는다. receipt 실패 pending 복구·비Windows 실행 중 취소·전체 발행 경계 행렬은 미완료다.

1417 Native no-run terminal/exit0(148520ms). 1418 선택 시험은 실제 SDK ExecutionOwner/opaque operation 취소→원 native family 강제 종료/Empty→physical_closed/receipt_recorded→Cancelled→다음 version 성공·ACK/close를 확인했다. 실제 원본 SHA 유지, 출력 없음, fixture marker 독점 해제를 검사했다. 1 pass/1554 filtered, 시험0.39초, 실행기14203ms/exit0. 바깥 시험 프로세스는 exit0/Empty/nonforced이며 안쪽 fixture 종료124/forced와 구분한다.

이는 핀 고정 Rust held-engine fixture와 실제 로컬 미디어/ffprobe를 사용한 취소 제어 시험이다. 실제 FFmpeg decode·추론·GUI·사용자 AI 품질 인수가 아니다. 원 fixture의 ready 신호는 marker 작성/flush 후 생성해 취소 시점 경쟁을 줄였다.

후속 확인: 1419 Native all-target strict terminal/exit0(27269ms), 1420 fmt --all --check terminal/exit0(3707ms), 1421 최종 custody terminal/exit0. R52 입력2306의 내용·식별 동일을 확인했다. 아래 작성 시 진행 중 기록을 이 실제 종료 결과로 갱신한다. 기본 구성 strict·R52 전체190·정상bins의 별도 최종 gate는 아직 계상하지 않는다.

근거: runtime/verification/source-sdk-r52-frame-control-index.json, source375-datalab-b0-r1-logs/1417-source-sdk-r52-native-no-run.*, 1418-source-sdk-r52-frame-cancel.*, source-sdk-r52-frame-cancel-r1/outer/{stdout.raw,outcome.jsonl,outer-receipt.jsonl}. 고정 lib SHA256 4EE5068C6AAB56DE48E0A8531C6A8957F69BEB8E68E2AFA289F7164EFD2186DF, fixture 55C2D8C2DB32DE75C0F7193C22EE12D6253DEF4BE05680A177D9D8269C02F8BE. 1419 strict는 작성 시 진행 중이며 fmt·최종 custody·R52 전체190 재실행은 아직 계상하지 않는다. 전체190/정상2bins의 최신 완료 gate는 원 R50 입력이다.

modootoday/software-qa/test-fixture-and-mock-build SHA256 A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100과 code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B의 fixture 경계/실제 결과 분리 원칙을 적용했다. 기존 WIP·소유권·실패·보류를 유지한다.
