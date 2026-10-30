---
id: evidence:kinekino/20261029114502
title: 프레임 원 SDK receipt 복구 연결 — R58
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 프레임 원 SDK receipt 복구 연결 — R58

1471 최종 custody terminal0/42100ms, 입력2311 content/identity unchanged 확인. 아래 작성 시 진행 중 기록은 이 최종 근거로 보완한다. 전체190/정상bins의 R58 최종 gate는 아직 미실행이다.

후속 실제 검증:1462 no-run terminal0/117401ms. fixed-r58/frame-recovery-tests.exe SHA256 E786B07D8A123D2F31C89BC9AD8AEF8B1965D9ED2FA9D8969265171A1918F314 /192592384bytes. 1464 원 SDK/실제probe 복구 exact1 pass/1558 filtered/0.55초(runner0/16787ms),1465 실제발행전후 취소 exact1 내부2사례 pass/1.27초(runner0/5727ms),1466 SDK execution20pass/3ignored/1536filtered(runner0/59ms),1467 실제held fixture 실행중취소 exact1pass/0.35초(runner0/13575ms). 각 C2 외부 receipt는 child0/Empty/nonforced이며 실제held fixture 내부 forced124와 구분한다.

1464에서는 receipt pending에서 원 source guard의 exclusive-open 거절을 확인했고, 명시 retry 후 같은 원 child receipt-0001.json의 exit0/Empty/nonforced와 원 sentinel 무변경을 확인했다. 원 응답은 stage=receipt/EngineFailed, owned_process.physical_closed=true/receipt_recorded=true/child_exit0이며 취소 요청은 첫 receipt 오류를 대체하지 않았다. 출력 없음·source SHA 동일·source guard 해제·후속version/ACK/ownerclose도 실제 통과했다. 실제 미디어 probe이며 FFmpeg decode·AI 품질 인수로 확대하지 않는다.

1468 최종 Native all-target strict terminal0/7454ms,1469 기본 all-target strict0/25417ms,1470 fmt check0.1471 최종 custody는 작성 시 진행 중이다. 아래 초기 작성 시 컴파일 중 표기를 이 결과로 보완한다.

근거 runtime/verification/source-sdk-r58-frame-recovery-index.json, source-sdk-r58-frame-{recovery,publication,cancel}-r1/outer/* 및 source375-datalab-b0-r1-logs/1462~1471*. 실제 복구 응답/원 sentinel/새 receipt는 runtime/build-cache/tmp/source-sdk-r58-frame-recovery-r1/frame-owned-30996-1793125440606691600-4에 보존했다. 이번 새 실패는 없으며 이전 R56 첫 실패는 원 기록에 유지한다.

2026-10-28 중앙 PM. [R57 typed 복구 권한](sdk-receipt-recovery-authority-20261028.md)을 PlayerSdk→query→FrameCapturePort→FfmpegFrameCapture→probe/process→원 Session으로 전달했다. 원 review_source MutationControl은 변경하지 않았고 FrameExecutionContext의 cancellation/recovery 참조를 분리했다. legacy capture/controlled와 query wrapper는 기존 경로를 보존한다.

Session은 physical_closed 뒤 최초 record 실패에서만 receipt 재시도 대기를 시작한다. 원 generation의 take_retry가 true일 때만 같은 Supervisor·FileSink에 한 번 record_cleanup하며 기존 실패 파일/첫 오류/원 deadline을 갱신하지 않는다. 재실패는 새 명시 세대를 기다린다. port 보고/소비 실패 또는 상한 초과는 owner를 반환/폐기하지 않고 pending을 유지한다. 반환 성공/취소 ACK를 만들지 않는다. CLI/wire 복구 명령은 미구현이며 capabilities 지원은 추가하지 않았다.

원 소스는 preimage-r57-frame-recovery에 보존했다. 입력2311 SOURCE SHA256 393B0981FD9596D3AD006D8EC354CD2DF6F543B3F52ECF87F96BC4FCE47E0212, inventory B460A91016198FFA385D52F7C2859B27559547E0962ED2DAA4C93D65D64544A2. root/source375 canonical 미적용 private 후보이며 AGENTS40C7F5A6 핀을 재확인했다.

1460은 연결 코드의 Native strict terminal0/35896ms, 이후 실제 SDK receipt fault 시험을 추가해1461 fmt write0·1462 Native no-run을 실행했다. 1463 snapshot terminal0/68856ms. 작성 시1462 컴파일 중이며 아직 실제 시험 성공으로 계상하지 않는다.

새 시험은 실제 핀 FFmpeg8.1.1/ffprobe와 기존 로컬 portrait를 사용한다. test-only checkpoint decorator가 이번 원 child evidence 디렉터리에 create_new/sync로 receipt-0000 sentinel을 만들어 충돌시키며 기존 discovery 디렉터리는 건드리지 않는다. 원 SDK pending/generation 관측→원 source 독점 guard 유지→취소 요청 중 Running 유지→명시 retry→새 receipt/첫 EngineFailed 보존→guard 해제·미발행·원본 SHA 유지·후속 version/ACK/close를 확인하도록 작성했다. 이 테스트용 디렉터리 관측은 제품 복구 권한이 아니며 제품은 원 opaque operation/token만 사용한다.

원 R52 지정190/bins, R53 발행취소, R55 실제receipt primitive, R57 synthetic authority의 증거는 각각 원 입력에 유지한다. 최신 후보 전체190/bins·16세대상한·연속 실제저장 실패/부분쓰기/권한오류·활성EOF·전체CLI/GUI/AI품질은 미완료다. 새 GUI/추론/다운로드/WARP/push 없음, C-only cache·heavy1/CPU2/BelowNormal·locked/offline/-j1 및 원 WIP/소유권/실패/보류 유지.

적용 modootoday/software-qa/test-fixture-and-mock-build SHA256 A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및 code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B: 합성 충돌·원 SDK/실제probe·지원/미검증 경계와 과거 실행을 분리한다.
