---
id: plan:kinekino/20261029113544
title: Worker control 후속 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Worker control 후속 관측

2026-10-21 SDK actual139의 [검증 정본](../../../sots/verification/ai/vad-worker-control-process-2026-10-21.md)에서 남긴 SDK 시험 범위다. 아래는 계획·source-only 후보이며 실행 권한은 중앙 execution-leases에만 있다. 실행 슬롯은 반환했다. 전체 제품 목표·Media 생산 admission·UI 품질 범위를 축소하지 않는다.

## 단조 축소 caller cap — 작성, 실행0

`runtime/development/vad-worker-native-timeout-r1` source3 `1423A7C338A69B2D00495969509FB1504E100B4B526E76D09DF09277D038682A`, before3 `B16C620D8658A9CA721D0E7DE7B7F44D0A6124E1ECF34D20704A712712BD3205`, 원 parent2 explicit override의 union39 `4E30011C1FFF66E253671B6E16F2AADB94980D700E91F9C085506DF75624837C`를 보존한다. 이39는 실제 검증 최종 source39 E9C690과 다른 후보이며 자동 합성하지 않는다.

원 total8/operation4/cleanup4를 유지한다. engine.ready의 exact bytes/원 guard를 실제 읽은 뒤 caller cap만 원 시작 Instant로 단조 축소한다. 원 NativeTransport Timeout을 보존하고 같은 transport Pending→원 cleanup/EOF/family·초기/최종 동일 process creation identity를 확인하는 시험이다. 새 timer/부모 observer 지연/child cutoff 변경0이다. 고정4s 자연 만료 검증을 대체하지 않는다. 새 Media 경로 수리와의 exact preimage/graph를 재검토한 뒤 별도 배정이 필요하다.

## 고정 부모 operation 자연 만료 — 작성, 실행0

채택 root048808a/실제 source39 E9C690을 기준으로 `runtime/development/vad-worker-native-natural-expiry-r1`에 source3 `18B4B0549D7A2BB91A55478237885C089342387DADCCF72D00A03E7CEFE18FED`, before3 `E8E8471A2E2E8F9011B4DB0A04173AED780C8610F8A947B2825900193868C3E9`, override2 `8C1D49DF3AB2ED2C5FCDEED1EBB5892BD5EA9B08351DB79A8ABD99571E04236E`, union40 `7825BA941BC347F84754A057799FC7423E86D8CBCEA4BD06464C9DD1FAD3C60E`를 별도 동결했다. 부모 driver2개 명시 override와 natural_expiry.rs new1이며 production 변경0이다. exact ignored case1을 작성했고 directfmt3만 통과했다. 모듈 소비점/하위 경로를 읽었으나 Cargo/type/OS/실행0이며 실제139를 상속하지 않는다.

원 고정 parent/child cutoff·observer·cleanup reserve를 그대로 둔다. 부모 observer의 원 Timeout 또는 직전 자식의 strict Failure Timeout 응답이라는 경합 결과를 원 error/reader 분기로 보존한다. Complete/Partial·잘못된 context/응답·다른 오류는 거절한다. ready의 원 bytes/guard와 동일 transport의 첫 Pending 또는 실제 Exited 관측을 확인한 뒤, 원 cleanup과 초기/최종 동일 creation identity·EOF/family·실제 forced 상태·원 raw를 연결한다. 비강제 종료는 exit0/유효 Timeout 응답을 요구하며, 강제 정리 후 비어 있는 stdout은 부모 Timeout 경로에서만 허용한다. 비어 있지 않은 stdout은 항상 원 공통 decoder의 Timeout Failure여야 한다. 결정성을 위해 observer를 늦추거나 deadline을 갱신하거나 child expiry direct-Supervisor case를 부모 observer 증명으로 바꾸지 않는다. 미래 중앙 배정에서 현재 preimage/graph·fixed BIN·실제 등록 목록을 다시 확인하고 이 한 case를 실행해야 한다.

## 범위 경계

현재 flush failure는 fixture writer 주입이며 실제 OS pipe 단절이 아니다. 그런 플랫폼 IO 요구가 배정되면 별도 소유/원 stream/cleanup 관측으로 검증한다. 기존 regular raw EOF·stdin EOF 요청·family Empty를 서로 대신 사용하지 않는다. production native TrustedRelease/LoadedRuntimeAdmission·배포 권리/고지·실제 미디어와 모델 품질은 Media의 별도 계약과 검증을 따른다. sample fixture나 Unsupported-only 결과로 완료하지 않는다.
