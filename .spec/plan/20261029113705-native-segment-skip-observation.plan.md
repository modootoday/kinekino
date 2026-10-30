---
id: plan:kinekino/20261029113705
title: Native OP/ED 관측 연결 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native OP/ED 관측 연결 제안

상태: active. 2026-10-20. 중앙 배정에 따른 읽기·설계 전용 후속이다. 기존 [host 연결](manual-skip-sdk-cli-contract.md)의 실제 native 관측 공백만 구체화하며 기존 계획을 대체하지 않는다.

중앙 최신 결정: SDK가 native 수신/guarded submit/default-unsupported 공개 계약의 설계·구현을 소유한다. 이 세션은 기존 core/SkipPort의 소비와 CLI capability 조립을 소유한다. 아래 SDK exact/서명/identity/보존 숫자는 인계용 제안이며 이 세션의 구현 범위나 확정 계약이 아니다. SDK 최소 서명 수신과 별도 source 배정 전 코드·실행0이다.

SDK 후보 통합 반영: 공통 계약의 현재 후보는 [native guarded 계약](native-guarded-observation-contract.md) 하나다. 아래 이전 `observe_seek`/`seek_result` 이름은 각각 `observe_native_guard`/`guarded_seek_result`로 매핑하며 중복 메서드를 만들지 않는다. 실제 최소 서명·인가 인과와 구현 scope는 여전히 PM 인수 전이다. 결과 보존은 pending+terminal **총64 slot 선예약** 후보로 정정한다(각각64의 합계128이 아니다). 명시 terminal 소비만 해제하며 미소비 eviction은 없다. 진단은 slot당8KiB, 과대 문자열은 prefix+SHA256+원길이+truncated 표시 후보다. 기존 ApiError에는 공개 source chain이 없으므로 아래의 'source chain 유지' 제안은 철회한다. 기존 공개 오류 매핑과 native 내부 bounded 원진단을 분리한다. 이 후속이 아래 이전 후보보다 우선한다.

독립 freshness 검토 반영: 현 `published_age_ms`는 게시 후 시간이며 주석/UI도 그 의미로 표시한다. 재publish 경로는 확인했지만 Unknown/false 차단이 있으므로 현행 권한 우회 결함으로 판정하지 않는다. 기존 필드 의미를 유지하고 새 native 수신 근거와 분리한다. 모든 수신 메시지 뒤 cached publish나 무관 ACK도 위치 freshness 증거가 아니다. engine/resolution identity도 freshness와 다르다. 위치는 engine playback time이며 source PTS로 승격하지 않는다.

## 2026-10-21 중앙 후속 · C 통합 뒤 최소 소비/CLI 인수표

### Outcome — 이번 결과와 목표

PM은 자막 exact8을 ea7fb5942ac4d32a71fec7cd6b38a65796a4cdbd로 채택한 뒤 이 기존 계획의 source-only 후속을 배정했다. 이번 결과는 manual skip/override/중복 방지에 필요한 입력·연결 공백·인수 조건이며 생산 코드나 실행 결과가 아니다. 아래 최신 표가 이전 source11/서명 미확정 시점의 소비 표보다 우선한다. 공통 guarded 계약 정본은 계속 [SDK 단일 계획](native-guarded-observation-contract.md)이다.

### Authority — 소유와 선행 조건

| 관심사 | 결정/구현 소유와 현재 상태 |
| --- | --- |
| 원 표본·issuer·scope·fence·64slot·read ticket | SDK 담당. live A7+B15는 인수됐지만 C는 아직 소스 후보다. 이 담당의 guarded 편집 lease는0이다. |
| 현재 단계명 | PM 후속의 A11/B6/C5 중 SDK는 B를 B7 source-only 새 배정으로 통보했다. 정확 B7 manifest/추가 경로·C5 통합/검증 여부는 SDK와 PM이 확정한다. B6 완료라고 간주하지 않는다. |
| skip 정책·attempt/revision/pass/override | 기존 application/domain을 유지한다. 소비는 별도 exact 배정 이후 기존 정책 구현을 공유하며 worker에 복제하지 않는다. |
| Manual sampled 허용·실 host capability | PM의 제품 판단과 C 통합·Rust/CLI/고정 미디어 gate가 선행한다. 소스 작성이나 public authorized 진단으로 true를 켜지 않는다. |
| CLI 결과 조회·원 ticket 보존 | 별도 additive wire/typed client/schema·호스트 수명/예산 배정이 필요하다. 기존 segments.skip DTO에 임의 필드 추가0, 독자 ticket ID·무제한 map0이다. |
| 자원·root/index | Cargo/shared·실 mpv·미디어는 새 중앙 배정 전0. 신규 추론·다운로드·worker 빌드0, stage/commit은 PM 소유다. |

### Evidence — 이번 소스 읽기와 직접 조율

2026-10-21 root ea7fb59에서 `application/skip_segments/{ports,service}.rs`, `host/{registry,contracts,schema,tests}.rs`, 기존 core tests, `domain/skip_segments/policy.rs`, `bin/player-cli/native_host.rs`, `application/playback_session/guarded.rs`를 읽었다. 현재 native host는 같은 EpisodePlaybackSessions를 주입하지만 SkipHostRegistry의 seek_submission=false이며 기존 포트는 Unknown/observation_authorized=false다. 이번 Cargo·CLI·mpv 실행은0이다. 과거 합성 host 통과 수를 이번 native 인수로 재합산하지 않는다.

SDK 직접 회신의 C A11 r2는 `runtime/coordination/sdk-guarded-ca11-r2/source11.json`, SHA256 D1C2FCE4DF09CEB29CBDC75394FE95AACA4709AE3725F83458914E9C602CF18B다. handoff와 frozen guarded API를 읽었다. 원 deadline을 최종 fence CAS 직전 현재시각으로 재검사하는 r2 수리는 재감사 중이며 실행0/production false다. source-only35 작성은 pass 수가 아니다. B7 scope 배선과 C5 parent revoke/read route는 이 후보만으로 완료되지 않는다.

| 최소 입력/출력 | 한 번만 소비할 경계 |
| --- | --- |
| host/session/binding | 기존 skip_session_id와 원 engine_instance_id/resolution_artifact_id, 같은 host 소유 SkipSession. JSON으로 scope/issuer/guard를 구성하지 않는다. |
| 수정 권한/구간 | RequestContext.expected_revision, segment_id, annotation/verified revision, 기존 OP/ED policy·PassState·Intent. stale revision/검증해제는 원 core 규칙을 따른다. |
| 관측 | 원 Observe8의 `into_guarded_parts()`로 얻은 private NativeGuardView와 비Clone guard. view의 position/duration/paused/idle/connected/error/seekability·identity를 사용한다. seconds→us 변환·freshness·권한 판단을 소비에서 다시 만들지 않는다. |
| 평가→제출 | 같은 호출에서 기존 domain evaluate 정확히1회, Eligible일 때 기존 attempted 예약/revision 증가 후 같은 guard로 Submit1. 새 barrier/legacy control fallback/last-guard 캐시0. 불필요·실패·취소면 guard 포기·seek0이다. |
| admission | 실제 enqueue의 `receipt()/into_parts()` 원 ControlReceipt와 read ticket. 접수·write commit·ACK·landing은 각각 다른 근거다. 실패 뒤 attempted/revision을 되돌려 자동 재시도를 열지 않는다. |
| 결과 | clone 가능한 read ticket으로 원 owner 결과를 조회한다. grant→command는 같은64slot 하나를 전환한다. pending consume 거절은 조회권 보존, terminal 명시 consume은 단1회, 그 뒤 NotFound다. 결과 지연이 override/pass를 변경하지 않는다. |

현재 연결 공백은 세 가지다. 첫째 `SkipPlaybackPort::observe/submit`은 guard를 전달하지 않으며 `SkipSession::skip`이 evaluate 안에서 다시 observe한다. 둘째 registry의 current_binding도 별도 observe를 한다. 기존 진단 observe를 guard 발급으로 그대로 바꾸면 중복 barrier/서로 다른 표본이 생길 수 있으므로, 한 호출의 원 view로 binding 확인과 기존 평가를 공유할 최소 연결을 SDK/PM이 별도 배정해야 한다. 셋째 현재 ActionResult는 기존 SkipResult만 반환하므로 지연된 원 ticket 결과의 typed CLI 조회와 보존 소유권이 아직 없다. 이번 문서가 새 trait/메서드/DTO를 확정하지 않는다.

### Unverified — 미결정과 닫힌 지원

B7 정확 핀·C5 scope/revoke/read 전달·C 전체 실제 gate, 같은 호출 소비 연결의 exact 경로와 lock 범위, 결과 wire의 공개 방식/host read ticket 예산은 아직 미확정이다. SDK worker는 barrier 응답을 기다릴 수 있으나 UI callback/IPC worker/Active·결과 registry lock을 잡고 기다리지 않아야 한다. 기존 skip 상태 소유 잠금과 외부 대기의 수명도 소비 설계 검토에 포함한다. 외부 mpv 전환까지 atomic compare-and-seek나 exact landing을 보장하지 않는다. Auto tick/detect/저장 복원과 실제 CLI 취소·OS 호출의 엄격한 wall-clock 한계는 이 후속으로 지원되지 않는다.

