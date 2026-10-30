---
id: sot:kinekino/windows
title: Windows 프레임 프로세스 보유 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Windows 프레임 프로세스 보유 계약

[R50 실제 근거](../../verification/application/frame-owned-family-20261028.md): 원 후손/단일child·실제 frame host와짧은launch Timeout분류를검증했다.나머지fault/취소/pending복구인수는아래범위대로남는다.

## 현재 private 구현

`infrastructure/frame_capture/process/owned`는 기존 owned-process launcher와 Supervisor를 소비한다. frame helper의 Windows run 경로만 연결하며 비Windows의 기존 helper를 검증한 것으로 표시하지 않는다. [결정 기록](../../../adrs/20261028-frame-native-process-custody.md).

프로그램은 명시 절대 경로이며 argv를 직접 전달한다. shell·PID 검색·재개방으로 실행/정리하지 않는다. private frame callers의 기존 환경 상속과 이름별 override/remove만 조립하며 임의 Command/env_clear 호환을 주장하지 않는다.

준비 전에 원 work cutoff와2초 cleanup reserve를 고정한다. stdout은 caller의 기존 상한, stderr는128KiB다. raw regular files는 별도 reader로 file identity와 크기를 검사하고, child exit·family Empty·최종 receipt가 확인된 뒤에만 결과 bytes를 읽는다. pipe EOF 관측은 false로 남긴다.

부분 launch 실패의 원 owner는 Session에 남는다. 원 오류 code/stage를 보존하며, TimedOut은 Timeout으로 매핑한다. child 종료 코드는 원 Windows u32 값으로 기록한다. 강제 종료 여부와 family 관측은 별개다.

## 미완료와 응답

Session은 보유한 원 Supervisor를 poll한다. physical_closed와 final_receipt_recorded가 모두 확인되기 전에는 결과를 반환하지 않는다. 기존 SDK worker가 반환하지 않은 호출을 계속 보유하며, host shutdown도 worker 완료로 잘못 표시하지 않는다.

receipt 실패는 최초 시도를 보존하고 자동 파일 재생성/재시도를 하지 않는다. owner는 같은 호출에 pending으로 남는다. R58은 원 SDK operation/token에 바인딩된 명시 재시도만 같은 owner/sink로 소비하도록 연결했다. 실제probe receipt충돌·원sourceguard 유지·새receipt/첫오류보존을 선택 인수했다. CLI/wire 노출·연속실패/상한 행렬은 미완료이며 정리가 불완전한 경우 end-to-end 반환 기한을 보장하지 않는다. 이것을 정상 종료/취소 ACK로 바꾸지 않는다.

R52는 FrameCapturePort::capture_controlled로 원 SDK MutationControl을 file frame adapter의 probe/process poll과 발행 직전 guard에 전달한다. 기본 legacy 포트는 입구 확인만 하며 실행 중 협력 취소 지원으로 표시하지 않는다. Windows file adapter의 원 family 정리 뒤 Cancelled 반환은 선택 시험으로 확인했다. SDK queued 취소·transport EOF·wire cancel ACK·실행 중 cancel·물리 정리 완료는 계속 구별한다. wire cancel ACK는 지원하지 않는다. PNG 검증과 원 no-overwrite publication은 유지하며 발행 후 늦은 취소로 실제 저장 성공을 되돌리지 않는다. 실제 발행 전/후 경합은 R53 시험으로 별도 인수 중이다.

## 필요한 인수

원 R47 실패와 같은 후손 fixture, 단일 child deadline, 실제 고정 FFmpeg의 endpoint CLI/typed frame 결과, 짧은 deadline 오류 code, 출력 상한·부분 launch·receipt failure/pending, 활성 frame EOF/취소와 panic 경로를 각각 검사한다. 앞의 일부 시험 통과만으로 나머지 행렬이나 전체 CLI/GUI·품질을 완료로 표시하지 않는다.
