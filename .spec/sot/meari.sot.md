---
id: sot:kinekino/meari
title: Meari 제품 플러그인 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Meari 제품 플러그인 계약

2026-10-19 제품 설계 계약. 실제 Meari registry의 변환 성공/오류·권한/schema 시험과 WASM 타입 검사를 완료했고, 실제 OS/Web 전체 지원은 미검증이다. [배포 결정](../../../../adrs/player/meari-product-plugin-deployment.md), [UI consumer 현황](../../../../plans/player/presentation/meari-consumer-gaps.md)을 함께 적용한다. 공개 API와 후속 미구현 설계는 아래 근거 범위로 구별한다.

## 책임과 패키지

Meari 담당은 범용 plugin SDK, manifest 검증, static registry, typed dispatch, instance/document 수명 및 공통 IR binding을 소유한다. 제품 UI 담당은 `packages/player-ui-plugins`의 제품 capability 선언과 Rust adapter를 소유한다. 새 패키지는 본체 API 인계 후 만들며 이번 consumer 검사 freeze에 섞지 않는다. 제품/환경별 구현을 Meari 본체에 추가하지 않는다.

패키지 목표 구조는 `domain/{identity,capability}`, `application/{sdk_bridge,path_selection,playback,window}`, `infrastructure/{native,web}`, `presentation/bindings`다. domain은 Meari renderer·OS·SDK 구현에 의존하지 않는다. application은 주입한 typed port에 호출을 전달하고 infrastructure가 GUI thread/browser/SDK 연결을 담당한다. TSX는 생성/공개된 binding을 선언적으로 소비하며 JS 정책·임의 command string·shell을 실행하지 않는다.

기존 `ApplicationClient` 및 typed `ConvertSubtitles`, `ConversionResult`, `ControlReceipt`, `ApiError`를 재사용한다. 자막 parser·loss 판정·신규 파일 publication·proposal CAS·재생 정책은 공통 application/domain에 남긴다. 루트 SDK를 새 패키지가 직접 끌어들여 순환 의존하지 않는다. 공유 DTO/port의 패키지 추출은 SDK 담당 소유이며 그전에는 host assembly가 port를 주입한다.

SDK 담당의 현재 source 대조: 자막 DTO 정본은 `media_subtitles::application::convert::{ConvertSubtitles,ConversionResult}`, 경로 port는 `media_subtitles::application::path_picker`, 오류는 `media_contracts::errors`다. root application/subtitles/mod.rs는 이를 re-export하며 미등록 잔여 convert.rs는 새 정본이 아니다. ApplicationClient/ControlReceipt/PlaybackControlPort와 request context는 아직 root 공개 SDK이므로 외부 plugin 패키지에 복제하거나 root로 역의존하지 않는다. 공유 공개 경로는 SDK 후속 checkpoint에서 인계받는다.

Meari 담당의 draft 인계는 `packages/meari/.spec/sots/integration/plugin-sdk.md`와 ADR0013이다. PluginManifest/CapabilityDescriptor/SchemaId, PluginRegistry register/dispatch/cancel/poll_events/unregister/rebind_scope, PluginAdapter Immediate/Pending이 예정돼 있으나 아직 공개 구현이 아니다. 제품의 picker adapter부터 canonical port로 준비하고, registry/lifecycle은 이 인계 API에 연결한다.

## 식별과 협상

제품 plugin의 안정 ID 후보는 `com.kinekino.player.ui-host`, wire protocol 후보는 `1`이다. 표시 제품명·경로·윈도 제목은 ID에 포함하지 않는다. 버전 세 축을 별도로 기록한다: plugin package SemVer, Meari plugin SDK version, 제품 SDK API version(현재 root `API_VERSION = "1"`). 최초 구현은 각각 정확 버전으로 고정하고 호환 시험 전 범위 호환을 주장하지 않는다.

manifest는 plugin ID/version, protocol version, 요구 Meari SDK version, target/transport, capability별 input/output/event schema version과 hash, typed configuration, 필요 permission, asset/artifact 범위, 수명 정책을 선언한다. registry가 ID 중복·미지원 protocol·schema 불일치·target 불일치를 등록 전에 거절한다. 빌드 성공만으로 capability를 available로 만들지 않는다.

설정은 host assembly에서 주입한 SDK handle, target 지원, 허용 resource grant, queue/byte/event 예산이다. 임의 DLL/JS 경로·실행 프로그램·외부 endpoint를 TSX props로 설정하지 않는다. 권한은 `window.control`, `file.choose`, `sdk.subtitles.convert`, `sdk.playback.control`, `sdk.playback.observe`처럼 capability별로 분리한다. manifest 요청은 허가 자체가 아니며 host grant와 실제 adapter 가용성을 모두 만족해야 호출한다.