2026-10-21 C5 계획 직접 조율에서 SDK에 다음 최소 요구를 회신했다. C5는 parent_binding/tests·parent_router/mod/tests·deferred_engine의 정확5 계획이며 이 회신은 구현 lease나 공개 wire 확정이 아니다.

- caller 기대 identity는 저장된 SkipBinding의 engine_instance_id와 resolution_artifact_id 둘이다. 이를 원 private view.identity와 일치시킨다. replacement의 expected_instance/expected_artifact도 같은 원 binding 의미지만 현재 guarded Observe에 caller expected 인자가 있다고 가정하지 않는다. scope/parent epoch/generation/nonce/deadline/sample revision은 issuer 내부 검증이며 소비가 재구성하지 않는다.
- 관측 전 stale config는 기존 revision 검사로 거절하고, 관측 뒤 교체/revoke는 같은 guard Submit의 원 SDK 검증에 맡긴다. 별도 current read나 재평가로 원 표본을 갈아 끼우지 않는다. 같은 view로 binding 확인과 기존 평가1회를 공유할 소비 exact는 별도 배정한다.
- 원 결과 조회는 current active 기대값으로 원 ticket을 차단하거나 새 engine에 재귀속하지 않는다. 원 issuer/host read scope를 검증하고 host 소멸·foreign issuer의 원 오류를 유지한다. revoke 뒤 read 허용은 제어 권한 복원과 구별한다.
- CLI 미결정에는 bounded host ticket owner와 wire 참조 방식, parent revoke/host close 뒤 read 수명, full/consume·정리 규칙, admission 성공 뒤 wire 응답 실패로 ticket이 전달되지 않았을 때의 미소비 결과 보존/회수 정책이 포함된다. 새 ID/map/명령명/DTO를 이 C5에 추가하지 않으며 총64 예산을 임의 확장하거나 terminal을 묵시 퇴출하지 않는다.

SDK 후속 회신: 위 소비 요구는 `runtime/coordination/sdk-guarded-cb7-r2/c5-binding-impact-plan.md`에 반영됐다. B7 r2 source10 핀은144C891A4A1743574EA2FA027DB07DE6E401AAE464C141E77F15429C5B466242이며 foreign-scope 수리의 PM 독립감사 인수/실행0 상태다. C5는 계획뿐이며 production false를 유지한다. A11 단독 heavy의 첫 실행34pass/fixture fail1 후 원 scope 소유 fixture·unused import 수리 r3 재실행 중이라는 SDK 통보는 중간 상태로만 기록한다. 이 담당이 해당 실행을 수행하거나 최종 통과를 확인한 것은 아니며 B7/C5 통합 성공으로 합산하지 않는다.

#### root4b58f69 후속 · lazy per-call 최소 후보 검토

2026-10-21 PM의 읽기 연구 후보를 기존 service/ports/registry와 SDK forward API에 직접 대조했다. 다음 **생산3+시험2 후보**는 기존 service.rs·ports.rs 정책을 수정하지 않고 소비할 수 있는 형태다. 구현 lease·실행·CLI wire·지원 flag 변경은0이며 helper4 동결/gate와 분리한다.

| 정확 후보 | 책임 |
| --- | --- |
| 신규 application/skip_segments/native_port.rs | private NativeSkipCall이 첫 observe에서 원 private view/비Clone guard를 한 호출 수명에만 보관하고 기존 SkipObservation으로 투영한다. submit은 같은 guard를 take1해 기존 Sessions::submit_guarded_seek를 호출하며 실제 반환 submission을 보존한다. last-guard 공유 캐시/새barrier/legacy control fallback0 |
| application/skip_segments/mod.rs | private adapter 및 별도 시험 등록만 수행 |
| application/skip_segments/host/registry.rs | typed guarded 진입의 with_session(...false)로 앞 diagnostic current_binding 관측을 생략하고 기존 session.skip을 호출. 원 Result와 Option<GuardedSeekSubmission>·갱신 snapshot을 함께 보유하는 private outcome을 반환 |
| 신규 application/skip_segments/native_port_tests.rs | call-local 표본·full binding·guard 1회·원 submission과 오류 보존 회귀 후보 |
| application/skip_segments/host/tests.rs | stale/cancel observe0·동일call 평가1·예약/closing/중복·후처리 오류의 원 outcome 회귀 후보 |

소스 정합 근거: 기존 SkipSession::skip은 next/index/cancel을 관측 전에 처리한다. evaluate는 받은 observation의 engine+artifact를 저장 binding과 대조하고 domain evaluate를1회 호출한다. Eligible일 때 attempted/revision을 submit 전에 예약한다. 따라서 lazy adapter는 이 순서와 기존 cancel/override/명시 Manual 재시도를 재사용할 수 있다. `with_session(false)`는 current_required 관측만 생략하며 저장 session/binding 검사와 기존 try_lock 소유를 유지해야 한다.

반영 전 필요한 조건은 다음과 같다.

- registry의 closing과 seek_submission=false 차단을 우회하지 않는다. call-local submit에서도 기존 BoundPort.submit과 같이 같은 closing 신호를 검사한다. SDK 원 guard 검증과 별개로 기존 host admission을 유지한다.
- adapter는 호출 마지막에 소유 소비해 원 core Result·실제 Ok로 받은 submission·snapshot을 추출한다. 후처리 오류에 `?`로 submission을 버리지 않는다. SDK가 Err로 반환한 숨은 admission의 ticket까지 adapter가 복구할 수 있다고 주장하지 않는다.
- trusted host가 주입한 **같은 Arc<EpisodePlaybackSessions>**를 registry 내부 private carrier에 조립 시 결속한다는 방향은 PM이 인수했다. 기존 public new(generation, port, maximum_sessions, seek_submission)는 signature/legacy false 동작을 유지하고 native carrier=None으로 둔다. 별도 typed Rust 생성자는 원 Sessions Arc 하나에서 기존 port와 private carrier를 함께 만들며 매 call의 raw Sessions 인자를 받지 않는다. binary 소비를 위한 public 진입과 private 내부 carrier 구분은 아래 SDK 후속 회신을 따른다. typed 호출은 registry의 carrier만 소비하며 engine 숫자/artifact 일치나 JSON으로 같은 host/issuer 권한을 만들지 않는다. 실제 API 서명과 호스트 배선 exact는 별도 배정 전이다.
- skip 상태 try_lock을 call 동안 유지하는 안은 기존 직렬 session 소유와 맞지만, barrier/submit의 대기 중 Active·결과 registry·router 잠금 보유0과 역 callback0을 소스 감사로 확인해야 한다. 이를 피하려고 잠금을 풀고 재평가하거나 revision을 복구하는 새 정책을 만들지 않는다.
- read ticket wire·bounded host 보존/consume·응답 유실 정책은 여전히 별도 미결정이며 이 private outcome이 공개 CLI 완료를 뜻하지 않는다. 실제 host의 enabled scope·capability와 기존 Unknown 포트의3차단도 별도 소비 배정 전 유지한다.

조립 경로 읽기와 SDK 후속 회신: 현재 `src/bin/player-cli/native_host.rs`의 원 Sessions Arc 생성 지점은81행, playback_host.rs의 host.sessions()를 Arc로 감싸는 지점은88행이다(root4b58f69 기준). 이 Arc를 새 typed native-owner 생성자에 한 번 clone 전달하고 같은 Arc를 기존 with_shared_episode_playback에 전달한다. 매 call Sessions 입력이나 별도 Sessions factory는 필요하지 않다.

CLI binary와 application library는 별도 crate이므로 새 생성자는 public Rust 진입점이 필요하되, carrier 필드/생성 내부와 호출 중 owner 선택은 private 또는 pub(crate)로 유지한다는 SDK 제안으로 정정한다. 앞서 library 내부 새 factory가 반드시 필요하다고 본 것은 과도한 제약이었다. typed 생성자는 원 Arc를 결속할 뿐 scope/guard 권한을 발급하거나 JSON/API schema·지원 flag를 열지 않는다. 실제 권한은 SDK의 private enabled scope/원 guard 검증에 남는다. 기존 public new/false와 Host/Engine::new disabled를 보존한다.

SDK의 standalone scope/factory 정확7 후보는 scope.rs/session_engine.rs/mpvhost.rs/workspace.rs·두 CLI assembly·기존 lifecycle tests이며, registry/service 소비와 분리한다는 회신을 받았다. Trusted SHA 검증 성공과 별도 명시 parentNone 진입만 enabled scope를 허용하고 같은 host.engine clone은 원 B7 scope를 공유하는 설계다. 아직 구현/지원 인수가 아니다. 동일 Arc pointer identity와 host drop 뒤 권한 폐쇄 회귀를 포함하며 정확 manifest/새 생성자 서명·소유 경계는 PM이 별도 배정한다. 현재 helper4나 실행 범위에 합류하지 않는다.

