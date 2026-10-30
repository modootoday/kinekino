---
id: evidence:kinekino/20261029114610
title: receipt 복구 연속 실패·세대 상한 — R59
status: active
created: 2026-10-29
updated: 2026-10-29
---
# receipt 복구 연속 실패·세대 상한 — R59

최종1488 기본 all-target strict terminal0/17886ms,1489 fmt check0/3837ms,1490 custody terminal/exit0·입력2311 content/identity unchanged 확인. 아래 초기 작성 시 미완료 표기는 이 후속 실제 결과로 보완한다. fixed-r59/recovery-budget-tests.pdb SHA256 D395284F91DDBC6CB40C269B61CBBCF060D843CB90AB2B261B6A1363AB3316EC/1183494144bytes도 보존했다. 이번 R59 전체190/정상bins는 미실행이며 원 R58 근거를 승계하지 않는다.

후속 실행 결과:1482 Native product lib no-run terminal0/114670ms,1483 snapshot2311. SOURCE SHA256 2A95E8FEE3C84A4AB53D2637595E1A43E0DC50206FDB59D9180FAAB5928422C1, inventory744CD3E9156D3F430CA00D167E010F31EEB68EE13C236E14FAEF901A28402C8D. fixed-r59/recovery-budget-tests.exe SHA256 B09AAED1505D6601F215A47939EA3ED95F807C13A7E4BE5477495253CDC40422/192604160bytes로 고정했다.

1484 SDK execution21pass/4ignored/1536filtered(0.05초,runner0/59ms). 새16세대 상한·17번째 및 반복 등록거절·추가retry20회거절/Running/worker미종료와 기존authority·model취소 회귀를 실제 확인했다. synthetic client 해제는 제품 소진후 복구로 계상하지 않는다.

1485 실제 SDK/native probe 2연속충돌 exact1pass/1560filtered/0.50초(runner0/15810ms). generation1 요청으로 재실패한 뒤 generation2 미요청 상태·oldtoken거절·receipt0002부재·원sourceguard 유지, 두 번째 명시 요청 후 receipt0002·첫EngineFailed 보존/미발행/source동일·후속SDK정리를 확인했다.1486 단일충돌 회귀 exact1pass/0.48초(runner0/4919ms). 양 C2 외부 receipt는 child0/Empty/nonforced/physical_closed=true이며 원receipt0000/0001 sentinel과 새receipt/SDK응답은 각 TMP에 보존했다.

1487 Native all-target strict terminal0/23904ms. 기본 strict1488·fmt·최종custody는 작성 시 아직 미완료다. 아래 초기 작성 시 진행 중 기록은 이 실제 결과로 보완한다. 이번 새 실패는 없다.

근거 runtime/verification/source-sdk-r59-recovery-budget-index.json, source-sdk-r59-{repeated,single}-receipt-r1/outer/*, source375-datalab-b0-r1-logs/1480~1488*. 실제 원process evidence와 sdk-response.json은 runtime/build-cache/tmp/source-sdk-r59-{repeated,single}-receipt-r1/frame-owned-*에 있다.

2026-10-28 중앙 PM. [R58 연결](frame-sdk-receipt-recovery-20261028.md) 뒤 남은 연속 실패/16세대 상한을 확인한다. source375/root canonical 미적용 private 후보이며 preimage-r58-recovery-budget에 원 recovery.rs/시험을 보존했다. AGENTS40C7F5A6 핀과 R58 gate1479 terminal0를 확인하고 시작했다.

기존 16세대 제한은 초과 등록을 거절하지만 조회가 마지막 consumed 상태만 보여 상한 소진 여부가 명시되지 않았다. R59는 ReceiptRecoverySnapshot.attempt_budget_exhausted를 추가하고, 소진을 최초 관측했을 때만 notifier를 호출한다. 마지막 유효 token의 추가 요청은 ResourceExhausted, 이전 token은 기존 stale 거절을 유지한다. generation/권한/원 owner는 재생성하지 않으며 시간 경과·반복 poll·취소로 재시도를 시작하지 않는다.

실행 계층 synthetic client 시험은 명시 요청16개/각1회소비, 세대마다 이전 token 거절·중복 AlreadyRequested,17번째/반복 등록 거절과 소진 표시, 추가 retry20회 거절·Running/worker pending 유지, 첫 오류 보존을 확인하도록 작성했다. 마지막 시험용 client 해제는 실제 native owner의 해제/상한 후 복구가 아니다. 제품 Session은 상한 소진 후에도 원 owner/guard를 pending으로 보유한다. 상한 소진 뒤 별도 관리 복구·CLI/wire는 미구현이다.

실제 SDK/native probe 시험은 기존 1회 충돌 사례와 새2회 연속 충돌을 분리한다. test-only checkpoint decorator가 receipt-0000/0001에 immutable sentinel을 저장하고 첫 재시도도 실패시키며, 새 generation2·원sourceguard 유지·자동 재기록 부재·oldtoken 거절을 확인한 다음 명시 요청으로 receipt-0002를 기록한다. 첫 EngineFailed/원 sentinel·미디어 SHA 보존·미발행·정리/후속 요청은 그대로 검사한다. 실제 저장 fault는 create_new 목적지 충돌이며 partial write/권한/디스크 고갈 전부를 뜻하지 않는다.

1480/1481 fmt write0,1482 Native product lib no-run/1483 snapshot은 작성 시 진행 중이다. 시험 통과로 계상하지 않는다. 원 R58 지정190/정상bins는 원 입력의 실제 결과로 유지하며 R59로 승계하지 않는다. 전체CLI/GUI/AI품질0/9·제품provider 연결 및 나머지fault는 미완료다.

C-only cache/heavy1/CPU2/BelowNormal/locked/offline/-j1, 컴파일 중 읽기 snapshot만 병행한다. WIP/소유권/원실패/보류 유지, GUI/추론/다운로드/WARP/push 없음. modootoday/software-qa/test-fixture-and-mock-build SHA256 A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및 code-change-review-verification9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B 적용: 합성 세대 시험·실제 원 SDK/native probe fault·제품 지원 범위를 구분한다.
