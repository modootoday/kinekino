---
id: plan:kinekino/20261029113735
title: 결과 wire 연결의 담당 분리
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 결과 wire 연결의 담당 분리

후속 C0 설계 정본은 [v1 계약](../../sots/contracts/playback/native-result-export/index.md)과 [정확 producer/소비/공유 경로 계획](native-result-export-c0-producer.md)이다. 아래 최초 producer4/소비10은 소유 anchor 이력이며 actual target의 issuer.rs·Engine 전달·전역등록/출력·bounded getter 필수 경로를 후속 계획에서 구분했다. 현재 C 생산 코드 배정/실행은0이다.

2026-10-21 PM 읽기 조사 인수. [phase2](native-result-export-phase2.md)와 [wire 계약](native-result-export-wire-contract.md)의 구현 배정 준비이며 지금 C 소스 편집·실행 권한은 아니다. 현재 SDK B11, 미디어 harness8, UI quarantine13의 배정은 유지한다.

## 소유권과 선행 계약

SDK는 application/playback_session/native_result_export.rs와 mpv/guarded_command/{exports,results,scope}.rs의 공통 producer4를 소유한다. 새 Engine 전달이 필요하면 playback_session/mod.rs·session_engine.rs를 별도 정확 범위에 추가한다. B11 작성 중 이 경로를 소비 담당이 동시에 편집하지 않는다.

소비 담당 후보10은 skip_segments/native_port.rs, host/{registry,contracts,dispatch,schema,mod}.rs, api/client/skip_segments.rs, CLI {native_host,playback_host}.rs, tests/native_skip_cli.rs다. CLI 조립2는 마지막 통합에서 연결하며 지원 flag를 먼저 열지 않는다. 전역 capabilities/schema 명령 등록과 시험 모듈 등록이 추가로 필요한지 실제 소스로 확인한다. 기존 service/ports/domain 정책은 보호하고 단순 정책 복제 대신 필요한 getter 한 곳만 근거로 조율한다.

SDK가 B 핀 이후 공식 issuer scope/reference의 버전·길이·strict codec, 원 annotation/verified revision·timebase·actual target 보존, 원64 row의 submit/ref 결합과 list/read/consume 오류 순서를 먼저 고정한다. 현재 opaque Rust reference는 공식 wire codec이 아니다. 제안된 길이/응답 cap은 실제 직렬화 검증 전에 확정 성공값으로 쓰지 않는다.

## 병행 순서

1. SDK가 공통 타입·행동 계약을 고정한다. 소비 담당은 그 전에도 유한 response projection·RequestContext 분류·원 선택 상태 추출과 회귀를 준비할 수 있다. 독자 ref 문자열이나 임시 mock을 정본으로 고정하지 않는다.
2. SDK producer4와 소비 경로는 별도 overlay에서 구현한다. 실제 seek target은 원 NativeSkipCall.submit의 Seek.position_us에서 전달하며 이후 현재 segment를 읽어 과거 target을 재구성하지 않는다. 무상한 segments snapshot을 응답에 복사하지 않는다.
3. 생산자 핀·typed API 인수 후 소비 소스를 정확 핀으로 결합하고 CLI 조립2 및 fresh CLI gate를 수행한다. 합성 CLI와 실제 mpv/landing·GUI 품질은 분리한다. Cargo/shared/index는 PM이 한 실행 담당에게만 배정한다.

## 필수 검증과 보호

공식 ref roundtrip·foreign/restart 거절, 수정 뒤 원 metadata/target 불변, stale/cancel preobserve0·same view/guard1·post-core 오류의 원 ref 보존, 출력 유실 뒤 bounded list 복구/추가 seek0, pending consume 권리 보존·terminal1회/후속 NotFound, 전체64·미소비 eviction0, 개별/전체 escaping byte cap, strict schema/context/typed client/CLI 오류 동등성, EOF 정리가 필요하다. 응답 유실 뒤 NotFound는 전달 ACK가 아니다.

SDK B/A·원 CAS/fence/permit/readcell/64/8KiB는 SDK 보호 대상이다. 소비10·새 시험은 소비 담당 책임이며 Meari/UI·기존 segments.skip DTO·지원false는 보존한다. 각각 before/source pin과 최종 상대 핀을 기록한다. 조사·소스 작성·컴파일·실행·제품 지원을 구분하며 이번 읽기 결과는 실행 성공이 아니다.