PM 추가 읽기 연구 통보에 따르면 현재 standalone native queue는 try_send 성공 뒤 row 삽입/receipt+ticket Ok까지 추가 fallible 단계가 없고 Full/Closed는 미접수·row0/원 grant 정리다. C8 generic decorator는 engine Ok 뒤 bind_read_context 실패 가능성이 있으나 실제 native producer는 fresh cell을 반환하므로 현 standalone에서 그 반례가 발생했다는 근거는 없다. receipt만으로 lost ticket을 복원하는 공식 API도 없다. 이 통보는 소비의 보존 범위를 **실제로 Ok로 받은 원 submission**까지로 한정하는 근거이며 모든 Err=admission0이라는 일반 보장으로 확장하지 않는다. helper gate 소스/시험에는 혼합하지 않았다.

#### 생산3+시험2의 구체 계약 후보 · 구현 전

helper18은 별도778c483 채택으로 마감했고 현재 root17d15767에서 이 소비 계약만 정리한다. source 편집·SDK factory7 합류·CLI wire·지원 flag·Cargo는0이다. 정확 후보5는 위 표의 native_port.rs/mod.rs/host/registry.rs/native_port_tests.rs/host/tests.rs를 유지한다.

생성자는 public Rust 조립 진입으로 generation·같은 Sessions Arc·기존 session budget을 한 번 받는 형태를 우선한다. private carrier는 registry.rs 내부에서만 생성하고 하나의 Arc에서 legacy port와 native 소유 참조를 함께 만든다. 기존 public new와 native carrier=None/false 경로는 그대로다. 새 생성자의 사용 자체로 지원을 true로 켜지 않으며, 실제 capability 개방은 후속 제품 인수 사항이다. 정확 이름과 enabled 경로의 내부 호출 권한은 구현 배정에서 고정한다. 두 CLI 조립 경로는 SDK factory7 소유이므로 이 후보5에서 수정하지 않는다.

| 경계 | 확정할 최소 동작 |
| --- | --- |
| native_port의 call-local 상태 | 원 Sessions 참조·원 closing 신호·첫 private view와 move-only guard·실제 Ok submission만 소유. 공유 Arc 내부에 last guard를 보관하지 않음. 기존 &self port에 필요한 내부 가변성도 이 호출 하나에 제한 |
| observe | 첫 호출만 원 observe_native_guard→into_guarded_parts. 같은 view의 full identity/playback/seekability/authorized로 SkipObservation 구성. 추가 observe가 필요해도 새 barrier 발급이나 진단 값 혼합0. core 평가는 기존 skip이1회 수행 |
| submit | 같은 closing 검사·원 view/engine 정합 확인 뒤 기존 Seek target을 원 guard take1로 전달. 일반 control·새 guard 발급0. 반환 Ok의 원 receipt를 core에 빌려 주고 원 submission은 call에 보존. Err는 원대로 반환하며 attempted/revision을 복구하지 않음 |
| registry 진입 전 | 저장 session id/binding·closing/budget/capability·expected revision의 기존 권한을 유지. carrier 없는 legacy를 native 지원으로 전환0. raw Sessions 인자를 매 호출 받지 않음 |
| registry call | with_session(...false)의 앞 diagnostic observe만 생략하고 같은 state try_lock에서 기존 session.skip 사용. native view의 full binding 검사는 기존 evaluate로 수행. 취소/stale는 관측 전에 원 코드가 거절 |
| private outcome | `result: Result<SkipResult,ApiError>`·`submission: Option<GuardedSeekSubmission>`·`snapshot: SkipSnapshot`를 함께 소유하는 형태. closure에서 core result에 ?를 쓰지 않고 call을 소유 소비해 outcome 구성. 외부 Result의 사전 실패에는 submission이 없고, call 이후 오류는 outcome 안에 원 submission/갱신 revision과 함께 남음 |
| 후속 결과 | 기존 read ticket API를 재사용하며 별도 query wire/map/id는 추가하지 않음. admission/원 결과/landing 구분과 원 host 예산·consume 수명은 다음 배정 전 닫힌 상태 |

SDK barrier를 기다리는 동안 보유할 수 있는 잠금은 기존 skip 상태의 직렬 소유뿐이라는 조건을 소스 감사에 건다. Active/결과 registry/router 잠금이 대기 경로에 남거나 SDK에서 skip으로 역호출하는 경로가 있으면 이 후보를 완료로 인수하지 않는다. try_lock을 풀고 재평가/재예약하는 대안은 이번 최소안에 넣지 않는다.

회귀 후보는 adapter의 관측 RPC1/같은guard submit1·다른 binding 거부/guard 재사용0·closing 뒤 submit0·submit Err의 원 오류, registry의 stale/cancel observe0·기존 attempted/revision·override/backward·same-session 재open·core 후처리 오류에도 실제 수신 submission 보존·legacy constructor/capability false를 포함한다. trusted guard fixture는 SDK의 기존 crate 내부 시험 seam을 재사용하고 public authority constructor를 만들지 않는다. pointer identity와 실제 host drop 권한 폐쇄는 SDK factory7 회귀와 소비 인수를 연결한다. 컴파일에서 사용하지 않는 private 중간 API나 추가 등록 파일이 필요하면 정확 경로를 먼저 PM에 보고하며 넓은 lint 허용이나 구현 범위 확장을 하지 않는다.

#### exact5 구현 배정용 짧은 동결 계약

최신 구현 후보: PM 정식 배정 뒤 root17d15767 기반 source5 **ED3A961B41B50B2A0A77BC6CF70D626574E18560A6C4F59AFDFD0479EB75C692**로 동결했다. 실제 Rust 소비를 위해 typed entry/owner-bearing outcome을 정확5 내 public Rust API로 정하고 mod.rs에서 최소 재수출했다. 아래 crate-private 형태는 구현 전 후보 이력이며 현재 공개 형상은 runtime/coordination/native-skip-consumer-r1/handoff.md를 따른다. private carrier/생산 false·기존 wire는 유지하며 lint allow/expect0이다. 신규10 작성/direct fmt0/Cargo·실행0/실제pass0이며 독립 감사 요청 상태다.

최신 중앙 인수/실행 계획: source5의 실제 길이/hash·before/보호7은 독립 감사에서 일치했고 새 P1/P2 없음으로 인수됐다. 후속은 SDK factory7과 비중복 union12 단일 gate이며 실행 owner는 SDK다. UI C21 명시 반환이 선행하고 별도 skip heavy를 중복 실행하지 않는다. 정확 입력/필터 예정26·SDK 영향7의 dedup·소스 수리 소유·ticket/receipt/PID 한계는 runtime/coordination/native-skip-consumer-r1/union12-input-handoff.md를 따른다. 실제 실행은 아직0이다.

2026-10-21 PM 요청에 따른 **서명 후보 동결**이며 코드 구현/지원 인수가 아니다. 경로는 정확히 다음5다: 신규 `src/application/skip_segments/native_port.rs`, 기존 `mod.rs`, 기존 `host/registry.rs`, 신규 `native_port_tests.rs`, 기존 `host/tests.rs`. service.rs·ports.rs·host/mod.rs·dispatch/schema·CLI bin2·SDK factory7·Cargo는 제외한다.

| 위치 | 동결할 호출 형태/반환 |
| --- | --- |
| registry public Rust 조립 | `SkipHostRegistry::new_native_owner(generation: String, sessions: Arc<EpisodePlaybackSessions>, maximum_sessions: usize) -> Result<Self, ApiError>` |
| 원 owner/port | 생성자는 원 Arc 하나에서 legacy port와 private native carrier를 함께 결속하며 `seek_submission=false`로 둔다. 새 thread/owner registry를 만들지 않는다. caller는 기존 `Arc::new(registry)`→`registry.owner() -> SkipHostOwner`와 기존 SkipHostApi coercion을 그대로 사용한다. 기존 new와 owner()/begin_close 의미·signature 불변 |
| private typed 진입 | `pub(crate) fn skip_native_call(&self, input: SessionRequest<SkipInput>, revision: DecimalU64) -> Result<NativeSkipOutcome, ApiError>`; raw Sessions 인자0 |
| call-local adapter | `NativeSkipCall::new(sessions: &EpisodePlaybackSessions, closing: &AtomicBool) -> Self`; 기존 SkipPlaybackPort 구현의 observe/submit을 통해서만 기존 core가 사용 |
| call 소비 | `NativeSkipCall::into_submission(self) -> Option<GuardedSeekSubmission>`; 실제 SDK Ok 수신값을 그대로 이동하고 ticket을 재구성하지 않음 |
| crate 내부 outcome | `NativeSkipOutcome { result: Result<SkipResult,ApiError>, submission: Option<GuardedSeekSubmission>, snapshot: SkipSnapshot }`; crate 내부 가시성만, serde/공개 wire/guard Clone 추가0 |

`skip_native_call`은 carrier=None이면 Unsupported다. 같은 registry의 with_session(...false)에서 call을 생성해 기존 `session.skip`의 **Result를 ? 없이 저장**하고, call을 소유 소비한 submission과 session.inspect를 outcome으로 묶는다. outer Err는 session/carrier/lock 같은 접수 전 실패이며, core 호출 뒤 오류는 원 outcome 안에 남긴다. state try_lock은 호출 내내 기존처럼 유지한다. 원 closing은 시작과 adapter submit 직전에 검사하고 Active/result/router 잠금으로 대기를 넘기거나 SDK 역 callback을 허용하지 않는다.

