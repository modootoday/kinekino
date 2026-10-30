---
id: plan:kinekino/20261029113540
title: Native 영상·UI 합성 후속 spike
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native 영상·UI 합성 후속 spike

상태: 대기열 설계. 실행·새 공급물 다운로드·renderer 변경은 배정하지 않았다. [조사 정본](../../sots/player/infrastructure/playback/native-composition-research.md), [NativeChild 조립](native-playback-child-assembly.md)을 따른다. 현재 Inspector 소비·cleanup·navigation gate를 먼저 마감한다.

1. UI 담당이 승인 원본에서 영상 위 반투명 transport·검사기/팝업·clip/scroll·focus/pointer 요구의 최소 묶음을 고정한다. 축소한 별도 창이나 영상 숨김으로 대체하지 않는다.
2. Meari 담당은 현재 GlHost가 같은 document scene과 production window present로 연결될 수 있는 공개 포트를 대조한다. slot 식별·layout revision·clip·paint order의 계약과 renderer 능력을 구분한다.
3. SDK 담당은 재사용 가능한 고정 libmpv 공급물이 있는지 확인한다. 채택한다면 revision·ABI·파일 해시·라이선스와 Rust 어댑터를 명시한다. 기존 mpv.exe 검증을 DLL 검증으로 승계하지 않는다.
4. 중앙의 별도 단일 슬롯에서 Rust 최소 GL 수직 spike를 우선 검토한다. GPU 경로의 비용이 범위를 넘으면 SW 합성의 정확성 spike나 확인된 D3D11 API와 비교한다. 새 backend 선택은 그 결과 뒤 결정한다.
5. 같은 출력의 영상+UI alpha/clip/order 픽셀, pause/seek redraw·resize·stale generation 거부·last-good·worker/context/SDK 종료를 확인한다. frame time·drop·CPU·메모리·A/V sync는 고정 미디어와 예산에서 실제 측정한다.

headless scene/CLI trace와 실제 Windows 표시·입력·Destroy는 다른 증거다. Web 동등성·HDR·전체 시각 인수도 별도다. 현재 외부 child의 수명 기반은 유지하며 새 합성 경로 조사 때문에 검증된 cleanup을 폐기하지 않는다.
