---
id: plan:kinekino/20261029113725
title: 2026-10-20 provenance 구현 조율 이력
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 2026-10-20 provenance 구현 조율 이력

이 문서는 당시 current-coordination.md 원문을 보존한 역사 기록이다. 현재 담당·실행 슬롯·미완료 상태는 [현재 조율](current-coordination.md)을 따른다. 아래의 진행 중/대기 표현은 보존 시점의 상태이며 최신 지시를 대체하지 않는다.

2026-10-20. 관측·구현·미검증을 구별한다. 제품 전체 완료나 사람의 품질 승인을 뜻하지 않는다. 이전 원문은 [10월20일 이력](coordination-history-2026-10-20.md)과 [회차·커서 이력](transition-cursor-coordination-history-2026-10-19.md)에 보존했다.

## 담당과 실행 자원

| 담당 | 인수한 근거 | 현재 책임 |
| --- | --- | --- |
| PM | Meari 최종17+13·strict/fmt·입력/소유 정리 인수 | 단일 실행 슬롯을 SDK 영속성 검증에 재배정 |
| SDK | parent 등록 root385/19ignored·consumer47/0ignored·각 strict/fmt·실제 reader9개 인수 | CLI/provenance 원자 저장·재조회 source 준비, 단일 heavy 재배정 |
| UI | bridge r2 전체46/0fail/0ignored·strict/fmt·외부delta0·소유[] | F/G 인수표·단일창 계획 완료. 새 독립 bootstrap Rust 파일/시험 준비, 부모/기존 source 동결·빌드 없음 |
| Meari | lifecycle17·기존 headless host13·strict/fmt·source20e21d11…c4bc1·소유[] 인수 | Native source/tests/manifest 동결. 실제 driver/창 연결은 후속 |
| AI | API·모델 결과·품질 결정·provenance 연결 차이 조사 | 추론/다운로드 보류. 제품 통합 뒤 최종 품질 검토 |

단일 process family·2CPU/affinity3·BelowNormal·descendants Job·-j1·RUST_TEST_THREADS1을 유지한다. 기본 offline/locked이며 의도된 최초 lock resolve만 별도 기록한다. 다음 슬롯은 terminal·소유 정리·명시 반환 뒤 배정한다. 기존 캐시/사용자 ChatGPT·Terminal은 종료·삭제하지 않는다. Pages·VM·DPI·새 장시간 시험은 보류다.

## 현재 배정: CLI 영속성/provenance와 Native 수명 구현

PM은 parent checkpoint의 명시 반환과 최종 inventory082c4035…ad59·root/consumer package identity 일치·소유[]를 인수했다. SDK는 [provenance 계획](ai-provenance-integration.md)의 기존 결과 기반 실제 reader·동일 원자 저장의 binding·fresh CLI inspect/edit/accept 재요청·project capture/CAS/restart 및 부정 입력 인수를 구현한다. 최초 코드 변경 전에 정확 파일 소유·저장 schema 호환/migration 계획을 고정한다. 새 추론/다운로드는 없다.

[제품 연결 checkpoint](historical-provenance-product-checkpoint.md)에 additive binding의 legacy bytes/capture SHA 보존·단일 publication·실제 reader 재검증 범위를 고정했다. PM은 고정 Rust verifier의 inspect-import 실행450ms/0과 준비 application 모듈의 root 등록 후 집중62006ms/0·3pass를 원문 확인했다. 후자는 MemoryStore/직렬화 fixture이며 실제 reader·디스크 원자 저장·fresh CLI·project 연결 완료가 아니다. 현재는 명시적 pinned verifier 프로세스 어댑터를 선택했고, packages library 승격과의 비용·배포 비교를 계획에 기록했다. 명시 설정/배포물 없이는 지원으로 표시하지 않는다.

후속 binding 집중60510ms/0·root8pass/2ignored와 실제 CLI cli-first107497ms/0·정확1시험/본문31.11s를 확인했다. 후자는 원본9개 import/fresh inspect 및 correction/cards/highlights 각각 원문 참조·기술 수정·foreign/stale 거부·accept/retry·자동 적용false·원문/binding 불변·project capture/reopen, scratch 누락/변조/binding 제거·교환/부분 publication 거부를 포함한다. PM은 고정CLI8065a232…c6266, 카드 edit PID65448와 재조회30704, 하이라이트 프로젝트 재조회119796의 별도 프로세스/reaped 기록과 카드 채택 revision2→3 및 companion 변조 후 accept 재요청 실패를 직접 대조했다. 새 historical document3/binding1은 일반 document/source1/2와 구분하며 legacy bytes/capture SHA 유지 회귀를 포함한다. root 전체/strict·consumer·최종 pins/정리는 아직 후속이고 사람 품질·render/export·GUI 완료는 아니다.