이 private typed 진입은 기존 public `skip`/SkipHostApi dispatch에 연결하지 않는다. 기존 경로의 seek_submission=false 거절은 그대로이며 새 constructor로 capability true를 광고하지 않는다. typed 진입의 신뢰 근거는 조립 시 고정한 carrier와 실제 SDK 원 guard이고, 호출자가 bool/숫자/JSON으로 새 scope를 발급하는 경로가 아니다. 따라서 SDK factory bin2 배선과 공개 CLI 지원은 여전히 다음 배정이다. private 중간 진입이 production에서 아직 미사용인 단계의 lint 처리는 넓은 allow 없이 해당 진입만의 근거 있는 non-test expect 필요 여부를 PM 감사에서 정하며, 공개 연결 시 제거한다.

필수 회귀: (1) 같은 Arc pointer가 port/carrier에 결속·legacy new/carrierNone/false 및 public dispatch 차단 보존, (2) stale revision·사전 cancel은 observe0/submit0, (3) fullbinding 일치의 first observe1/domain evaluate1/sameguard submit1·추가 barrier0, (4) foreign binding/Unknown·guard 부재는 submit0, (5) 관측 뒤 closing/revoke/만료는 원 오류/submit0 또는 SDK 원 disposition이며 자동 재시도0, (6) 실제 Ok submission 뒤 core receipt mismatch 같은 후처리 Err에서도 outcome의 원 submission과 갱신 snapshot 보존, (7) submit Err의 attempted/revision 보존·Automatic 중복0/명시 Manual 정책 재사용, (8) override/backward/동일 session 재open으로 pass 리셋0, (9) SDK await 동안 Active/result/router lock 보유0·역호출0의 소스 인과 확인. 기존 SDK crate 내부 guard 시험 seam을 재사용하고 public authority·전역 ticket map은 추가하지 않는다. 실제 OS/CLI wire/landing 인수는0이다.

#### 후속 CLI wire/원 ticket 보존 경계 · 읽기만

공동 union12 실행과 분리한 후속 계약이다. 현재 dispatch는 기존 segments.skip→registry.skip만 호출하며 RequestContext의 idempotency_key/timeout_ms를 Unsupported로 거절한다. typed client도 기존 ActionResult만 decode한다. source5의 NativeSkipOutcome은 serde가 없고 현재 wire로 내보내지 않는다. 이 읽기로 CLI 지원을 true로 켜지 않는다.

직접 읽은 원 SDK 사실: NativeSeekTicket은 immutable admission Arc와 별도 read_context OnceLock을 공유하며 공개 getter는 identity/same_request뿐이다. 안정적인 wire id나 serialize/공식 id→원 ticket 복원 API는 없다. mpv 결과 Ticket은 private id와 Weak host/Rows·원 identity를 갖고 Registry::read는 같은 owner/id/identity·pending consume 거절·terminal 명시 소비를 검사한다. Debug의 id 표시는 프로토콜이 아니며 파싱하지 않는다. 원 receipt·engine 숫자·native row id만으로 decorated ticket의 read context를 재구성해서는 안 된다.

| wire 전 반드시 닫을 조건 | 책임/제한 |
| --- | --- |
| 공식 참조와 resolver | SDK가 원 decorated NativeSeekTicket을 보존하는 bounded read-reference owner/형식·검증을 먼저 확정한다. CLI 담당이 독자 id/map/serde ticket을 만들지 않는다. 원 scope/owner를 확인하며 current active engine으로 remap0 |
| 원64 예산과 저장 크기 | 원 Observe grant→command 슬롯 전환과1:1 대응을 유지한다. 추가 metadata/키/원 read context/오류 저장 크기를 함께 산정한다. native queue와 무관한 새64 작업 슬롯·무제한 duplicate 목록을 만들지 않는다. 정확 보존 자료구조는 미배정 |
| 접수 뒤 응답 유실 | 전달되지 않은 원 ticket도 보존하고 조회 가능한 공식 recovery 근거를 SDK/PM이 결정해야 한다. request_id는 현재 상관 값이며 idempotency key/고유 요청 저장소로 승격하지 않는다. transport 오류 뒤 자동 resubmit0, inspect로 attempted/revision만 확인한 것을 ticket 복구로 표시0 |
| read/consume 수명 | pending 조회는 원 권리를 유지하고 consume=true 거절이 row를 지우지 않는다. terminal 소비는 단1회·이후NotFound, 원 revoke/교체 뒤 read scope는 유지하되 host 소멸은 원 SourceExpired. session close가 미소비 ticket을 자동 consume/evict해도 된다고 가정하지 않음 |
| 기존 wire 호환 | 기존 segments.skip의 ActionResult/오류 의미와 context 검증은 유지한다. 별도 additive 명령/typed DTO·client/schema/capability가 원 Rust 계약과 함께 배정돼야 하며 결과 조회 성공과 command/ACK/landing을 구분 |

후속 영향 후보는 SDK 소유 application/playback_session/guarded.rs와 원 결과 owner/resolver 책임, 제품 소비의 skip_segments/host/{contracts,registry,dispatch,schema}.rs·application/api/client/skip_segments.rs 및 capability 조립이다. 정확 신규 경로·명령명·public read-ref shape·추가 저장 예산은 아직 확정하지 않았다. source5/SDK7 작성 범위를 확대하거나 지금 이 파일들을 수정하지 않는다. 이 선행 조건 없이 public Rust outcome을 JSON Value로 바꾸는 우회는 하지 않는다.

필수 후속 인수는 동일 endpoint의 fresh CLI에서 원 참조→pending 재조회→terminal1회 소비·그 후NotFound, foreign host/issuer 거절, 원 revoke/교체 후read·host 소멸, full budget/미소비 결과 보존, 응답 전송 실패의 recovery, 원 receipt/error·write commit/ACK/별도 landing 구분이다. 지금은 작성·실행0이며 기존26/SDK125 공동 gate 통과 수에 포함하지 않는다.

#### 원64 row 내 export record 권고 · SDK 선행 계약

SDK가 구체화한 [생산자 최소 계약](native-seek-row-export-producer.md)을 후속 source-only 제안으로 연결한다. 직접 검토 후 SDK에 read_context=None의 standalone/미장식 routed 구분, 사전 결속 시 기존 C8의 두 번째 set 방지와 generic1회 규칙 보존, native issuer 권한과 skip provenance 세대의 타입/검증 분리 및 관련 회귀를 회신했다. 이 항목들은 현재 구현/시험 결과가 아니라 생산자 계획의 확정 전 조건이다.

2026-10-21 PM 권고를 반영한 후속 설계다. 구현0이며 현재 union12 pin/실행에는 합류하지 않는다. 새로운 application ticket map/독자 command ID/추가 작업 슬롯 대신, SDK 원64 row 안의 optional export record가 원 read capability·host generation·skip provenance를 보존하고 같은 row의 bounded wire reference를 제공하는 방향으로 좁힌다. guard는 직렬화하지 않는다.

원 소스의 구체 공백을 확인했다. Registry::admit_observation은 grant key.id를 그대로 command Ticket.id와 Snapshot key로 전환한다. try_send 실패면 원 grant를 지우고 포기하며, 성공 뒤 Queued row를 만든다. 그러나 application NativeSeekTicket의 admission Arc/read_context OnceLock은 그 이후 issuer의 ticket.submission(receipt)에서 생성된다. 따라서 지금 wrapper에서 단순히 export 값을 저장하는 방식은 admission 전 예약/원 read-context 보존을 충족하지 못한다. SDK가 이 생성·저장 순서를 소유해 바꿔야 한다.

특히 같은 private Ticket에 submission(receipt)을 두 번 호출해 row용과 caller용 NativeSeekTicket을 만들면 서로 다른 admission Arc/OnceLock이 생긴다. export는 **원 NativeSeekTicket을 한 번만 생성한 뒤 그 공유 read cell을 보존**해야 한다. C8 bind_read_context가 장식하는 동일 OnceLock을 공유해 원 parent/router read 권리를 잃지 않아야 한다. 원 native Ticket의 host/Rows와 C8 ReadOwner의 binding/router는 Weak임을 읽었으며, export 추가 시 이를 강한 process/host 소유나 순환 참조로 바꾸지 않는 인수가 필요하다.

| SDK 선행 API 책임(이름/서명 미정) | 요구 동작/오류 경계 |
| --- | --- |
| grant의 export 준비 예약 | trusted skip host가 검증한 host generation·원 skip_session/pass·segment/annotation revision·요청 전 revision provenance를 원 guard/grant에 결속. 크기/형식·scope/issuer/metadata 저장 여유를 enqueue 전에 검사. 현재 wire JSON이나 public authorized bool로 issuer를 만들지 않음 |
| admission과 같은 row 전환 | 원64 grant→command의 같은 id/slot을 유지하고 export 준비도 같은 원 소유 트랜잭션에 포함. 접수 뒤 별도 저장소 insert 실패로 ticket이 사라지는 정상 오류 경로를 만들지 않음. full/closed/검증 실패는 원 SDK의 미접수/정리 규칙을 보존 |
| 원 read cell 보존/장식 | caller가 실제 Ok로 받은 원 ticket과 export record가 같은 admission/read-context cell을 공유. generic decorator 실패는 원 오류와 row 상태를 보존하고 완료로 바꾸지 않음. 모든 SDK Err=미접수라는 일반화는 금지 |
| export된 원 row 조회/열거 | export된 row만 원 host/issuer/provenance로 bounded 열거·조회. 원 skip의 현재 revision/current route가 달라져도 원 결과 조회를 잘못 차단/재귀속하지 않음. native 내부 Debug.id를 wire key로 해석하지 않음 |
| 명시 consume | pending consume은 NeedsInput/원 row·조회권 불변. terminal consume은 원 row를 단1회 해제·그 뒤 NotFound. consume 응답 유실은 at-most-once 해제이며 무제한 tombstone/자동 재submit으로 보상하지 않음 |

