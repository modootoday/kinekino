---
id: plan:kinekino/20261029113935
title: V2 NoMedia opt-in 실제 조립 — 읽기 후보
status: active
created: 2026-10-29
updated: 2026-10-29
---
# V2 NoMedia opt-in 실제 조립 — 읽기 후보

2026-10-21 PM 배정 읽기 계획. 코드/API/Cargo/shared/OS 변경0. V1 source10 및 SDK B의 완료를 기다리며 같은 문서 상태를 반복 정리하는 대신 실제 타입·소유 경계를 확인했다. Meari 담당의 독립 읽기 인계 뒤 추가 확정한다.

## 실제 제약과 재사용

`viewer_cleanup.rs`의 ViewerController/ViewerOwners/ViewerCleanup은 ViewerNoChild 구체형이다. Returned는 NativeRunOutcome<ViewerController>와 controller.runtime의 Rc 동일성을 검사하고, 실제 ChildWindowHost를 감싼 NativeHostRuntime에 원 request_close/observe_close를 보낸다. ViewerHost 내부 종료는 endpoint joins→callback ACK/registry→ExecutionOwner 순서를 갖는다. NoMedia를 넣기 위해 이 경로를 복제하거나 기존 NoChild alias만 QuarantineChild로 바꾸면 안 된다.

`viewer_native_launch.rs`는 공급된 PreparedNativeExecution에서 원 loop/startup/execution을 한 번 꺼내고, 같은 SDK client·LocalHost로 ViewerHost를 만든다. read-panel 준비/실제 callback document mount/bind_document 뒤 ShellHostController를 만든다. 현재 install_frame_receiver/install_frame_consumer와 ShellFrameConsumer<QuarantineChild>의 typed bind가 이미 존재한다. consumer bind 시 실제 preparation.surface_slots/children/document가 입력되며 원 실패 carrier와 pending document/host가 controller에 남는다. attached는 첫 실제 View/frame 전에 wake를 설정한다.

현재 확인 범위에서 Meari 새 API 필요 근거는 없다. 제품 측 generic 소유 carrier와 별도 NoMedia setup 실패 carrier가 필요하다. Native API 충족 여부의 최종 근거는 독립 읽기/컴파일/후속 실제 슬롯으로 나눠 인수한다.

## 작은 구현 체크포인트 후보

| 단계 | 정확 경로 후보 | 목적·비입력 |
|---|---|---|
| V2a 타입 재사용 | viewer_cleanup.rs, viewer_native_launch.rs, viewer_cleanup/tests.rs | TypedViewerController<C>/TypedViewerOwners<C>/TypedViewerCleanup<C>/TypedPreparedViewerLaunch<C>로 원 carrier를 재사용하고 기존 ViewerController/ViewerOwners/ViewerCleanup/PreparedViewerLaunch 이름은 ViewerNoChild 구체 alias로 유지. 원 finish_viewer는 구체 signature를 유지하고 내부 typed cleanup에 위임. 새 native child/loop/정책0 |
| V2b opt-in | viewer_native_launch.rs, 신규 viewer_native_launch/no_media.rs, 신규 viewer_native_launch/no_media/tests.rs | 공통 준비 helper를 통해 같은 document와 원 외부 owner를 받고 명시 NoMedia launcher를 추가. 기본 NoChild 호출은 바꾸지 않음 |
| 필요 시 책임별 추가 | viewer_host/mod.rs, viewer_callbacks/codec.rs | 실제로 사용되기 시작한 정확 API의 dead_code 속성만 제거하거나 새 선언 입력의 provider hash 책임이 생긴 경우만 조율. 속성26 전체 일괄 제거/넓은 fingerprint 재설계는 하지 않음 |

경로는 모두 `src/assembly/meari/` 기준이다. 확정 exact lease가 아니라 실제 배정 전 후보이며 V2a/V2b를 하나의 거대 소스로 합성하지 않는다. frame727행 추출은 이 표와 별도 의미 불변 체크포인트다. generic화는 typed source owner를 유지하는 수단이며 NoChild fallback, dyn cleanup executor, runtime downcast를 만들지 않는다.

## 생성·설치·실패 순서

1. 호출자는 원 ViewerLaunchInput과 trusted Rust NoMedia options(request NonZeroU64/절대 expires_at/NonZeroUsize budget)를 준다. video selector나 HostStamp를 외부에서 받지 않는다. 새 worker/LocalHost/SDK/loop를 만들지 않는다.
2. 기존 준비 로직을 공통 helper로 재사용해 실제 callback document를 한 번 mount/bind한다. 기존 shell/document 실패는 같은 원 ViewerOwners<C>에 보존한다. default launch는 그대로 NoChild를 선택한다.
3. V1의 borrowed inspect_viewing_video를 **현재 같은 document handle에서** 호출한다. typed Primary ID와 현재 fullstamp로 원 FrameRequest를 한 번 만들고 원 deadline/budget을 고정한다. V1 과거 metadata를 장기 권한으로 사용하지 않는다.
4. NoMediaFrameConsumer::new에서 consumer/receiver 한 쌍을 얻는다. 같은 ViewerHost에 receiver를 한 번 설치하고 같은 PreparedShellDocument로 ShellHostController<ViewerHost,QuarantineChild>를 만든다. consumer를 시작 전에 한 번 설치한다. 원 registrar/coordinator 생성은 실제 startup preparation만 담당한다.
5. 명시 run/into_unrun은 같은 generic PreparedViewerLaunch를 통해 원 loop/startup/controller를 이동시킨다. request/receiver/consumer를 다시 만들거나 만료를 연장하지 않는다.
6. startup bind 오류는 이미 있는 FrameSetupFailure의 original binding/preparation과 원 pending document/host를 controller에 보존한다. attach 실패/returned runtime은 기존 NativeRunOutcome와 같은 generic ViewerOwners가 보존한다. close는 그 원 ChildWindowHost barrier를 거친 뒤 ViewerHost 종료로 들어간다.

