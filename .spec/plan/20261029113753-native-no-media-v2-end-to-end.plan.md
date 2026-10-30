---
id: plan:kinekino/20261029113753
title: 실제 Viewer NoMedia 종단 시험 준비
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 Viewer NoMedia 종단 시험 준비

2026-10-21. PM 인계 대기 중 작성한 **읽기 기반 구현 계획**에서 시작했다. 당시 source 변경·Cargo·OS 실행 승인을 뜻하지 않았으며 SDK 슬롯과 분리했다. V2b CPU 인수 root는 `736de48eb7c4f87f8a75eede9fc72d4cdcd74c3f`이며 [검증 정본](../../../sots/verification/presentation/native-no-media-v2b-2026-10-21.md)과 구분한다.

후속 PM 배정: root `9ffd85597ec9305bacae20eb8e9f3fce1abfad3d`를 기준으로 source7 독립 구현 및 시험별 supervision adapter1 추가가 승인되어 **source8 작성 중**이다. 작업 overlay는 `runtime/development/ui-viewer-end-to-end-r1`, before7 준비 이력과 추가 before8은 `runtime/coordination/ui-viewer-end-to-end-r1`에 둔다. 기존 supervisor가 인자0·원 exact/env/schema 상수에 고정된 서명을 확인해 새 시험을 위한 adapter 경로만 추가했다. 현재 AI가 단일 heavy이며 UI Cargo/OS 실행 권한은 없다. 처음의 읽기 계획과 후속 source 작성 배정을 구분한다.

source8 r1을 보존한 뒤 실제 Meari close surface revocation 및 wake coalescing 계약에 맞춰 시험 단언2를 r2에서 수정했다. [현재 소스 인계](../../../../runtime/coordination/ui-viewer-end-to-end-r1/source-r2/HANDOFF.md)에 actual8·변경1/불변7을 고정했다. 순수 probe5+close 계약2와 ignored 종단2를 작성했으며 직접 Rustfmt --check만 통과했다. 컴파일·시험 실행은 아직 0이다. PM/Meari 독립 읽기 및 중앙 실행 배정을 기다린다. 원13화면 전체 목표는 유지한다.

## 실행 경로와 관측 계약

최신 source 후보는 [r3 실제8](../../../../runtime/coordination/ui-viewer-end-to-end-r1/source-r3/HANDOFF.md)이다. r2의 종료 단언 수정은 Meari/PM 독립 읽기를 받았으며, 이후 frame_budget64의 최대70개 사건을 담도록 probe 고정 용량만80으로 조정하고 회귀1을 추가했다. 작성 pure8/ignored2, 직접fmt8 check0이며 Cargo/실행0이다. get_goal 재조회로 전체13화면 목표 본문과 status=active를 확인했다. 기존 목표를 교체·complete 처리하지 않았고 source-only 배정에서 구현과 검토를 이어간다. 전체 구현/시각/실행 완료는 여전히 미입증이다.

정상 사례는 실제 owned ViewerLaunchInput과 source_generation 1의 별도 유효 입력으로 공개 prepare_no_media_viewer_launch → TypedPreparedViewerLaunch::run을 실행한다. 기존 source_generation 0 실패 fixture는 보존한다. 자체 Controller/가짜 registrar/port 시험으로 대체하지 않는다. blocking run 반환 후 finish를 호출하는 것만으로 정상 종료를 유발할 수 없으므로, run 내부 Completion 적용 뒤 원 controller의 close intent·NavigationGuards 판단과 원 NativeWindowWake를 사용하는 좁은 test-only 요청 경로를 연결한다. wake만 호출하여 기존 navigation 판단을 생략하지 않는다.

