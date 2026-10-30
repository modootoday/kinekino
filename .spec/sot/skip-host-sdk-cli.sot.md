---
id: sot:kinekino/skip-host-sdk-cli
title: 수동 skip 장수명 host·SDK·CLI 소스 후보
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 수동 skip 장수명 host·SDK·CLI 소스 후보

## 최종 결합 gate·root 통합

[실제 검증 정본](../../verification/application/manual-skip-host-2026-10-20.md): 고유46pass/외부 corpus ignored1, 최종 CLI4시험37요청, default/native strict·fmt0. 경고용 r4 import 제거·r5 기존 helper 관측 수리와 자막 fixture1 PNG 주입을 별도 보존했고 제품 정책은 바꾸지 않았다. 최종 integrated18 D08F697E7117D09477FCC74E05D9BFA87601463FFAC7C7A444AD35B9AF9F00F7·보호615 차이0, CLI artifact A016D645…6035D5다. PM은 exact18을 **43e6d9276b912992d818661dd0d4d25eaa634b5c**로 통합했다. PID79112 재사용의 원 nonempty 조회와 이후 현재 부재 조회를 분리·정정했고 사용자 앱 kill0·heavy/snapshot 반환을 마쳤다. 실제 endpoint는 검증했지만 포트·generation은 합성이므로 실제 native seek/host subprocess 재시작·OS generation·inflight 취소·durable/자동 timer 지원으로 확대하지 않는다.

## 시험 전용 r3 보완

PM 배정으로 tests/manual_skip_cli.rs 한 파일에 evaluate/override wire 시험1건을 더했다. 최종 source17-r3 manifest `runtime/coordination/manual-skip-host-r3/source17.json` SHA **203D85C372ED61FB2F4A9C90976FD6C543C0FF8DCA100CC48E0F7B233D96E4B3**, test-only.diff SHA **5C39C2E5AFE7D831DEF4F1121C7FE45C63C4396029F9DAFB1FAA47248B070EE5**. 원 C5E8 핀/source/와 shared6/core2 patch는 보존했으며 나머지16파일 해시는 불변이다. 새 명령·정책·native/GUI/OS generation 범위는 추가하지 않았다.

같은 endpoint/session/binding에서 evaluate→manual_seek override→canonical inspect/evaluate, stale revision override 거부 후 snapshot 전체 불변, 명시 새 pass 후 eligible 재활성, cancel 후 blocked와 최종 typed SDK/CLI 상태 동일을 확인하도록 작성했다. 합성 seek counter는0이어야 하며 평가/override가 실제 seek를 수행했다고 표시하지 않는다. 직접 rustfmt check/diff check만0, Cargo/CLI 실행0이다. registry5+CLI4 작성/미실행, 새 wire 시험15호출을 포함해 정상 gate 예상 fresh CLI37이다. 실행계획은 예상 전체 active46으로 갱신했고 실제 stdout/원 call 수를 정본으로 삼는다.

## 독립 읽기 검토 인수

PM이 C5E8E4DF6F1B0B13DC4AFC2E13853599F9DC8333BF4D46D8D26CE3588B063AE6 source/patch 일치와 추가 확정 결함0으로 읽기 검토를 인수했다. 예산1·동일 binding open의 one-pass 보존·mutation current binding/inspect stored-state 구분·close/restart·모든 builder의 PNG/skip owner 이전·Native Unknown 차단은 소스 근거다. registry5/CLI3은 여전히 미실행이다.

실제 native host 재시작, 동시 요청 경합, in-flight 취소, 모든 builder의 직접 계측은 작성된 시험 범위 밖이다. 기존 tests/support/local_host_cli.rs의 blocking wait/join은 원 helper 한계로 남는다. 실제 gate는 원 runner/child handle과 추가 소유 host·CLI의 생명주기를 보존하고, 종료 receipt·소유 PID 부재를 확인해야 한다. timeout·멈춤을 기능 실패 원인으로 단정하거나 unrelated 프로세스를 종료하지 않으며, 실행이 남았을 때 슬롯 반환/정리 완료로 표시하지 않는다.