provenance는 caller 문자열을 그대로 권한으로 신뢰하지 않는다. skip registry가 자신의 generation/session/pass/현재 구간과 검증 revision을 기존 규칙으로 확인한 값만 준비한다. 원 expected_revision은 기록된 요청의 세대이며 후속 결과 read에 새 현재 revision 일치를 요구하는 근거가 아니다. RequestContext.request_id는 상관 값이며 native row id나 복원 수단이 아니다. wire reference는 read-only이고 guard/submit 권한을 포함하지 않아야 한다.

메모리 인수는 원 슬롯 수64를 유지하는 것과 별개로 필수다. export record의 고정 Rust 크기·bounded id/provenance strings·read cell/context/Arc·실패 진단·JSON escaping과 목록 응답을 함께 산정한다. 현재 row payload8KiB/diagnostic prefix·display1KiB 규격 위에 export를 무상 추가했다고 주장하지 않는다. 원 총 payload 경계 안에서 prefix/metadata 몫을 명시하거나, 다른 총 byte 예산이 필요하면 SDK/PM이 별도 승인·검증한다. 원 오류 hash/original_bytes/truncated는 보존한다. 읽기 열거는 최대64행의 참조/요약만 반환하는 안이며 모든 오류 본문을 한 번에 복제하지 않는다. 정확 per-record byte cap과 목록 출력 cap은 아직 source size/실제 직렬화로 검증하지 않았다.

출력/실패 순서는 다음과 같이 구분한다. 접수 전에는 typed payload·provenance·export reserve·출력 형상/크기의 필요한 검사를 끝낸다. 접수 뒤 core 후처리 오류와 response encoding/transport 쓰기 실패가 생겨도 원 exported row를 보존한다. submit 응답을 잃은 fresh CLI는 원 host/provenance의 exported row 열거로 찾으며 재submit하지 않는다. 열거·peek 응답 쓰기 실패는 row를 소비하지 않는다. terminal consume 성공 후 응답 쓰기 실패는 row가 이미 사라질 수 있고 이후 NotFound를 '사용자가 결과를 받았다'는 ACK로 표시하지 않는다. exactly-once delivery를 보장하지 않는다.

EOF는 원 admission close→기존 in-flight/worker/child 정리→scope 만료 순서를 보존한다. native ticket이 child owner를 강하게 연장하지 않는다. host restart/foreign host reference는 거절하고 자동 복원하지 않는다. cleanup 실패/Incomplete는 원 소유 상태로 보고하며 Drop·transport 종료를 결과 소비 ACK로 만들지 않는다. 동기 OS 수명의 엄격한 wall-clock 한계는 여전히 별도다.

후속 fresh CLI 인수표는 (1) 원 exported reference/receipt·peek와 native 원 read cell의 same_request 인과, (2) submit 응답 실패 뒤 같은 host/provenance 목록으로 복구·seek 추가0, (3) full64에서 추가 admission0·명시 terminal consume만 예산 해제, (4) pending consume NeedsInput 후 재조회/terminal consume 경쟁1회·이후NotFound, (5) consume 응답 실패 뒤 at-most-once 해제·자동 재submit0, (6) override/구간 revision 변경·교체/revoke 뒤 원 결과 조회, (7) EOF·원 scope 만료/재시작·foreign 거절, (8) export metadata/오류 prefix/목록/escaping/출력 쓰기 실패의 byte bound·원 오류 보존이다. 먼저 SDK 생산자 Rust 회귀로 준비/접수/보존을 인수한 뒤 제품 host contracts/registry/dispatch/schema·typed client·bin 조립·fresh CLI를 별도 배정한다. 지금 시험 작성/실행은0이다.

### Phases — 배정 후 SDK/CLI 최소 인수표

아래는 새로 작성/실행한 시험이 아닌 인수 조건이다. 모든 CLI 단계는 같은 명시 host endpoint와 고정된 fresh CLI artifact·원 JSON/exit·snapshot revision/pass·request별 결과를 보존한다. 합성 제어 포트 시험과 실제 mpv 시험은 별도 열거한다.

| ID/경우 | SDK/원 core 조건 | CLI 인수 근거 |
| --- | --- | --- |
| S1 준비/기본 차단 | open→edit→accept_skip→Manual policy. unknown/미인가/false/unsupported는 submit0 | 서로 다른 fresh CLI가 같은 session 상태를 공유. capability/schema/실오류 일치, 준비만으로 seek 지원 true0 |
| S2 명시 Manual | 같은 Observe8/view로 평가1, 원guard Submit1, 기존 target=out_us·EnginePlayback 시간축 | segments.skip 원 접수 receipt와 별도 원 결과 조회. binding/구간/원 target·ticket 상관, 별도 위치 관측. landed/품질 승인으로 표시0 |
| S3 stale/변경 | stale config·annotation·foreign binding/guard/revoke/deadline/표본 revision 변경은 새 seek0 | 원 ApiError와 같은 session inspect로 revision 확인. 자동 재open/retry0; SDK seam과 실제 변경 사례를 구분 |
| S4 중복/불명확 실패 | attempted 예약은 admission 전에 유지. 접수·거절·ACK 지연/실패 후 자동 재시도0 | 같은 session reopen은 reused이고 pass/attempt를 보존. Automatic 반복은 원 AlreadyAttempted 차단. 명시 Manual 재시도는 최신 revision으로 원 정책만 소비 |
| S5 override/backward | 신뢰된 backward 관측 또는 ManualSeek override 뒤 Automatic 억제. policy 변경/늦은 ACK/위치/오류로 억제 해제0 | override→evaluate의 Overridden과 seek0. 명시 Manual 동작은 기존 규칙대로 별도 검사. BeginNewPass만 pass 증가/새 guard 필요 |
| S6 취소 | 사전 cancelled/stale revision은 원 cancel 정책; 미시도 BeforeAdmission, 시도 후 TooLateOrUnobserved | cancelled 입력과 override Cancel의 동일 상태 의미. API 취소를 이미 쓴 seek 철회나 성공 landing으로 표시0 |
| S7 결과/예산 | Observe grant와 command 총64, 이전grant→같은slot transfer. pending clone 재조회/consume 거절·terminal 1회 소비 | 새 wire가 인수된 뒤만 실제 CLI 조회. full은 write0/ResourceExhausted, 미소비 terminal eviction0. 성공 조회를 성공 seek로 세지 않음 |
| S8 교체/종료 | old guard 새 제출0, 원 ticket은 원 owner read scope로만 조회. host 소멸/foreign route는 원 계약 오류 | 다른 endpoint/재시작에서 session·ticket 복원0. 현 active로 remap0, EOF/원 owner 정리 근거와 결과보존 경계를 분리 |

첫 단계 완료 조건은 SDK의 B7/C5 정확 핀·서명/수명 회신과 PM의 소비/wire exact 배정이다. 다음 단계는 Rust seam 회귀→fresh CLI 회귀→중앙이 고정 배정한 기존 mpv/미디어 검증 순서다. 이전 합성 core/host 시험을 변경 영향 확인에 재사용하되 native8RPC·원guard·실제 seek 증거를 대체하지 않는다. 시험 작성/실행·새 코드 적용은 그 배정 전까지0이다.

### Rollback — 문서와 미인수 연결

이번 변경은 이 기존 계획의 최신 절만 수정한다. 코드·지원 bool·공유 snapshot·root index는 바꾸지 않는다. 후속 소비가 실패하면 원 pin/로그를 남기고 Unknown/false/Unsupported를 유지한다. 이미 실제 실행한 seek는 문서/설정 되돌림으로 취소할 수 없으므로 향후 실험은 소유 세션과 결과를 따로 보존한다.

스킬: modoo.today `modootoday/spec-authoring/plan`과 참조 지시의 `modootoday/spec-authoring/plan-authoring` 본문을 읽고 기존 계획 재사용·Authority/Evidence/Unverified/완료조건/복구 분리를 적용했다. 이번 semantic 검색의 runtime delegation/CI scaffold는 무관하여 채택하지 않았다. 새 비 Rust 정책/검증 도구는 없다.

## 중앙 인계 — core 소비에 필요한 최소 요구

### 2026-10-21 SDK 직접 조율

