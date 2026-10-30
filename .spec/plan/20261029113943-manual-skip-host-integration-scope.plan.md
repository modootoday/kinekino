---
id: plan:kinekino/20261029113943
title: Manual skip host·SDK·CLI 구현 경로
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Manual skip host·SDK·CLI 구현 경로

최종 통합 인수: PM root **43e6d9276b912992d818661dd0d4d25eaa634b5c** exact18/index비움/푸시0. [실제 gate 정본](../../sots/verification/application/manual-skip-host-2026-10-20.md)에 고유46pass·CLI4/37·양 strict/fmt0·source18/보호615·PID 재사용 정정과 반환을 마감했다. 이후 native 실제 seek/host 재시작/동시성/inflight 취소·OS generation·모든 builder skipowner 직접 계측·durable/자동 timer는 별도 미검증 범위다. 현재 UI capture/shell 배정이므로 추가 재시험·구현 없이 보고만 마친다.

최종 시험 전용 r3: source17 `203D85C372ED61FB2F4A9C90976FD6C543C0FF8DCA100CC48E0F7B233D96E4B3`, tests/manual_skip_cli.rs 1파일만120줄 추가, 나머지16/공유 patch 불변. evaluate/override·stale/canonical snapshot·합성 call0 회귀1건을 더해 registry5+CLI4 작성/미실행이다. gate의 CLI 예상37호출/전체 active46으로 갱신했다. 기존 C5E8 원 핀은 보존하며 현재 Meari heavy 중 Cargo/CLI/공유 snapshot 변경0이다.

다음 실제 실행을 위한 [gate 계획](manual-skip-host-gate.md)을 고정했다. shared6/core2 정확 patch·신규9 SHA·현재 보호 목록·통합 핀을 보존하고 core/host20, fresh endpoint CLI3/22요청, PNG7, 자막9+4, parity2와 양 strict/fmt를 단계별로 검사한다. 실제 stdout 수를 정본으로 삼고 원 helper blocking wait/join 및 PID 회수 조건, 합성 host와 실제 native restart의 차이를 명시했다. 현재는 Meari 소유 중이며 계획 작성 외 적용·Cargo·CLI·snapshot 변경은 없다.

독립 읽기 검토 인수: source17 C5E8…3AE6/공유·core patch 불변, 확정 추가 결함0. 실행은 현재 Meari heavy 반환 후 중앙 배정까지0이다. 통합 시 core commit eb626726757d7297e8fe82e7c11b74e8f3173853과 PNG를 보존한 정확 patch만 적용한다. 실제 gate의 필수 마감 조건에 원 runner/child handle·소유 host/CLI·receipt·PID 부재 확인을 포함한다. 기존 local_host_cli helper의 blocking wait/join 때문에 실제 종료를 관측하기 전 성공·정리·heavy 반환을 주장하지 않는다. native host 실제 재시작/동시성/in-flight 취소/모든 builder 직접 계측은 별도 미검증 범위다.

source17 작성·핀 완료: [소스 계약](../../sots/player/application/manual-skip-host.md). host-r2 manifest C5E8E4DF6F1B0B13DC4AFC2E13853599F9DC8333BF4D46D8D26CE3588B063AE6, shared6 patch349FC80D…493D, core2 extension patchBCAD36EB…BC9F. 현재 eb62672 root에 읽기 apply-check만0이며 실제 적용은 하지 않았다. registry5+CLI3 작성/미실행, 기본 Native seekUnknown·durablefalse·hosttickfalse 유지. 다음은 독립 읽기 검토 후 중앙 패치 합류와 별도 자원 배정 gate다. 코어 r3 실제15와 혼합 인수하지 않는다.

2026-10-20 PM이 승인한 다음 단계의 exact scope 제안이다. core r2 AAB8F0753960A38E20C9A8A97C4D051F1C33F623113A2B90B1AAB943B88C86A7은 읽기 인수됐고 실제 Cargo 시험은 아직 없다. [연결 계약](manual-skip-sdk-cli-contract.md)을 따른다. source-only로 구현하며 실행/새 host 프로세스는 배정 전 시작하지 않는다.

## 최소 변경 경로 17

신규9:

1. `src/application/skip_segments/host/mod.rs`: unavailable/default API port와 공통 노출.
2. `src/application/skip_segments/host/contracts.rs`: opaque host/session 식별·strict wire DTO·typed 결과.
3. `src/application/skip_segments/host/registry.rs`: 한 host의 동기화된 ephemeral SkipSession 소유, open/lookup/close·checked counter·closing·stale binding. 정책은 core를 호출한다.
4. `src/application/skip_segments/host/dispatch.rs`: 공통 RequestContext/서비스 변환, expected_revision 단일 권위, 오류 보존.
5. `src/application/skip_segments/host/schema.rs`: host 주입/seek 지원별 명령 capability와 strict 요청/응답.
6. `src/application/skip_segments/host/tests.rs`: 다중 요청 소유·revision·close/restart·unknown 지원·seek call0/1 fixture.
7. `src/infrastructure/skip_session_identity.rs`: 기존 Windows CoCreateGuid 방식의 host generation 제공, 실패 전파. 정책/다른 엔진/새 dependency 없음.
8. `src/application/api/client/skip_segments.rs`: 기존 review_call를 재사용하는 typed extension trait.
9. `tests/manual_skip_cli.rs`: 기존 LocalHost와 Rust-owned injected host fixture에 여러 fresh CLI를 연결하는 실제 인수 시험. 실제 native 긍정 seek로 표시하지 않는다.