검사 실패 또는 receiver/consumer 설치 거절은 아직 Native child 등록 전이다. 별도 NoMediaSetupFailure는 원 ViewerOwners<QuarantineChild>(동일 controller+UnrunNative), 원 원인, 설치 거절이 반환한 receiver/consumer를 함께 보존해야 한다. receiver가 이미 host에 설치됐다면 다시 꺼내서 새 채널을 만들지 않는다. cleanup은 원 generic finish_viewer를 사용하며 추가 failure artifact는 반환 carrier와 함께 유지한다. 별도 shutdown state machine이나 실패 시 자동 재준비는 없다. 이 실패 carrier의 최종 타입/가시성은 V2b 배정 전에 필드별로 고정한다.

controller 만들기 전 문서 검사 실패에서도 원 PreparedShellDocument 전체와 UnrunNative 및 ViewingVideoError를 보존한다. DocumentFailure의 원 mount stage로 위장하지 않고 명시적인 준비 실패 carrier로 반환한다. callback close 책임을 지운 diagnostic-only error로 축소하지 않는다.

## 검증을 주장할 수 있는 층

- CPU: 같은 실제 V1 문서의 fresh ID/fullstamp·원 request/deadline/budget 불변, typed consumer/receiver 설치 거절 반환, generic 소유 타입의 컴파일, 기존 callback/commit/ACK/close 정책 회귀. NoMedia frame port의 원14/Viewer4를 재사용한다.
- 현재 실제 ViewerHost 생성은 concrete LocalHost+ExecutionOwner가 필요하다. 기존 실제 viewer cleanup/launch 시험도 pipe/worker/Native-loop authority 때문에 ignored다. 이를 pure 테스트라고 실행하거나 API 재설계로 권한을 감추지 않는다. CPU만으로 원 실제 Viewer launcher/endpoint/worker 전체 생성·종료를 입증할 수 없다는 한계를 남긴다.
- V2 이후 별도 배정된 pipe/worker 또는 V3 OS: 실제 원 prepared launcher→typed controller→actual preparation bind·N proof 소비→owned Completion·중복/Busy/만료·원 child-first close/remaining0를 관측한다. 기존 별도 ownwindow fixture 성공을 이 생산 조립 성공으로 승계하지 않는다.

일반 visible/재생/미디어 지원·main/default 전환·시각 완료는 V2 인수에 없다. observed frame_result는 실행 권한 없는 metadata이며 SDK commit/ACK 오류를 대체하거나 hidden child를 visible로 승격하지 않는다.

## Meari 독립 읽기 인계 반영

현재 API로 observe_owners<C:NativeChild>/typed cleanup의 Returned outcome→원 runtime Rc::ptr_eq→NativeHostRuntime<ChildWindowHost<ViewerHost,C>>::request/observe_close를 유지할 수 있다는 읽기 지원을 받았다. 새 Meari 필수 API 근거는 없다. 단순 default generic은 C 필드가 없는 ShellFailure 변형의 독립 생성에서 추론 불명확을 만들 수 있으므로 위 표를 **Typed* 실체 + 기존 NoChild 구체 alias**로 정정했다. 이는 아직 소스/컴파일 인수가 아니다.

설치 단계별 원 보존 값은 다음처럼 고정한다.

| 실패 시점 | 반환에서 반드시 함께 유지할 값 |
|---|---|
| fresh video 검사 | 원 PreparedShellDocument<ViewerHost>·UnrunNative·ViewingVideoError |
| receiver 설치 거절 | 같은 prepared document/host·반환 receiver·아직 설치하지 않은 consumer·UnrunNative |
| consumer 설치 거절 | receiver/host가 이미 들어간 같은 typed controller·반환 Box<dyn ShellFrameConsumer<QuarantineChild>>·UnrunNative |
| 실제 startup bind | Returned typed outcome의 원 controller.pending document/host·FrameSetupFailure(original binding/preparation)·controller의 consumer |

현재 ViewerOwners는 rejected receiver/consumer를 담지 않는다. V2b에 제품 측 명시 carrier가 필요하며 이를 동등한 타입 이름만으로 덮지 않는다. 구현 후보는 typed owners의 `PreparedDocument` 준비 상태(원 document+native)를 추가하고, NoMediaSetupFailure가 그 owners와 rejected actor 원 객체를 함께 소유하게 하는 안이다. 준비 상태의 cleanup은 원 poll_unrun_shutdown에 위임한다. failed-setup 종료 wrapper는 같은 typed finish/drive_cleanup을 한 번 호출하고 rejected artifact를 결과 carrier와 함께 반환하며 새 shutdown executor를 만들지 않는다. 이 안과 별도 opaque setup carrier 안 중 최소 정확 경로를 V2b 배정 전 필드별 대조한다. 추가 경로가 필요하면 PM 조율한다.

V1 inspect의 pub(super) helper는 prepared document의 공개 borrowed 조회를 조립 시 호출하면 추가 가시성이 필요 없다. 실제 deferred bind 때 활성 선언까지 재검사하도록 강화하려면 제품 helper의 좁은 pub(crate) 위임이 별도 필요할 수 있으나, 현재 fullstamp 재검사 계약 이상의 요구를 V2에 임의 추가하지 않는다. Meari가 추가 제안한 이 선택 사항을 새 라이브러리 API 장애로 분류하지 않는다.