PM의 새 reader 소스 검토에서 동기 subprocess의 GUI thread 지연 가능성을 확인해 SDK/UI에 전달했다. 현재 Immediate adapter와 Pending 방어 시험은 실제 provider 비동기 완료를 보장하지 않는다. CLI 기능 인수 뒤 비동기 scheduling/typed Pending·실제 terminal wake를 제품 통합 잔여로 관리하며 Rc document/registry/GL을 worker로 이동하지 않는다. stdout/stderr 초과·timeout·nonzero·pipe 실패·kill/wait 실패에서 deadline과 실제 소유 정리를 구별하는 Rust 인수도 SDK에 요청했다. 이는 읽기 검토이며 결함 재현/수리 완료 근거가 아니다.

Meari는 별도 meari-native crate 안의 domain/application/infrastructure lifecycle·driver·native export와 Rust 시험 소스를 담당한다. core/다른 crate/모든 manifest·lock/제품 source는 수정하지 않는다. 승인된 owner/epoch·등록 lease·동시 cleanup 예약·주입 deadline·child-first 합성·무유실 terminal wake를 구현하며, 공통 상태 시험부터 실제 child 연결까지 단계별로 인수한다. cargo/GUI 슬롯은 아직 배정하지 않았고 SDK와 소비 입력 충돌을 사전 확인한다. source 구현만으로 W01~13 완료를 표시하지 않는다. DPI/VM/시스템 절전 실험은 보류다.

SDK의 reader checkpoint terminal·프로세스 정리와 실행권 임시반환 뒤 PM은 Meari 공통16시험의 headless 집중 gate를 배정했다. 실행 전 정확 cmd/source inventory/features/디스크 여유를 기록하고 기존 Meari 전용 target을 사용한다. 전용 문맥이 없으면 `runtime/build-cache/meari-native-lifecycle`을 사용하며 root/consumer target은 공유하지 않는다. 실제 GUI/OS 창 실험은 이번 배정에 없다. SDK는 제품 source 작업만 계속하고 재배정 전 새 빌드를 실행하지 않는다. 앞 문단의 cargo 미배정 상태는 이 배정으로 대체하며 W01~13 실기 잔여는 유지한다.

첫 실행은 생성 unwind 시험 추가로17개 source가 대상이며 `cargo test --offline --locked -j1 -p meari-native --test window_lifecycle`, features 추가 없이 headless다. 실제 기존 전용 `packages/meari/target`을 재사용했다. PM은 focused receipt의 runner19012/child19444·affinity3·priority16384·descendants와 meari-native 컴파일 진행을 확인했다. source inventory428dd68b…1e0cb·Native15파일 delta이며 manifest/core/root 변경은 없다. 아직 시험 통과나 실제 parent/driver 연결 완료 근거가 아니다.

focused는175720ms/0·17pass/0fail/0ignored·본문0.07s로 종료했고 PM이 원 receipt/시험명을 대조했다. lifecycle epoch/revocation·foreign checkpoint·post-revoke KeepOpen 실패 보존·예약/early completion/timeout/재시도·생성 unwind를 공통 상태/합성 경계에서 확인했다. 첫 컴파일 후 추가 수리는 없으며 Native fmt/strict·기존 headless host 회귀와 소유 정리는 후속이다. 실제 driver/parent handle/Windows child·W01~13 실기 전체 완료로 확대하지 않는다.

최종 인수: 최초 strict는시험fixture type_complexity1건으로 실패했고 역할 타입별칭만 보완했다. r2집중22613ms/0·17pass, strict-r210476ms/0, 기존headlesshost26680ms/0·13pass, fmt-r2529ms/0을 확인했다. 최종source20e21d11…c4bc1·source delta[]·소유[]·manifest/lock불변과 명시 반환을 PM이 인수하고 heavy를 SDK에 재배정했다. [Meari 실행 근거](../../../packages/meari/.spec/evidence/verification/window-lifecycle-common-2026-10-20.md). Native source는 SDK consumer 회귀까지 동결하며 실제 driver/OS 연결 미완료는 유지한다.