## Capability와 target 행렬

아래 `예정`은 구현 증거가 아니다. Native는 Windows 제품 host, Web은 browser WASM을 뜻한다. Web에서도 동일 입력/결과 의미를 유지하고 불가능한 기능을 `Unsupported`로 돌려준다.

| Capability | 공통 입력과 결과 의미 | Native adapter | Web adapter |
| --- | --- | --- | --- |
| `subtitle.path.open` | 목적/허용 format → 선택 경로 또는 명시 취소. 파일을 읽거나 변환하지 않음 | 기존 `SubtitlePathPicker` GUI-thread 호출 연결 예정 | OS 절대 경로 선택은 Unsupported. 향후 File/Blob import는 별도 resource capability로 설계 |
| `subtitle.path.save` | SRT/VTT format → 저장 후보 경로 또는 취소. 저장 성공 영수증이 아님 | 같은 picker의 save 연결 예정 | 동일 경로 의미를 제공 못하면 Unsupported. download를 path 저장 성공으로 바꾸지 않음 |
| `subtitle.convert` | 기존 `ConvertSubtitles` → `ConversionResult` / 기존 `ApiError` | 공유 host/probe에서 실 SDK·파일 전달 검증, 제품 root 등록 후속 | 인증·resource grant를 갖춘 같은 SDK transport가 없으면 Unsupported. browser 전용 parser 복제 금지 |
| `playback.control` | 기존 play/pause/seek/volume DTO → `ControlReceipt` | 공통 SDK control port 연결 예정 | 같은 SDK transport가 없으면 Unsupported. HTML video 실행을 native 제어 완료로 표시하지 않음 |
| `playback.observe` | engine/source identity → 관측 snapshot/event | SDK status 관측 연결 예정 | 같은 SDK transport가 제공한 관측만 표시. 값 추정 금지 |
| `window.control` | minimize/maximize/restore/close intent → host 접수/실제 관측 구별 | OS adapter 연결 예정 | browser 창 최소화·최대화·종료는 Unsupported. fullscreen은 별도 capability이며 대체 성공 아님 |

웹 네이티브 파일 import/export와 미디어 surface가 필요하면 resource/stream 계약을 추가한다. 기존 path DTO의 뜻을 browser용으로 바꾸지 않는다. 모든13화면군/하위탭과 키보드·공통 컴포넌트 범위는 이 첫 capability 목록으로 축소되지 않는다.

## 호출·오류·취소

첫 수직 경로의 request context/SDK closure/root 의존성과 Native/Web 조립은 [자막 변환 host 계약](plugin-conversion-host.md)으로 분리한다. codec 시험 통과와 실제 host 조립을 구별한다.

Meari 공통 envelope가 document ID/epoch, plugin instance/generation, request ID, capability/schema version을 운반한다. 제품 payload는 기존 SDK DTO를 그대로 보존한다. SDK request context의 expected revision/idempotency/source identity를 plugin에서 삭제·재생성하지 않는다. SDK 오류의 code/details/remediation/request ID를 손실 없이 표시하고 plugin 등록/dispatch 오류와 구별한다.

지원 없음은 `Unsupported`, 입력 추가 필요는 `NeedsInput`, 잘못된 payload는 `InvalidArgument`, permission 거부는 `PermissionDenied`다. stale document 결과는 새 문서에 적용하지 않고 lifecycle 진단으로 남긴다. SDK source 만료는 기존 `SourceExpired`를 유지한다. path picker의 `Ok(None)`은 사용자의 취소이며 export 성공/빈 경로/엔진 오류로 변환하지 않는다.

재생의 `accepted`/`completion: unobserved`는 실제 실행 확인이 아니다. 자막 변환 loss는 SDK가 반환한 목록으로 확인하고 사용자가 허용한 현재 입력에만 `allow_loss`를 전달한다. 입력/format/speaker 변경 후 이전 동의를 재사용하지 않는다. plugin이 자체 loss 판정·CAS·자동 채택을 수행하지 않는다.

비동기 dispatch는 bounded queue와 terminal 결과 하나를 갖는다. 취소 요청 접수·실제 취소·이미 끝난 상태를 구별하고, SDK에 취소 API가 없는 동기 작업을 취소됐다고 표시하지 않는다. UI 구독 해제는 작업 취소와 다르다. 파일 publication이 끝난 뒤 화면이 닫혔으면 SDK 결과를 보존하되 새 문서에 성공 토스트를 투영하지 않는다. worker thread를 강제로 중단하거나 사용자 player를 자동 종료하지 않는다.