후속 합의: SDK 단일 정본 상단의 2단계 결정안을 직접 읽고 소비4항의 **설계 형태**에 동의한다고 회신했다. 같은 barrier의 `into_guarded_parts()`, 실제 enqueue 원 receipt의 `receipt()/into_parts()`, guard는 noClone/read ticket만 Clone, typed result getter를 소비한다. 경쟁 API 추가 요구는 없다. terminal 소비 경쟁은 한 소비만 성공하고 나머지는 NotFound, pending 소비 거절은 원 slot/조회 권리를 보존하는 회귀를 따른다. A7/B13/C11은 SDK의 단계별 후보이며 아직 이 세션 소비 구현 배정이 아니다. Manual-only sampled strength와 외부 mpv 전환의 비원자성은 제품 허용 범위의 PM 결정 사항이다. 설계 합의와 실제 코드·CLI·미디어 인수는 구별하며 현 권한false를 유지한다. 앞 '아직 합의 완료로 표시하지 않는다'는 전달 당시 이력이며 이번 합의는 위 네 소비 형태에 한한다.

PM 배정에 따라 SDK 담당에게 같은 thread 메시지로 최소 소비 요구를 전달했다. 단일 공통 정본의 갱신은 SDK가 소유하며, 합의 완료나 구현 가능으로 아직 표시하지 않는다.

- 공용 관측 투영과 move-only guard를 같은 호출에서 소비한다. skip 쪽 seconds→us 정책·관측 identity·last-guard 캐시를 새로 만들지 않는다.
- 기존 attempted 예약/revision 증가 뒤 받는 ControlReceipt는 실제 queue admission만 나타내야 한다. SDK의 공식 대응 없이 ticket으로 receipt를 생성하지 않는다. 지연 실패는 예약/revision을 복구하거나 자동 재시도를 열지 않는다.
- pending 조회 후 동일 요청을 재조회할 권리가 남아야 한다. pending에 consume=true를 주어 거절될 때도 결과와 조회 권리를 잃지 않아야 한다. terminal은 명시 consume 성공 뒤만 해제하며 이후 NotFound, 다른 engine 재귀속0을 요구한다. 구체 borrow/반환 타입은 SDK가 확정한다.
- 공개 결과에서 pending/terminal, queue/write/ACK 단계, 원 오류 매핑을 읽을 수 있어야 한다. 위치 관측/landing은 별도다.

SDK가 위 사용법과 수명을 단일 정본에서 확정하면 앞 최소 소비 exact를 실제 서명에 맞춰 축소하고 중앙 배정을 요청한다. 현 단계에서는 공유 소스/Cargo/OS/GUI/모델0 및 인가false를 유지한다. 기존 modoo.today 계획 작성 본문을 계속 적용하여 이 전달 사실과 미합의 항목을 구별했다.

### root87992b3 이후 읽기 인계

중앙 통보 기준 root87992b3/Mearid669 인수, SDK combined29 heavy 준비 중이다. 이 세션은 해당 실행을 수행하지 않았다. 단일 공통 정본은 [SDK 계약](native-guarded-observation-contract.md)이며 아래는 그 소비 요구만 정의한다. source11 manifest6A90A9D65B70F0F57B3BB34F219508BB8F042AAA7792B4C9B0535EB6292909D9 및 `runtime/development/sdk-native-observation-r1/src/application/playback_session/guarded.rs`를 직접 읽었다. live root에는 아직 guarded.rs가 없으므로 통합된 API로 오인하지 않는다.

source11은 관측 진단만 제공한다. FileAttribution은 Unverified뿐이고 authorization=false/guard=None, submit/result는 Unsupported다. 제출·결과 타입은 자리표시자이므로 현 단계에서 native_port를 제품에 배선할 수 없다.

최소 소비 exact 후보(구현 배정 아님):

| 단계 | 이 담당 경로 | 목적 |
| --- | --- | --- |
| Rust 소비 연결 | 신규 `src/application/skip_segments/native_port.rs`; 기존 `skip_segments/{mod,ports,service}.rs`, `skip_segments/host/registry.rs` | SDK 호출별 관측/guard를 기존 평가·예약 경로로 전달. 장수 last-guard 캐시0, domain 정책 변경0 |
| Rust 소비 회귀 | 신규 `src/application/skip_segments/native_port_tests.rs`; 기존 `skip_segments/tests.rs`, `skip_segments/host/tests.rs` | 기존 one-pass/취소/override 및 stale/오류 전파 회귀. 신규 test 파일 등록만 mod.rs에 연결 |
| 실제 host 조립 | 기존 `src/bin/player-cli/{native_host,playback_host}.rs` | 동일 Arc<EpisodePlaybackSessions> 소비. 별도 engine·timer·GUI 경로0. 실제 인수 전 capability false 유지 |
| CLI 후속 결과 | 범위 미확정. 기존 `skip_segments/host/{contracts,dispatch,schema}.rs`, `application/api/client/skip_segments.rs`, 신규 `tests/native_skip_cli.rs`가 영향 후보 | SDK 결과 조회 계약 확정 후 additive wire만 별도 배정. 기존 segments.skip DTO에 ticket 필드를 임의 삽입하지 않음 |

공용 API 차단(경쟁 인터페이스를 제안하지 않고 SDK 정본에 해결 요청):

- NativeGuardObservation은 기존 PlaybackStatus 대신 seconds 기반 native 표본을 제공한다. 기존 core가 요구하는 position/duration us·paused/idle/connected/error 의미로 투영할 공용 SDK 경계가 필요하다. skip 소비에서 별도 반올림·범위·오류 정책을 구현하지 않는다. 기존 status와 native 표본의 세대 정합성이 미확정이면 합쳐서 인가하지 않는다.
- GuardedSeekSubmission은 현재 ticket만 갖고 기존 core의 ControlReceipt 반환에 직접 대응하지 않는다. SDK 정본의 queue admission 의미를 소비할 공개 결과가 확정되어야 한다. ticket 존재로 기존 receipt를 임의 생성하지 않는다.
- NativeSeekTicket은 Clone이 없고 query가 ticket을 소유 이동한다. pending 조회 후 다시 조회/terminal 소비할 때 동일 ticket을 어떻게 유지하는지 공용 사용법이 아직 없다. 임의 복제·외부 mint·새 ID 체계를 만들지 않는다.
- GuardedSeekResult는 private 자리표시자라 queue/write/ACK/오류·terminal 여부를 읽을 공개 결과가 없다. 결과 관측/명시 소비와 원 오류 표시를 SDK 정본에서 확정해야 한다.
- guard 발급 인과, 파일 귀속, revoke/write 경계, native 오류 보존은 SDK 소유다. source11을 소비해서 이 공백을 해결했다고 표시하지 않는다. core의 기존 SkipResult와 guarded 후속 결과를 연결할 additive CLI 소유권·수명도 중앙 배정이 필요하다.

실제 mpv 통합 인수 조건(작성한 시험/통과 결과가 아님):

| 시나리오 | 실제 SDK/CLI에서 남길 근거 |
| --- | --- |
| same-instance 관측→manual | 고정 기존 로컬 미디어·mpv·source pin, 같은 host endpoint/engine/resolution과 SDK가 발급한 동일 관측 근거, verified 수동 구간의 기존 판단→원 요청 결과→별도 위치 관측. source PTS/정확 도달 보장 주장0 |
| override | 명시 override/backward 후 기존 auto 억제와 manual 동작을 각각 확인. 늦은 ACK/위치가 억제를 해제하지 않음. 실제 Auto tick은 미지원이므로 auto 정책 평가와 unattended 실행을 구별 |
| duplicate | 같은 pass의 접수·실패·지연 결과 뒤 자동 재시도0, 같은 session 재open으로 pass 초기화0. 수동 재시도는 기존 revision 규칙을 따름. 실제 write 요청 수와 결과 ticket을 연결 |
| revoke/교체/close | foreign/stale 관측은 enqueue0; admission 뒤 revoke는 SDK가 약속한 write 경계에 따라 RejectedBeforeWrite 또는 실행 미관측으로 보고. 원 engine 결과를 새 engine/pass에 재귀속0. 접수 후 취소를 seek 철회로 표시0 |
| result | queue/write/ACK/landing을 별도 기록. 원 pipe·ACK 실패, 부분 write·연결 끊김, pending 재조회/terminal 소비/64slot 소진을 검증. 실제 장애를 만들 수 없으면 해당 사례는 결정적 합성 회귀와 실제 미검증으로 구분 |

host18의 실제 LocalHost+CLI4/37요청은 합성 playback 포트 증거다. 실제 mpv seek·file attribution·OS host generation·revoke/write 인과·원 request 오류·landing 인수로 재사용하지 않는다. SDK source11의 작성9도 실제 통과 수가 아니다. 이번 문서 후속은 코드/Cargo/OSGUI/추론0이며 신규 다운로드·worker 빌드0이다. 전체 제품 OP/ED 목표는 유지한다.

SDK 최소 서명이 오기 전 identity를 중복 구현하지 않는다. 아래는 기존 정책을 소비하기 위한 입력·오류·검증 요구다.