2026-10-20. PM이 exact17 구현을 배정했다. core r3 실제 검증과 별개로 이 문서는 미컴파일·미실행 source-only 후보를 기록한다. [코어 실제 결과](../../verification/application/manual-skip-core-2026-10-20.md), [연결 계약](../../../plans/player/manual-skip-sdk-cli-contract.md), [범위17](../../../plans/player/manual-skip-host-integration-scope.md)을 참조한다.

## 정확 핀과 병합

최종 `runtime/coordination/manual-skip-host-r2/source17.json` SHA256 **C5E8E4DF6F1B0B13DC4AFC2E13853599F9DC8333BF4D46D8D26CE3588B063AE6**. 출처는 독립 `%USERPROFILE%/.codex/worktrees/subtitle-range-export/com.kinekino.player`이며 core 정책 기초는 r3 9C1BE6D389B41E9E23D089E49058F88CC6A383A068AC53C8A301641069EF15FE다. 신규9·기존8이며 정확 목록은 manifest에 있다. 이전 host-r1 준비 핀도 보존한다.

공통 root/API/host/infra 등록6은 shared-changes.patch SHA **349FC80D5377474C61010A7556D515AA1185E5157E96C902EA97C7688726493D**로 분리했다. 6파일68추가/4삭제이며 PNG snapshot_export 필드와 기존 typed 경로를 보존한다. core2의 host 모듈 등록·응답3 Deserialize 추가만 core-extension.patch SHA **BCAD36EBED474ED482E7FE9DF2366C639AC24C0953FBA7E22A25BA374804BC9F**에 둔다. policy.rs 등 코어 정책은 포함하지 않는다. 현재 PM root eb626726757d7297e8fe82e7c11b74e8f3173853에서 두 patch의 `git apply --check`가0이었다. 실제 적용·stage/index/commit 또는 snapshot 변경은 하지 않았다.

shared 읽기 기초6은 manual-skip-host-prep-r1/before6.json 및 before/ 원문을 보존했다. 전체 source 파일은 검토·해시 정본이지 live/snapshot 전체 덮어쓰기 허가가 아니다. 새 파일9만 정확 소스로 전달하고 기존8은 중앙에서 패치·충돌 검토로 합류한다. root의 최신 PNG·SDK surface·core r3를 오래된 checkout으로 되돌리지 않는다.

## 상태와 수명

SkipHostRegistry는 하나의 host generation과 checked counter로 opaque session ID를 발급하고 Mutex로 ephemeral SkipSession을 소유한다. OS generation은 전용 Windows Rust CoCreateGuid 어댑터에서 만들며 실패를 전파한다. 요청 JSON은 generation을 선택하지 않는다. 현재 native/playback host 조립은 기존 한 playback engine에 맞춰 명시 예산1을 전달한다. 예산0·counter 소진·busy/poison·다른 engine/resolution binding을 명시 오류로 반환한다.

같은 binding을 다시 open하면 기존 ID/state를 재사용해 one-pass guard를 초기화하지 않는다. 같은 endpoint의 장수명 SDK owner만 여러 fresh CLI 요청의 상태를 유지한다. endpoint 없는 CLI의 새 PlayerSdk에는 기본 UnavailableSkipHost가 있으므로 상태를 임시 생성하거나 이전 session을 복원하지 않는다. host generation이 달라지면 동일 engine 숫자라도 이전 session ID는 NotFound다. close는 skip 세션만 제거하고 재생을 닫거나 큐 명령을 취소하지 않는다.

inspect는 지정 ID/binding의 **저장된 ephemeral 상태 읽기**이며 현재 native engine의 일치·재생 증거가 아니다. 미디어 교체 뒤에도 revision을 읽어 old 세션을 명시적으로 닫을 수 있도록 이 읽기는 현재 engine 관측을 강제하지 않는다. 그렇지 않으면 응답을 잃은 이전 revision을 알아낼 수 없어 예산1을 영구 점유하는 문제가 생긴다. 잘못된 ID/소유 host/저장 binding은 여전히 거절하고 edit/verify/policy/evaluate/skip/override는 현재 binding 관측을 요구한다. stale 세션의 inspect 성공을 active 지원으로 표시하지 않는다.