사건은 하나의 제한된 append-only Rust trace에서 증가하는 sequence로 기록한다. timestamp·최종 DTO·개별 bool로 순서를 추정하지 않는다. 기록은 request ID/full HostStamp/window identity/ticket의 비교용 메타데이터만 갖고 proof·raw HWND·등록 권한을 보관하지 않는다. 필요한 사건은 원 bind 결과, native disabled-child proof 검증 성공, 실제 FrameReflection의 update.result Some, 정상 close 요청 결과, inner Viewer close 최초 진입 시 원 coordinator report의 remaining, 최종 run/cleanup 결과다. Completion에는 reason·남은 callbacks·result·적용 횟수와 incoming turn stamp, 해당 after_dispatch의 committed before/after 및 실제 document stamp를 함께 기록한다.

정상 통과는 같은 request/window에 대해 proof 성공 < owned Completion 적용 < 정상 close 요청 < child remaining 0 상태의 Viewer close 진입 < 원 shutdown/Closed 순서를 요구한다. 각 borrow를 해제한 뒤 close intent를 보낸다. trace 재진입·overflow·관측 실패는 명시 실패이며 ACK/Closed를 합성하지 않는다. 실제 적용 횟수를 세고 기대와 다른 중간 revision도 원 모습대로 남긴다.

별도 실행은 prepare 후 startup 전 **원 pending 문서**를 borrowed cfg(test) seam으로 reproject하여 fullstamp bind 거절과 원 owner 정리를 확인한다. controller.pending은 private이므로 native_launch 자식 시험에서 직접 접근 가능하다고 가정하지 않는다. 바깥 반환값만으로 bind 거절 위치를 추정하지 않는다.

## 최소 source 후보와 책임

| 경로 (`src/assembly/meari/` 기준) | 변경 책임 |
| --- | --- |
| `mod.rs` + 새 `viewer_native_probe.rs` | cfg(test) 한정 scoped trace·단조 sequence·원 close 요청 전달과 coordinator의 약한 참조를 통한 report 읽기. 원 owner의 수명을 늘리지 않는다. |
| `player_shell_host.rs` | attached의 실제 runtime/wake에 probe 연결, pending 원 문서 borrowed test seam. 기존 request_close와 같은 route/guards 의미를 재사용하며 생산 visibility를 넓히지 않는다. |
| `quarantine_surface/frame_delivery/native_backend.rs` | consume_checked 성공 뒤 실제 proof 검증/기한 통과 사건. context의 원 frame stamp/ticket과 대응하고 proof borrow 밖에서 기록한다. |
| `viewer_host/mod.rs` | after_dispatch 성공 시 실제 reflection 적용 사건과 commit stamps, close 최초 진입 관측. 실패한 SDK outcome은 성공 reflection으로 기록하지 않는다. |
| `viewer_native_launch/no_media/tests.rs` + 새 `tests/end_to_end.rs` | 별도 ignored 정상/변경 거절 exact 두 개와 새 env/evidence schema. 공개 wrapper/run 경로를 사용한다. |

이는 파일별 구현 후보이며 PM의 source 배정 전 고정 입력 manifest가 아니다. probe 자료형과 close 요청 seam은 구현 전 독립 읽기로 검토한다. Meari API 부족은 현재 확인된 0개이며 test-only 제품 seam을 라이브러리 부족으로 보고하지 않는다.

### 소스에서 확인한 seam 제약