## 등록·수명·HMR·자산

host 시작 시 manifest 검증 → 권한/adapter 협상 → instance 등록 → document binding 순서다. unmount/close/reload는 신규 dispatch 차단 → listener 해제 → pending 결과의 generation 차단 → SDK가 지원하는 취소 요청 → handle 정리 순서다. cleanup은 반복 호출 가능해야 하며 다른 instance의 미디어/작업을 종료하지 않는다.

TSX/CSS HMR은 plugin instance와 SDK 작업을 무조건 재생성하지 않는다. 동일 계약의 presentation 교체만 허용하고 binding은 document epoch로 재연결한다. manifest/schema/native binary 변경은 재협상·정상 재시작 대상으로 분리한다. in-flight mutating command를 HMR에서 자동 replay하지 않는다. panic/transport 단절은 해당 instance를 실패 상태로 전환하고 명시 복구 전 stale callback을 거절한다.

plugin manifest에 없는 asset/file 접근은 허용하지 않는다. icon/font는 Meari catalog/resolver를 사용하고 제품 전용 SVG만 제품 자산으로 등록한다. artifact 출력은 SDK가 발행한 receipt/handle을 사용하며 plugin이 임의 저장 루트를 새로 결정하지 않는다. Web asset URL과 Native font/SVG loader의 실제 공급은 별도 consumer 검증 대상이다.

## 인수와 증거

최신 [실 SDK bridge r1](../../../verification/presentation/sdk-plugin-bridge-2026-10-19.md)은 공유 제품 host/실 ApplicationClient/SubtitleFiles의4시험과 고정 CLI의 새 파일0·충돌5·손실동의필요3을 확인했다. 원본·충돌출력·root lock SHA는 전후 동일하다. 아래 r2에서 남았던 ApplicationClient/파일 연결은 이 probe 범위에서 입증됐으며 제품 root 등록·source IR·Native GUI·Web 권한/브라우저·시각은 남아 있다.

`packages/player-ui-plugins`의 codec은 canonical DTO→host closure→canonical result만 전달한다. [r1](../../../verification/presentation/product-plugin-codec-2026-10-19.md)의 codec3회귀 뒤 공개 서명을 인수해 실제 PluginAdapter를 구현했다. [r2](../../../verification/presentation/product-plugin-registry-2026-10-19.md)에서 fmt/test6/strictClippy/WASM library check가 모두 exit0이었다. Native/Web target 설정별 registry의 메모리 변환 성공·정본 SDK 오류·grant/schema 거부·완료 cancel/unregister 경계를 검사했다. no-transport는 Unsupported이며 SDK에 없는 취소·진행률을 만들지 않는다. root ApplicationClient/RequestContext 조립·실제 OS publication·Web browser/권한·source binding은 남아 있다. hash pin은 현재 host 주입 metadata이며 진위 검증이나 OS 권한 승인으로 확대하지 않는다.

첫 CLI 계약 검사는 실제 Meari registry를 사용하여 정상·중복 ID·버전/schema/target 불일치·permission 거부·지원 없음·취소·stale 결과·중복 cleanup·event 순서/예산을 검사한다. adapter fake는 계약 시험에만 쓰고 지원 판정은 실제 SDK adapter 결과와 구별한다. SDK의 실제 convert 결과/오류/새 파일 receipt, playback 접수/관측, picker 취소를 각기 보존한다.

Native/Web 소비 smoke는 같은 capability/schema/request를 사용하고 지원 행렬과 결과 의미를 비교한다. Web unsupported도 명시 결과를 검사하며 지원 구현으로 세지 않는다. GUI 고유 focus/keyboard/권한 gesture/수명만 중앙 슬롯에서 검사한다. 완료 표에는 target·Meari revision·plugin hash·SDK version·명령·exit·artifact·미검증을 기록한다. 아직 이 plugin의 실행 증거는 없다.

## 적용한 설계 근거

`architecture`의 trade-off/ADR 기준과 modoo.today `modootoday-free/design-delivery-plus/frontend-first-journey` 본문을 읽었다. 후자의 typed port·동일 adapter 계약·실제 연결 전 wiring 상태 구분·서버 confirm·허위 수치 금지를 이 계약에 적용했다. 프런트엔드 우선 작업 순서, mock을 제품 성공으로 사용하는 방식, HTTP 전용 오류/390px 기준은 이 프로젝트의 Rust SDK/CLI 선행 및 승인 시안 규칙으로 대체한다. 라이선스/빌드/화면 합격 근거를 이 스킬로 대신하지 않는다.