| 항목 | 소비 요구와 회귀 조건 |
| --- | --- |
| 입력 | 현재 binding, 기존 PlaybackStatus 의미, tri-state seekability, 실제 관측에 근거한 authorization, 해당 관측에 묶인 불투명 제출 근거가 필요하다. 세대 생성/해석은 SDK만 소유한다. |
| 평가/제출 | 동일 호출이 얻은 관측 근거를 제출에 전달한다. foreign/stale/revoked 근거는 submit0이어야 한다. core 정책은 기존 evaluate를 공유하며 새 eligibility 규칙을 만들지 않는다. |
| 원 오류 | 기존 ApiError code/detail을 그대로 전파한다. Unsupported를 지원/성공으로, queue 접수를 완료로 바꾸지 않는다. stale/revoked/unknown의 구체 오류 매핑은 SDK 계약을 따른다. 임의 NeedsInput/RevisionConflict 재분류를 먼저 고정하지 않는다. |
| one-pass | 기존 외부 admission 전 attempted 예약을 유지한다. 접수/거절/오류/지연 실패가 자동 재시도를 열거나 이전 revision을 복구하면 안 된다. 명시적 수동 재시도는 기존 정책을 따른다. |
| 취소 | 기존 제출 전 취소는 observe0/submit0. 접수 이후 취소·대기 취소를 실제 seek 철회나 Before로 보고하지 않는다. stale revision 취소는 상태를 바꾸지 않아야 한다. |
| override | 기존 backward/override 억제와 명시 새 pass 경계를 유지한다. 늦은 ACK/위치/오류가 override를 해제하거나 pass/revision을 초기화하면 안 된다. |
| 지연 결과 | Queued 뒤 IPC 실패는 동일 요청의 원 오류로 관측하고 완료로 승격하지 않는다. 결과 보존/소비는 SDK 계약을 따르며 segments의 기존 응답을 소급 수정하거나 자동 재전송하지 않는다. 늦은 결과는 현재의 다른 binding/pass에 적용하지 않는다. |
| freshness 회귀 | 단순 SDK 재조회로 native event age가 감소하지 않아야 한다. 무관 ACK로 position 수신 sequence/시각이 바뀌지 않아야 한다. 기존 published_age가 갱신되는 것은 그 자체로 실패가 아니다. |
| capability/CLI | 실제 관측·guarded submit·후속 오류의 CLI 근거 인수 전 Unknown/authorization=false/submission=false. synthetic fixture·surfaceReady·queueAccepted로 지원 승격0. Auto tick/detect는 계속 별도 미지원이다. |

회귀는 공통 SDK 및 명시 endpoint의 새 CLI 프로세스로 확인하며 GUI 반복 검증으로 대체하지 않는다. 실행 슬롯·실제 미디어는 중앙 배정 후다. 위 요구를 만족하는 SDK 최소 서명 수신 후에만 소비 exact를 축소·확정한다.

## Outcome — 목표

공통 Rust SDK와 기존 native session/control/status를 통해 명시적 CLI skip을 실제 mpv 관측에 연결한다. 현재 `Seekability::Unknown`, `observation_authorized=false`, 실제 host의 `seek_submission=false`는 유지한다. 실제 미디어 인수 전 지원으로 승격하지 않는다. Auto tick과 detect는 별도 미지원이다.

## Authority — 결정권

| 범위 | 담당·결정권 |
| --- | --- |
| 실행 슬롯·공유 snapshot·root 통합 | 중앙 PM. 현재 UI가 heavy 소유 |
| native surface/session 공통 변경 | SDK와 파일 소유권 조율 후 PM 배정 |
| skip 계약·정책 연결 | 자막/스킵. 기존 core 정책 재사용 |
| 실제 seek 권한 개방 | 원본 관측·CLI·실제 미디어 근거 인수 후 중앙 결정 |

SDK 읽기 회신: 현재 source19는 playback_session/mod, mpv/session_engine, deferred_engine, events, ipc, control을 변경하지 않아 직접 파일 중첩은 없다. 공통 mpv/mod 등록은 별도 조율한다. native 수신 identity·guarded submit·worker dequeue·요청별 결과는 SDK 별도 exact 배정, 이 담당은 기존 core/port 소비를 맡는 안을 중앙에 제출했다. 이는 구현 배정 확정이 아니다. engine/resolution/file/connection 세대와 원 active lock 및 route/parent revocation을 함께 고려하며 surfaceReady를 native 관측 권한으로 사용하지 않는다. source19 동결·코드/실행0을 유지한다.

## Evidence — 확인한 근거

2026-10-20 소스 읽기이며 신규 Cargo·OS·미디어 실행은 0이다. `rg -n 'publish|snapshot|fn observe' src/infrastructure/mpv/session_engine.rs`와 관련 모듈 직접 읽기로 확인했다.

