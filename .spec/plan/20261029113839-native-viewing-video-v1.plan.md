---
id: plan:kinekino/20261029113839
title: V1 Viewing video 선언·동일 문서 계약 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# V1 Viewing video 선언·동일 문서 계약 제안

2026-10-21. PM의 source 배정 전 읽기 계획. 코드·Cargo·OS 변경0. UI host7 채택 commit `6e732c206a4839f2c26917782a527161e4eb035a` 기준이다. [중앙 체크포인트](native-main-integration-checkpoints.md)의 V1만 대상으로 한다.

## 현재 관측

`viewing/player.tsx`는 WindowChrome/ViewingStart/TransportBar를 조합하며 video가 없다. read-panel은 같은 ViewingPlayer를 한 번 포함한다. 고정 document inventory는 player/start-screen/chrome/transport 네 소스이며 read-panel opt-in 시 하나를 추가한다. 스타일은 prepare_viewing_styles가 tokens/components/viewing 세 소스를 실제 assets와 함께 읽는다.

Meari markup::build는 명시 id를 실제 Element.id로 보존하고 중복/빈 id를 거절한다. NativeSurfaceRegistrar.reserve_video는 원 handle의 전체 HostStamp와 Ready window를 검사한 후 같은 실제 document.elements에서 **id 일치 + dom_tag video**를 요구한다. renderer node/renderer identity/DOM document identity는 실제 registrar가 내부에서 얻는다. V1은 이 private authority를 만들거나 NodeId를 복사하지 않는다.

mount_prepared_shell_impl은 원 namespace/epoch/source_generation과 실제 callback export로 InteractiveDocument를 만들고 같은 source_generation으로 NativeDocumentHandle을 만든다. 오류 시 외부 SDK/host owner와 pending ticket은 DocumentFailure에 반환된다. 새 문서 검증도 이 원 오류 경로를 유지해야 한다.

## 제안 exact9

| 경로 | V1 책임 |
|---|---|
| `src/presentation/web_ui/viewing/player.tsx` | 기존 중간 ViewingStart 영역에 stage wrapper와 단일 src 없는 video 선언. catalog에서 온 semantic ID prop을 id에 사용. 시작 콘텐츠와 chrome/transport 순서 유지 |
| `src/presentation/web_ui/viewing/read-panel.tsx` | video ID prop을 같은 ViewingPlayer로 한 번 전달. 두 번째 video 생성0 |
| `src/presentation/web_ui/styles/viewing.css` | 중간 stage의 기존 공간 안에서 video와 시작 콘텐츠를 겹치게 배치. 기존 공통 배경/spacing 토큰 재사용, 외곽 크기와 chrome/transport 스타일 변경0 |
| `src/presentation/web_ui_adapter/player_shell/document.rs` | 책임별 `video` 하위 모듈 등록 및 base_props에서 고정 typed ID를 한 번 주입. read_props는 이를 상속 |
| `src/presentation/web_ui_adapter/player_shell/document/video.rs` 신규 | 닫힌 enum `ViewingVideoId::Primary`와 단일 as_str 매핑. 임의 문자열 파서/selector 입력 없음. Meari Native/SDK/OS 의존0 |
| `src/presentation/web_ui_adapter/player_shell/preparation_tests.rs` | default/read-panel 두 graph의 ID prop 전달과 동일 catalog 사용 CPU 회귀 |
| `src/assembly/meari/player_shell_document.rs` | Native 전용 video 하위 모듈 등록, 실제 handle 생성 후 같은 mount 안에서 선언 검사. 오류 stage를 명시하고 원 DocumentFailure owner 반환 보존 |
| `src/assembly/meari/player_shell_document/video.rs` 신규 | 실제 handle 요소를 읽어 정확 ID·video tag·전체 video 단일성을 확인하고 원 full HostStamp를 담은 owned metadata 반환. native surface lease/proof 생성0 |
| `src/assembly/meari/player_shell_document_tests.rs` | 실제 CPU mount·잘못된 선언·문서/source 세대와 원 owner 보존 회귀 |

현재 고정 inventory4/opt-in5를 늘리지 않는 최소 안이다. 새로운 JSX 소스나 공개 범용 component API를 추가하지 않는다. 기존 ViewingPlayer가 제품 공용 video 영역을 소유하고 start-screen은 기존 시작 콘텐츠로 유지한다. 나중 복수 화면에서 공통 video primitive가 실제 필요해지면 별도 추출한다.

## 선언과 metadata 계약

catalog 예시 의미 문자열은 `aw-viewing-primary-video`이며 최종 이름은 V1 배정 시 고정한다. prop은 application request/브라우저 메시지/사용자 문자열에서 받지 않는다. default/read-panel은 같은 base_props 카탈로그를 사용한다. TSX id 문자열 하드코딩을 두 군데 만들지 않는다.