UI에는 canonical snapshot과 명시 선택을 initial props로 변환하는 순수 Rust bootstrap source/시험 준비를 추가 배정했다. 새 독립 파일만 작성하며 mod/lib/manifest·기존 factory/bridge·SDK 소유 fixture에는 등록/수정하지 않는다. 정확 경로를 소유자와 확인하고 SDK의 변경 중 DTO는 최종 계약 후 맞춘다. 미등록 source 준비를 컴파일·제품 연결 성공으로 집계하지 않는다. 기능 정책/저장/자동 선택은 추가하지 않으며 cargo/GUI 실행은 중앙 배정 전 수행하지 않는다.

UI가 `src/presentation/web_ui_adapter/proposals/bootstrap.rs`와 `bootstrap/tests.rs`의 순수 변환부·시험8개 source를 준비했다. PM 관측 SHA는 각각5c83d5dd…26391/e294db50…28fac이며 부모 등록·컴파일·실행은 아직 없다. SDK reader 소유 시험은 reader-owner-focused-first47205ms/0·4pass/2ignored로 확인했다. 후속 reader-actual-nine-first4069ms/0·1pass/296filtered는 고정 verifier로 실제9개 원문 재구성/교환 reference 거부를 실행했고 reader strict37364ms/0도 통과했다. PM이 두 원 receipt/stdout/stderr를 확인했다. 이는 실제 reader checkpoint이며 disk binding/fresh CLI 성공이나 사람 품질 승인은 아니다.

## 최근 인수: 제품 SDK parent 등록

SDK가 root lib/assembly/Cargo/lock·제품 전용 player-ui-plugins workspace·공통 manifest/context/회복 가능한 lease 및 도구 재수출을 담당한다. 같은 SDK/3store와 actual Rc registry/context/scope를 재사용한다. 제품이 tools에 의존하거나 정책을 복제하지 않는다.

첫 resolve76ms/101은 plugin workspace 중복이었다. root의 명시 dot 제거 및 정확 exclude를 적용한 metadata-r4에서 제품7/default7·Meari 제외를 PM이 직접 확인했다. Meari 코드/자체 manifest는 변경하지 않았다. root 집중114213ms/0·4pass로 scope·close Busy 재시도·context 옵션·manifest 역할 pin을 확인했다.

consumer는 공유 target에서 동일 serde_core1.0.229의 식별 충돌을 재현했다. 동일 source/checksum·다른 Meari fingerprint·unhashed libmeari.rlib 재사용을 관측했으며 단독 인과 확정 전이다. 기존 캐시는 보존하고 D의 runtime/build-cache/sdk-product-parent-consumer에서 격리 실행했다. 최초 실행은 기존19 통과·새 시험의 빈 Review 저장소 준비 실패였고, r2는 새 시험의 close 단계 오류 기대 불일치였다. 필요한 저장소 폴더 준비와 실제 UI 단계의 Adapter 오류 기대를 수정한 r3는29092ms/0·factory18+panel2=20pass이며 PM이 receipt와 원 stdout을 확인했다. 새 실제 제품 lease/store 시험까지 통과했으나 전체 회귀·공유 target 결함 해결·OS 창 통합을 입증하지 않는다. 공개 타입 우회와 Meari 변경은 수행하지 않았다.

승인한 consumer 변경은 document_factory.rs 한 파일의 제품 lease 위임과 실제3store 시험1개다. 기존 UI46/assert/예산 의미는 유지한다. 제품 presentation/Meari 소스는 동결한다. [등록 계획](presentation/sdk-product-host-registration.md), [읽기 검토](presentation/sdk-product-parent-review.md)를 참조한다.

invalid 원 request_id의 red45553ms/101에서 직접 SDK InvalidArgument와 등록 경로 Unsupported 불일치를 확인했다. RequestContext::validate 공통화와 hash 전 검증 수리 후 집중6pass를 확인했다. 최종 root full313867ms/0·385pass/19ignored·strict55270ms/0·fmt1294ms/0, 격리 consumer full207462ms/0·47pass/0ignored·strict382830ms/0·fmt309ms/0 및 bins2364ms/0을 PM이 인수했다. 새 workspace를 포함하므로 이전373과 동일 범위로 비교하지 않는다. 최종 pins·package identity·소유 정리·명시 반환도 확인했으며 실제 창·fresh CLI/provenance 완료 근거로 확대하지 않는다.