- `skip_segments/ports.rs`는 현재 Unknown/미인가를 반환한다. 두 실제 CLI host는 동일한 EpisodePlaybackSessions를 공유하지만 submit은 비활성이다.
- `session_engine.rs::observe`는 `mpv.snapshot()`을 다시 publish한다. `playback_status/observed.rs`의 publish 시각은 읽기로 갱신될 수 있다. `published_age_ms`만으로 새 native 수신을 증명할 수 없다.
- `mpv/ipc.rs`는 time-pos/duration/pause 등을 구독하지만 seekable을 구독하지 않는다. `events.rs`에는 속성별 수신 세대·시각이 없고 성공 ACK는 전역 개수다.
- `mpv/control.rs`의 receipt는 bounded queue 접수다. 이후 pipe write/명령 ACK/위치 도달을 개별 명령 결과로 증명하지 않는다. `session_engine`의 기존 active lock·engine ID 검사는 재사용해야 한다.
- mpv 공식 문서의 [seekable 및 partially-seekable](https://mpv.io/manual/stable/#properties)은 seekable=true도 모든 seek 성공 보장이 아님을 명시한다. [이벤트](https://mpv.io/manual/stable/#events)의 playback-restart는 초기 로드 후에도 발생한다. 단독 완료 근거로 사용하지 않는다. stable 문서 열람은 설치 mpv 버전의 실제 지원 검증과 다르다.

적용 스킬: modoo.today `modootoday/spec-authoring/plan` 본문 및 참조 `modootoday/spec-authoring/plan-authoring` 본문. 관측/가정 분리, 결정권, 단계별 완료 조건과 복구를 적용했다. 무관한 healthcare 검색 결과는 채택하지 않았다.

## Unverified — 미검증과 최소 계약 제안

아래는 설계 제안이며 실제 런타임 보장이 아니다.

1. native 수신 경로에서 engine/resolution에 더해 connection/file 세대와 속성별 수신 근거를 보존한다. seekable은 true/false/unknown을 구별한다. null·부재·연결 종료·파일 전환은 권한을 닫는다. 단순 조회·무관 ACK로 위치/seekability 근거를 새로 만들지 않는다.
2. duration/pause는 값이 안 바뀌어도 정상이다. 임의 TTL이나 모든 값의 변경을 요구하지 않는다. 동일 파일 세대의 유효성 및 필요한 경우 상관 ID가 있는 재조회 표본의 경계를 정의하고 실제 순서 시험으로 확정한다. 여러 get_property가 원자적 표본이라는 보장은 하지 않는다.
3. 관측 후 일반 control만 호출하면 관측과 submit 사이 교체가 가능하다. 기존 active 소유권 아래 기대 engine/resolution/관측 세대를 다시 검사하는 guarded submit 계약이 필요하다. IPC worker가 큐에서 꺼낼 때까지의 파일 전환도 별도 검사해야 한다. UI나 timer에 정책을 복제하지 않는다.
4. 공개 trait 확장은 default-unsupported인 추가 메서드/별도 계약을 우선 검토한다. 기존 공개 struct 필드 추가로 외부 literal을 깨뜨리는 방식은 피한다. `playback_session/mod.rs`, `mpv/session_engine.rs`, deferred forwarding은 SDK surface 작업과 겹치므로 미배정 편집하지 않는다.
5. 접수/실제 pipe write/해당 request ACK/후속 위치 관측을 구별한다. 원래 ControlReceipt를 완료로 바꾸지 않는다. 원본 seek 오류를 돌려주려면 요청별 상관 결과 계약이 필요하며 전역 error/ACK로 대체할 수 없다. 위치 관측도 수동 seek 등 외부 개입이 있으면 인과를 단정하지 않는다.
6. 기존 one-pass 예약·취소·override·revision 및 중복 자동 시도 방지를 유지한다. 제출 뒤 취소는 이미 접수한 seek를 되돌린다고 보고하지 않는다.

설치 mpv 버전별 unavailable 응답·이벤트 순서·부분 seek·외부 조작 구분·단조 시각 및 상관 결과 보존 한도는 미검증이다. 새 worker나 timer 필요성은 아직 인정하지 않는다.

## Phases — 단계와 완료 조건

### 中央 요청 후속: exact 후보와 추가 계약

아래 경로는 배정 요청 목록이며 수정한 소스가 아니다. 공통 SDK 작업과 skip 소비 작업을 별도 exact로 인수한다.

| 소유 | 신규 책임 모듈 후보 | 기존 최소 연결 경로 |
| --- | --- | --- |
| SDK/application | `src/application/playback_session/guarded_seek.rs` | `src/application/playback_session/mod.rs`: 모듈 공개·기본 메서드 및 Sessions 전달만 |
| SDK/native 수신 | `src/infrastructure/mpv/seek_observation/mod.rs`, `state.rs`, `tests.rs` | `mpv/mod.rs`: 등록·소유 핸들, `mpv/events.rs`: 수신 이벤트 전달, `mpv/ipc.rs`: 구독/조회 상관 및 수신 전달 |
| SDK/native 명령 | `src/infrastructure/mpv/guarded_seek/mod.rs`, `results.rs`, `tests.rs` | `mpv/control.rs`: 기존 문자열 큐를 내부 typed envelope로 감싸는 연결, `mpv/session_engine.rs`: active 검사/추가 계약 구현 |
| SDK/수명 전달 | 신규 없음 | `mpv/deferred_engine.rs`, `mpv/parent_router/mod.rs`: 기존 admit/route 경계를 통한 추가 메서드 전달. `parent_binding.rs`는 새 세대 검사가 현 admit으로 충분하지 않을 때만 별도 범위 요청 |
| skip 소비 | `src/application/skip_segments/native_port.rs` | `skip_segments/mod.rs`, `ports.rs`, `service.rs`, `host/registry.rs`: 아래 guarded 경로 연결·기존 평가 함수 공유. 실제 두 host 조립 변경은 `src/bin/player-cli/native_host.rs`, `playback_host.rs`에 별도 exact 배정 |

SDK trait 서명 제안(문서 예시이며 새 코드가 아니다):

```rust
fn observe_seek(&self) -> Result<SeekObservation, ApiError>;
fn submit_guarded_seek(&self, input: GuardedSeek) -> Result<GuardedSeekAdmission, ApiError>;
fn seek_result(&self, input: SeekResultQuery) -> Result<SeekRequestResult, ApiError>;
```

세 메서드는 EpisodePlaybackEngine에 기본 `Err(ApiError::new(ErrorCode::Unsupported, ...))`를 제공한다. 기존 구현·fake는 새 기능을 자동 지원하지 않는다. Sessions는 입력 검증 후 전달한다. 기본 메서드는 기존 `observe/control`로 fallback하지 않는다.

새 타입은 기존 `EpisodePlaybackObservation`, `NativePlayback`, `ControlEpisodePlayback`, `ControlReceipt`, `SkipObservation` 필드를 변경하지 않고 별도 정의한다. `SeekObservation`은 기존 playback/binding 값과 발급자만 생성할 수 있는 opaque `SeekObservationGuard`를 묶는다. guard는 engine/resolution, native 수신 인스턴스, connection/file generation, route/parent epoch, 관측 barrier 식별자에 바인딩한다. caller JSON이 guard를 만들 수 없게 하고 host 외부 복원·직렬화는 제공하지 않는다. `GuardedSeek`는 guard와 기존 control을 포함하되 Seek 외 입력을 거절한다. `GuardedSeekAdmission`은 기존 ControlReceipt와 별도 opaque 요청 ticket을 포함한다. receipt 의미·필드는 그대로다.

내부 수신 상태는 private 구조체로 둔다. 각 속성(time-pos/duration/pause/idle/seekable 및 부분 seek 여부)은 known/unknown 값, connection/file generation, native 수신 sequence와 단조 수신 시각을 가진다. 수신 instance는 호스트 수명 내 checked nonzero counter로 발급하고 wrap 시 실패한다. 기존 engine/resolution에 함께 묶으므로 재시작 복원 ID로 사용하지 않는다. query barrier는 요청별 값 수집 도중 세대 변경·종료·revocation이 있으면 폐기한다. 무관 ACK·SDK read·snapshot publish는 갱신원이 아니다. 속성 시각은 진단용이며 임의 TTL 권한은 두지 않는다. barrier의 시점 일관성과 외부 조작 경쟁 보장은 실제 IPC 순서 검증 전 미확정이다.

submit은 원 active/route admission 아래 guard를 검증하고, 기존 단일 IPC worker가 실제 write 직전에 세대/revocation을 다시 검사한다. IPC 관측과 명령의 직렬 처리는 mpv 내부 상태에 대한 원자적 compare-and-seek가 아니다. 읽지 못한 전환/외부 조작 경쟁을 제거했다고 주장하지 않는다. 실제로 보장 가능한 경계를 SDK가 확정하기 전 권한은 false다.

원 요청 결과 보존 한도 제안: 기존 queue64와 별개로 pending 최대64, terminal 최대64, native 관측은 현재 세대의 최신 값만 보존한다. pending을 덮어쓰거나 시간 만료로 성공 처리하지 않는다. admission 때 결과 슬롯까지 예약하여 가득 차면 `ResourceExhausted`로 write 전에 거절한다. terminal은 명시 소비/ack 후 해제하며 미소비 결과를 임의 eviction하지 않는다. close 때 pending은 종료/미관측으로 전이하고 SDK owner가 살아 있는 동안 조회 가능하게 두되 owner drop 후 복구는 없다. 원본 저수준 오류는 기존 ApiError 매핑과 source chain을 유지하고 미디어 URL·비밀정보를 추가 노출하지 않는다. 보존 객체/오류 메시지 바이트 한도는 SDK의 기존 오류 형식 검토 후 결정할 미확정 항목이다. 숫자64는 기존 queue와 맞춘 제안이며 부하 측정 결과가 아니다.

결과는 Queued/Written/CommandAccepted/Failed/OutcomeUnobserved 및 별도 위치 관측으로 구분한다. request_id별 원래 pipe/ACK 실패를 보존하며 전역 ACK나 playback-restart로 특정 seek 완료를 만들지 않는다. 조회 취소는 이미 쓴 seek의 철회가 아니다. 후속 위치가 목표 근처라는 사실도 exact seek 성공·원인 확정과 구별한다.

skip 쪽은 새 `GuardedSkipPlaybackPort`의 observe/submit에 위 guard/ticket을 전달한다. 기존 SkipPlaybackPort와 synthetic 테스트 경로는 보존하되 새 native 구현이 기존 submit으로 fallback하지 않게 한다. service는 기존 evaluate의 정책 판단을 private 공통 함수로 추출해 guarded 경로와 공유하고 예약/취소/override 로직 역시 한 구현만 유지한다. 이 작은 core 연결도 별도 exact 검토 대상이다. guard를 장수 shared port의 '마지막 관측' 캐시에 보관하지 않는다. 각 skip 호출의 관측 값과 guard를 같은 스택 수명으로 전달하여 다른 evaluate/open 호출이 덮어쓰지 못하게 한다.

현재 wire/schema를 바꾸지 않는 1단계는 내부 SDK 계약·관측 회귀까지다. 기존 segments.skip 결과에는 ticket 필드를 끼워 넣지 않는다. 원 요청의 후속 오류·관측을 실제 CLI에서 조회하려면 별도 additive 명령/typed client/schema가 필요하며 이는 미배정 2단계 exact다. 이 추가 CLI 계약과 실제 미디어 gate 전에는 실제 host의 submission을 열지 않는다. 기존 ControlReceipt를 통해 뒤늦은 오류까지 반환했다고 표시하지 않는다.

1. 현재 단계: 소스 공백과 위 최소 계약을 중앙·SDK에 전달한다. 완료 조건은 충돌 파일과 담당 경계 회신이다. 코드·Cargo·OS·공유 snapshot 변경은 없다.
2. 배정 후 Rust 수신 상태 모듈과 guarded 계약을 최소 범위로 구현한다. 신규 책임 모듈은 IPC 관측 상태/명령 결과와 application 계약으로 분리한다. 완료 조건: false/unknown/null, 같은 값 반복, 무관 ACK, 늦은 옛 세대 이벤트, 교체/close, poisoned/busy, 큐 full/disconnect, 원래 transport 오류의 결정적 회귀 통과. 기존 core를 다시 구현하지 않는다.
3. 중앙 단일 슬롯에서 fmt/test/clippy와 실제 LocalHost+새 CLI 프로세스 회귀를 수행한다. 같은 endpoint에서 stale binding/revision, 교체·close, 취소 전/접수 후, override, 중복 방지를 확인한다. 합성 IPC 결과와 실제 mpv 결과를 구별한다. 소스 핀·명령·exit·소유 PID 정리를 보존한다.
4. 중앙이 배정한 기존 로컬 미디어와 고정 mpv로 CLI 실측한다. 실제 seekable/unknown/불가, pause/time/duration, 새 파일 세대, 원래 seek 오류, 제출 후 위치를 확인한다. 부분 seek/실패 fixture가 없으면 해당 사례는 미검증으로 남긴다. 모델·다운로드·신규 worker 빌드는 없다. 완료 조건은 실제 지원 범위만 중앙 인수하며 자동 tick/detect는 계속 별도다.

## Rollback — 복구

1단계는 제안 문서를 후속 근거로 수정하며 원판 이력을 지우지 않는다. 2단계는 미인수 연결을 제거하고 기존 Unknown/미인가/submit 비활성으로 복귀한다. 3·4단계 실패는 원로그를 보존하고 권한을 열지 않는다. 실제 미디어에서 이미 수행한 seek는 자동 복구할 수 없으며 테스트 소유 재생 세션 종료로 정리한다. root stage/commit은 PM만 수행하고 push하지 않는다.