- `native_backend::consume_checked`는 Pending도 Ok로 반환한다. native adapter의 반환을 한 번 받아 **Ok(Progress::Observed)**일 때만 proof-complete 사건을 남긴다. verify closure 내부 기록은 마지막 deadline 검사보다 먼저이므로 완료 사건으로 사용하지 않는다. context(&self)는 &mut self를 요구하지 않으므로 test-only 현재 frame 메타데이터 보관은 interior mutability가 필요하다. request ID는 scoped trace의 최초 입력과 대응하며 backend가 원 request를 새로 소유하거나 갱신하지 않는다.
- `ShellHostController::attached`의 실제 runtime을 짧게 borrowed하여 `runtime.host().children()`의 원 Rc에서 Weak를 만든 뒤 borrow를 해제한다. ViewerHost::close 안에서 runtime을 다시 borrow하는 closure를 보관하지 않는다. 원 coordinator.report()의 오류·Weak upgrade 실패는 미관측으로 기록한다. child remaining만으로 성공을 확정하지 않고 report status와 원 window identity도 대응한다.
- 원 request_close는 route.try_borrow_mut → state.request_close(guards) → wake.request_close 순서다. test-only callback은 route Weak와 동일 wake를 사용하고 같은 순서/오류 전달을 보존한다. Completion 기록 중 trace/receiver/document borrow가 살아 있는 동안 callback을 호출하지 않는다. 기본 guards는 fixture가 생성한 깨끗한 상태에만 사용하며 실제 pending/dirty/recovery를 무시하지 않는다.
- `FrameReflection::apply`의 update.result Some에서 적용 횟수를 증가시킨다. after_dispatch 호출 수나 result 필드의 최종 존재 여부로 대체하지 않는다. 적용 결과·incoming turn과 commit receipt 메타데이터를 한 사건으로 묶는 방법을 우선하며, 원 receipt를 trace에 이동하지 않는다. `outcome?` 앞 오류 경로는 반영 완료 사건을 남기지 않는다.
- bind 거절은 controller.start의 기존 `bound` 결과에서 기록한다. 실패 후 원 pending 및 FrameSetupFailure의 preparation/binding을 유지하는 코드는 그대로 둔다. 변경 시험은 pending 문서의 stamp 전후를 기록하고 into_unrun으로 공개 run을 우회하지 않는다.

관측 설치는 scoped guard로 이전 thread-local 상태를 복구하며 각 시험 간 누출을 막는다. weak 참조 및 메타데이터만 남기고 controller/runtime/endpoint/ExecutionOwner를 새 소유하지 않는다. event 제한/sequence overflow/borrow 충돌은 실패 근거에 포함하며 원 제품 오류를 덮지 않는다. 이 제약은 source 구현·회귀로 추가 검증할 대상으로서 아직 실행 근거가 아니다.

기존 own_window의 owned_process Supervisor/WindowsFamily/Deadline·receipt·timer join 방식을 재사용한다. test identity/env/evidence는 별개다. 공유 helper 추출이 필요하면 그 source 경로를 추가 배정받고 새 종료 executor를 만들지 않는다. 원 family forced/미관측 exit는 실패로 보존한다. outer watchdog은 실패 안전망이며 정상 종료 증거가 아니다. 임의 숨김 창 옵션·VM/DPI 변경·사용자 player 종료를 추가하지 않는다.

## revision 구분과 인수

bind 전 fullstamp 변경은 거절 사례다. bind 후 reservation 이전 일반 props revision 변경은 same_source 정책상 무조건 거절이 아니다. reservation→paint/consume 변경은 원 slot stale, Observed 뒤 변경 frame은 DocumentChanged다. 후자의 전체 행렬은 첫 정상/거절 두 사례 통과 뒤 별도 범위로 진행하며 origin·예산·기한을 시험 편의로 갱신하지 않는다.

source 배정 뒤 before/root/index/provider와 독립 활성 overlay 경계를 고정한다. 중앙 슬롯에서 fmt --check, default/native all-target strict Clippy, 필요한 기존 회귀와 fresh binary의 ignored exact 두 개를 실행한다. 원 실패·binary·receipt·PID identity·family 종료와 원 owner retention을 보존하고 슬롯을 먼저 반환한다. 이 인수는 실제 NoMedia 수명주기 범위이며 영상 재생·전체 화면·13화면 시각 일치 완료를 뜻하지 않는다.

## 2026-10-21 실제 인수 관측

위 source-only 계획의 첫 정상/변경 거절 두 사례를 최종 UI17r2+Meari2e0bf 동일BIN에서 실제 수행했다. regular95와별도OS2가 통과했으며, 변경 거절은 WorkersQuiescent(Unconfirmed)로 유지한다. 원실패/receipt/보호/종료 근거 및 미완료 범위는 [제품 조립 검증](../../../sots/verification/presentation/native-product-ui17-2026-10-21.md)을 참조한다. 전체 revision행렬·미디어·시각승인을 인수한 것은 아니다.
