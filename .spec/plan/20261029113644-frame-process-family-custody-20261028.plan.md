---
id: plan:kinekino/20261029113644
title: 프레임 프로세스 원 family 소유권 연결
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 프레임 프로세스 원 family 소유권 연결

2026-10-28 최신: [R52](../../sots/verification/application/frame-sdk-cooperative-cancel-20261028.md)의 원 SDK 실행중 취소/정리 선택 시험과 [R53](../../sots/verification/application/frame-sdk-publication-cancel-20261028.md)의 실제 로컬 decoder 발행직전 취소·실제 저장후 늦은취소 성공보존2사례를 통과했다. F5/F6의 해당 선택 경계만 인수하며 wire ACK/활성EOF/receipt 복구/전체fault 행렬은 미완료다. R52 지정190/정상bins는 원 입력의 실제 통과이며 R53 재실행으로 승계하지 않는다.

2026-10-28 후속 [R50 구현/근거](../../sots/verification/application/frame-owned-family-20261028.md): F0 원사례2.02s통과,F1 원launcher/F3 두cutoff와raw/receipt연결을구현했다.F2/F4는같은호출stack의Supervisor와borrowed poll_cleanup을통해owner를보유하며정상/기한경로를검증했다.부분launch·receipt/termination fault전수와명시pending복구는아직미완료다.F5/F6 실행중취소·발행경합및F7전체후속gate는남는다.

중앙 PM, 2026-10-28. [R47 재현 실패](../../sots/verification/application/frame-descendant-deadline-failure-20261028.md)가 다음 구현의 기준이다. 이 문서는 실행 계획이며 미구현 인터페이스를 현재 지원으로 표시하지 않는다.

|요구|현재 근거/공백|구현·인수 조건|
|---|---|---|
|F0 원 실패 보존|1381의2초 요청이30.04초 뒤 반환|원 source/producer/raw 유지, 새 구현에서 같은 선택 사례 통과|
|F1 생성부터 원 family 소유|frame process는 std Child만 사용|기존 owned-process WindowsFamily/spawn의 suspended→assign→resume 계약 사용, PID 검색·재개방 금지|
|F2 부분 생성 실패 보유|기존 spawn은 LaunchFailure.owner를 반환함|원 SDK/session이 owner와 source/tool guard를 먼저 보유, 실패를 직렬화하며 owner를 버리지 않음|
|F3 deadline와 cleanup 분리|부모 종료 뒤 reader join이 기한을 초과함|원 작업 기한을 갱신하지 않고 cleanup을 별도 관측, raw EOF와 family Empty·child exit·receipt 완료를 구별|
|F4 미완료 owner 전달|기존 Supervisor::FinishOutcome::Incomplete가 원 owner 보존|동기 Result<Output,ApiError>로 불완전 owner를 소실하지 않도록 원 SDK 실행/session 경계부터 연결. 별도 registry·보유 없이 background detach 금지|
|F5 실행 중 취소|FrameCapturePort에 cancel 인수 없음|공통 SDK 실행 제어의 실제 요청·정리·terminal 판정을 연결한 뒤에만 지원 표시. CLI 종료/EOF를 자동 취소 ACK로 해석하지 않음|
|F6 발행 경합|정상 PNG 검증/no-overwrite 근거 존재|취소 후 미발행/원본 보존, 발행 뒤 늦은 취소의 원 성공 보존을 원 publication 경계에서 검증|
|F7 실제 회귀|R46 정상 host EOF·실제 frame·190/bins|후손 실패→통과, 정상/기한/부분 실패/receipt 실패/cleanup Pending·활성 EOF를 검사한 뒤 새 입력의 영향190/bins 검증|

우선 매핑할 기존 Rust 계약은 `packages/owned-process/src/infrastructure/windows/launch.rs`의 SpawnSpec/LaunchFailure, `application/supervisor.rs`의 Supervisor/ProcessFamily/FinishOutcome, 그리고 기존 SDK execution/LocalHost의 미완료 worker 보유 경계다. 원 프로세스와 source/tool/file guard가 최초 실패부터 최종 정리까지 같은 실행 소유권 안에 남아야 한다.

첫 구현 전에 F2/F4의 구체 보유·반환 경계를 확정한다. 현재 동기 FrameCapturePort의 공개 호환 경로를 임의로 성공 처리하거나 ApiError 안에 owner가 보존됐다고 가정하지 않는다. worker가 실제 정리를 끝내기 전에는 cancel/timeout 응답을 물리 종료 ACK로 승격하지 않는다.

scope는 공통 Rust SDK/CLI와 해당 프로세스 어댑터다. 새 GUI·수동 시각 검토·모델 추론·미디어 다운로드·WARP·push는 재개하지 않는다. 기존 소유권/WIP와 다른 담당의 원 증거·cache 문맥을 유지한다.