기존8:

10. `src/application/skip_segments/mod.rs`: host 모듈만 additive 등록.
11. `src/application/skip_segments/contracts.rs`: 기존 response3에 Deserialize 추가만. r2 취소 정책 불변, 변경은 별도 새 핀.
12. `src/infrastructure/mod.rs`: identity 어댑터 cfg Windows additive 등록.
13. `src/application/api/mod.rs`: optional/shared registry port 주입·기본 unavailable·기존 builder 이전·dispatch/schema/capability 합류. PNG snapshot_export 필드를 모두 보존한다.
14. `src/application/api/client.rs`: extension 모듈과 trait additive 재수출.
15. `src/application/api/capabilities.rs`: 기존 segments.* 이름과 신규 lifecycle/evaluate/skip/override 명령 등록. detect는 미지원 유지.
16. `src/bin/player-cli/native_host.rs`: 기존 engine/session을 쓰는 한 registry 주입·종료 무효화. 새 implicit daemon/engine을 만들지 않는다.
17. `src/bin/player-cli/playback_host.rs`: 같은 소유 방식의 주입·종료 무효화.

domain/application 루트 mod는 core 등록 patch에 이미 있으므로 전체를 다시 교체하지 않는다. 새 Cargo/dependency/CLI flag/GUI 변경은 없다. API 중앙3파일과 host2는 현재 live 최신 root(a611f791bad209f156a95d1f2e26cc2dce8499ec 관측) 내용을 기초로 독립 source diff를 준비하고, PNG 마감 후 중앙에서만 정확히 병합한다. 오래된 독립 checkout의 PlayerSdk 전체 본문이나 공통 mod를 snapshot/live에 덮지 않는다. 전체 expected17와 registration/중앙 수정 diff를 분리 핀한다.

## 소유·가용성 결정

각 host는 OS에서 만든 고유 generation과 checked session counter를 가진 하나의 registry를 SDK에 주입한다. 요청은 opaque skip_session_id와 현재 engine/resolution binding을 모두 제시한다. registry는 같은 SDK 인스턴스에서만 여러 CLI 요청의 state를 유지한다. 외부 JSON snapshot을 state로 재주입하지 않는다. host 종료 플래그를 먼저 세워 새 요청을 거절하며 이미 동기 호출된 seek의 취소·join을 발명하지 않는다.

capability는 세션 편집/평가와 실제 seek submission을 분리한다. stock native port의 seekUnknown/미인가 관측은 그대로 두고 skip 명령은 Unsupported, evaluate는 차단 이유를 반환할 수 있다. synthetic trusted port만 주입된 인수 fixture에서 skip의 call0/1을 검사한다. 모든 경우 durable=false·detection_supported=false·host_tick_supported=false다. Auto preference가 저장돼도 무인 실행이나 자동 검출을 지원한다고 표시하지 않는다.

Mutation은 RequestContext.expected_revision을 checked parse해 core DTO로 매핑한다. payload의 중복 expected_revision은 받지 않는다. missing host/default SDK는 세션 상태를 임시 생성하지 않는다. 세션 open/create와 current active binding을 확인하는 관측의 범위, endpoint lifetime/close·역탐색 pass guard를 schema·실제 반환과 일치시킨다. `segments.detect`는 모델 없이 성공시키지 않는다.

## 검증 배정 후

먼저 core15·새 registry 시험과 strict/fmt로 실제 컴파일을 확인한다. 이후 fresh CLI 바이너리 고정→동일 LocalHost에 별도 프로세스 open/edit/verify/policy/inspect/evaluate/skip/override/close→잘못된 endpoint·restart·stale session·unknown native 차단을 검사한다. 실제 요청/응답/exit와 host generation/session/pass/revision·seek counter·pre/post exe SHA를 보존한다. 현재 준비·source 구현은 실제 통과나 Native skip 지원 근거가 아니다.

스킬: 앞서 읽은 modoo.today requirement-to-testable-stories의 상태 수명·의존·실패 경계와 code-change-review-verification의 소스/관측/미검증 구분을 적용한다. r2 revision+1/stale flag 취소의 직접 assert 공백은 core 정본에 기록했고, 새 회귀로 보완할 경우 별도 scope/pin에 명시한다.