host owner는 종료 flag를 세워 새 요청/새 submit을 차단한다. 기존 SDK/LocalHost executor를 재사용하며 새 worker/타이머/엔진/자동 daemon을 만들지 않는다. 이미 동기 dispatch된 seek를 flag로 철회하거나 native 완료/join을 보장하지 않는다. Arc 소유가 남은 in-flight 호출의 종료는 기존 host 계약과 구분한다.

## SDK·wire·기능 가용성

PlayerSdk는 optional host API의 기본 unavailable·builder 이전과 `with_skip_host`를 제공한다. `with_shared_episode_playback`으로 같은 EpisodePlaybackSessions Arc를 registry와 기존 SDK에 주입한다. 기존 with_episode_playback·PNG snapshot exporter·다른 builder는 보존한다. typed extension trait는 기존 review_call과 공통 core를 호출하며 UI/CLI 전용 정책을 만들지 않는다.

9개 host 명령은 open/inspect/edit/accept_skip/set_skip_policy/evaluate/skip/override/close다. detect는 미지원이다. mutation은 RequestContext.expected_revision 하나를 checked parse해 core 요청으로 매핑하며 payload의 중복 revision은 거절한다. strict 요청/응답 schema와 host 주입 여부·seek submission capability를 함께 제공한다. missing host는 Unsupported이며 transport timeout/disconnect를 자동 retry하지 않는다.

세션 편집/평가와 실제 seek 가용성은 분리한다. native 조립은 seek_submission=false이며 core Native 어댑터의 Seekability::Unknown/관측 미인가 차단도 유지한다. 따라서 Auto 설정 편집은 가능해도 실제 native skip·자동 host tick·자동 검출을 지원한다고 표시하지 않는다. 모든 capability에 durable=false/detection_supported=false/automatic_host_tick_supported=false와 admission-only 의미를 둔다. inspect snapshot을 입력으로 받아 pass guard를 재구성하는 저장 우회는 없다.

## 작성한 시험과 실제 근거 한계

registry5: 반복 open의 guard 보존·원 seek1; close/new generation 무효화; busy/예산/counter/stale binding 및 상태 읽기·명시 close; unknown seek/default unavailable/owner close; expected_revision 단일 권위·중복 wire field 거절.

활성 root CLI3: 여러 fresh 프로세스가 같은 LocalHost의 open→edit→confirm→policy→inspect→skip→flag 취소→close를 공유하고 typed SDK 결과와 비교; unknown native-like fixture에서 Auto 편집과 실제 submit0·unsupported schema; owner close와 새 generation에서 이전 ID 거절. 합성 trusted/unknown 포트와 기존 LocalHost를 쓰는 시험 선언이며 실제 named-pipe/프로세스 실행은 아직 없다. `MANUAL_SKIP_EVIDENCE_DIR`가 있으면 각 호출 JSON/응답/exit와 CLI SHA를 create_new로 기록하도록 작성했다.

source17은 직접 rustfmt --check와 diff --check만0이다. 코드 17/registry5/CLI3의 컴파일·실행·지원 인수는 아직 없다. core15 통과를 host17 통과로 합산하지 않는다. 두 중앙 패치의 적용 가능성 검사는 코드가 컴파일되거나 실제 endpoint 수명을 입증한 결과가 아니다. 독립 읽기 검토→중앙 정확 합류→별도 allocated Cargo/CLI gate가 남아 있다.

스킬 적용: modoo.today requirement-to-testable-stories로 fresh process/같은 owner/host restart의 정상·실패 수명을 분리하고 실제 call0/1·오류·상태 보존 시험을 작성했다. code-change-review-verification의 source/실행 분리, immutable pin과 읽기 검토 근거를 적용했다. Rust/SRP, 중앙 자원·소유권 및 추론/download 보류를 유지했다.