video에는 src/srcObject/poster/autoplay/controls/media event/SDK callback을 주지 않는다. preload none, tabIndex -1, aria-hidden으로 비활성 placeholder 의미를 명시한다. 브라우저나 Native에서 영상 재생·native child/등록/visible 권한을 만들지 않는다. CSS의 정확 문법 지원은 실제 CPU document/style mount로 확인하되 시각 일치 통과로 부르지 않는다. 구조용 0/100% 외 새 임의 palette/spacing/radius 토큰을 만들 필요가 없다.

Native 검사 결과 `MountedViewingVideo`는 private 필드의 typed ID와 origin HostStamp만 담은 **과거 mount metadata**다. 임의 생성자/문자열 수신/lease/raw HWND/NodeId는 노출하지 않는다. 같은 NativeDocumentHandle을 검사한 동기 read 안에서 실제 stamp를 얻고, 거기에 사용된 source/definition/document/host identity를 보존한다. 검증 중 외부 host/SDK callback/dispatch/reproject를 호출하지 않는다.

다른 문서에 같은 ID가 있어도 결과를 교환할 수 없다. mount 후 reproject/turn/source 변경 시 metadata는 fresh authority가 아니다. V2가 request를 만들 때 **그 시점의 원 handle을 다시 검사하여** 현재 full stamp를 얻고, 원 consumer bind가 이를 실제 preparation document stamp와 다시 비교해야 한다. V1의 mount stamp를 그때까지 무조건 유효하다고 가정하지 않는다. production FrameRequest.video String→typed ID 변경은 V1에 섞지 않고 V2 계약에서 결정한다.

## CPU 인수안 — 신규7 예상

1. default 고정 graph를 실제 mount하면 primary ID의 video가 정확1, 다른 video0이고 src/controls/autoplay/callback이 없다.
2. read-panel opt-in도 같은 ID/video1이며 callback props와 기존 label 전달을 보존한다.
3. Native metadata stamp가 원 mounted handle.stamp와 전체 일치하며 source_generation/namespace epoch 경계를 보존한다. route observed/selection/SDK 작업은 생기지 않는다.
4. 같은 ID의 div는 MissingOrWrongTag로 거절한다.
5. video가 없거나 둘인 graph는 각각 명시 오류이며, duplicate id는 Meari 원 mount 진단을 보존한다. 이 병렬 case는 하나의 표 기반 시험으로 가능하다.
6. 독립 문서 두 개의 같은 typed ID는 다른 host/owner stamp이며, 첫 결과를 두 번째 문서의 current 결과로 인정하지 않는다. 실제 reproject 후 stamp 불일치도 확인한다.
7. 선언 검사 실패에도 원 SDK Arc/host Rc/pending ticket이 유지되고 새 owner/route commit/native attach가 생기지 않는다.

신규7은 작성 전 예상이며 의미 있는 실제 시험 이름으로 재산정한다. 기존 preparation/document/style 회귀와 최종 fmt/test/clippy는 source 배정 및 중앙 실행 슬롯 뒤에만 수행한다. 현재 SDK CLI A81 soleheavy/shared이므로 UI Cargo0을 유지한다.

## 727행 frame 모듈 후속 분리

분리는 필요하다. 현 파일은 public request/result 계약·Completion mailbox·generic Flow·Native registrar/proof adapter·consumer bind를 함께 포함한다. V1 exact9에는 섞지 않는다. V2에 앞서 별도 **의미 불변 추출** 체크포인트로 `frame_delivery/{contract.rs,mailbox.rs,flow.rs,native_backend.rs}`를 두고 mod 루트에는 typed consumer 조립과 좁은 재공개만 남기는 안을 권한다. 기존 tests.rs의14개와 Viewer4 이름/단언을 유지하고 새 API/상태/clock/cleanup 정책을 만들지 않는다. private 가시성은 sibling 내부로만 넓히고 NativeBackend/proof를 제품 외부에 공개하지 않는다. 추출 체크포인트는 원 필터18+영향 회귀로 단독 인수하며, V2 설치·unused 속성 제거와 합쳐 감사 범위를 키우지 않는다.

V1 구현·분리 어느 쪽도 현재 배정된 코드 수정은 아니다. PM이 exact source checkpoint를 배정한 뒤 시작한다.

## 최종 CPU gate 상태

V1 source10 최종 r4 `5F47A4CD371F1EB493C94195BAC585F2046F26AD16C0908C2B669FE2E4206F1C`는 양 all-target strict/fmt와 같은 fresh Native binary의 고유48(신규7+기존41)을 통과했다. 원 callback 시험의 도달 불가능한 mount 가정1만 실제 graph 준비 거절 경계로 수리했고 production9는 불변이다. [실제 검증 SOT](../../../sots/verification/presentation/native-viewing-video-v1-2026-10-21.md)에 관측·한계·원 실패를 기록했다. root1443/snapshot1405·원 bundle/before/mtime·library/Cargo 보호 차이0/index0, 원11terminal receipt/기록22PID 부재를 확인하고 PM·SDK에 heavy/shared를 반환했다. root 채택은 PM 책임이다. 실제 시각/OS/생산 opt-in·V2·frame 분리는 별도 범위다.