UI와 Meari의 읽기 대조에서 현재 Native driver는 새 CPU 창을 생성하며 기존 eframe 창/GL import나 외부 native-child용 공개 parent handle/lifecycle port를 제공하지 않음을 확인했다. Meari는 owner/epoch/surface generation·등록 lease·revocation/cleanup ACK·정상/forced 종료 인수 계약을 문서화한다. 이는 미지원 경계의 설계 작업이며 새 API 구현/실제 단일 창 통합 완료가 아니다. UI 인수표는 기능을 CLI에 맡기고 실제 wake/입력/시각만 GUI에 남기며, 사용자 최신 지시에 따라 향후 조작 검토는 메인 모니터에서 수행한다. 현재 디스플레이 설정 변경이나 GUI 실행은 없다.

Meari의 [lifecycle 초안](../../../packages/meari/.spec/sots/integration/native-window-lifecycle.md)과 [ADR 후보0016](../../../packages/meari/.spec/adrs/0016-native-window-child-lifecycle.md)을 PM이 읽기 인수했다. W01~13은 전부 미실행이고 source/manifest/build 변경은 없다. UI는 React→Meari 단일 창 목표를 기준으로 Meari 최상위 창 소유와 기존 eframe import의 pinned 버전·조립 접점·이관 비용을 문서로 비교한다. libmpv나 GL 공유 채택은 전제하지 않는다.

## 최근 완료와 실제 한계

- [오디오 CLI](../../sots/verification/application/native-audio-cli-2026-10-19.md): 실제 Next/Previous·옵션·PNG·mute/volume 독립 관측·EOF 정리. 실제 endpoint 청취는 미검증이다.
- [WebView 오디오](../../sots/verification/application/native-audio-webview-2026-10-20.md): actual-r2 9135ms/0·세 leg, 최종 strict4932ms/0·fmt1294ms/0·full103183ms/0·373pass/19ignored·source inventory66157cf7…b1f41·소유[]. 첫 timeout/재실행 입력을 보존했다. WAV의 Web audio guard와 audio 없는 AVI의 native decode/IPC 근거이며 실제 provider 전환·native audio 출력·청취·실제 IPC-loss fault까지 입증하지 않는다.
- [UI bridge r2](../../sots/verification/presentation/native-window-bridge-2026-10-19.md): 선택19·전체46·strict21757ms/0·fmt310ms/0·inventory4b2c97b0…c9c50. 최초15pass2fail의 초기/재투영 예산 배선을 단일 소유로 수리했다. 실제 SDK/registry·headless Completion 근거이며 OS wake/키보드/IME/제품 창·시각 일치는 별도다. [포화 검토](presentation/native-window-bridge-review.md).
- [커서 실제 검사](../../sots/verification/application/native-pointer-cli-2026-10-19.md): 소유 mpv 창의 hover/idle/이동/가장자리 관측. embedded/fullscreen/실제 transport overlay는 미검증이다.

## 다음 인수 순서와 미완료

1. parent 등록 최종 인수 완료. 동일 입력의 반복 검사는 추가 변경/실패가 없으면 실행하지 않는다.
2. fresh CLI의 proposal 원문·수정·명시 채택 재열기와 project/provenance 원자 결합. 기존 API/저장 규칙을 재사용하며 sidecar·메모리 ref로 영속 결합을 가장하지 않는다. [계획](ai-provenance-integration.md).
3. 실제 parent lease·명시 선택·동일 SDK/registry·NativeWindow/OS wake·source 전환/창 종료 연결. eframe/공유 GL을 검증 없이 가정하지 않는다. [제품 연결](presentation/proposal-product-registration.md).
4. CLI 기능 인수 뒤 필요한 UI/UX·전체13화면 일관성, 실제 provider 오디오 전환·청취와 첫 seek stall 원인 잔여를 각각 인수.
5. 제품 연결 뒤 AI 담당과 원 candidate·수정 output·명시 결정·모델 조건 결합 및 사람 품질 검토. 새 추론/다운로드는 중앙 배정 전 실행하지 않는다.

mpv 동봉 배포·Rust adapter 분리·libmpv·source submodule은 [권장안](mpv-package-boundary.md) 단계다. 채택/구현 완료로 표시하지 않는다. 전체 Q01–Q09·모델 기능·자막 화자 분리·제작/에셋 범위는 기존 기능별 정본과 계획을 유지한다.
