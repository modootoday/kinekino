---
id: sot:kinekino/native-ui
title: Native 영상·UI 합성 조사
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Native 영상·UI 합성 조사

2026-10-20. 읽기 전용 조사다. 실제 합성·OS 표시·성능·Web 동등성은 검증하지 않았다. 엔진이나 renderer 교체 결정이 아니다. [후속 spike](../../../../plans/player/native-video-composition-spike.md), [현재 child 조립](../../../../plans/player/native-playback-child-assembly.md)을 참조한다.

## 관측과 계약

제품 VideoSurface는 STATIC child HWND와 SetWindowPos/SWP_NOZORDER·ShowWindow를 사용한다. 조상 clip·CSS alpha 합성 포트는 없으며 기존 handle 캐스트와 Destroy 결과 관측도 새 경로에서 보완해야 한다. Meari NativeWindowDriver는 VelloCpuWindowRenderer/softbuffer에 결합돼 있다. 별도 GlHost/Skia GPU paint 코드는 존재하지만 production window의 GL 합성·present 성공을 입증하지는 않는다.

NativeParentLease는 borrowed parent 권한이며 특정 영상 slot의 paint 순서·합성 권한이 아니다. window geometry나 viewport 사각형만으로 실제 clip·transform·overlay를 표현했다고 보지 않는다. 최신 계약 초안과 포트 audit는 runtime/development/meari-p2-predicate/.spec/{sots/frontends/native-surface-slot.md,evidence/verification/native-surface-port-audit-2026-10-20.md}에 있으며 PM이 내용을 읽었다. 해당 계약은 미구현 상태다.

독립 child window와 부모 paint의 관계는 창 단위 clipping·Z order를 따른다. 사각형 영역을 비워 불투명 UI를 표시하는 것과 영상 위 반투명 UI·그림자·둥근 모서리를 합성하는 것은 다르다. [Microsoft Window Features](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features), [Window Regions](https://learn.microsoft.com/en-us/windows/win32/gdi/window-regions).

## 후보와 미검증

| 후보 | 재사용·추가 책임 | 현재 한계 |
|---|---|---|
| 외부 mpv child | 기존 IPC·trusted executable·broker/cleanup 재사용 | 별도 overlay/compositor 없이는 승인 디자인의 alpha overlap 입증 불가 |
| libmpv OpenGL FBO | 기존 GL/Skia 자산 재사용 후보, 동일 합성 대상의 UI paint | production renderer/present·context 소유·상태 복원·render wake·device loss 미구현 |
| libmpv software buffer | CPU scene에 정확한 paint 순서로 삽입하는 정확성 후보 | stride/format/frame 소유·색 변환 필요, 실시간 성능/HDR 미검증 |
| libmpv D3D11 texture | Windows 합성과 caller-owned texture 후보 | 현재 고정 배포물의 API 지원과 device 동기화·색 공간·present 검증 필요 |

고정 v0.40.0 render API는 OpenGL FBO와 software memory 계약을 제공한다. 조사 시점 upstream master에는 D3D11 header도 있었으나 mutable upstream의 존재를 현재 배포물 지원으로 전이하지 않는다. 실제 채택 시 commit·ABI·DLL·라이선스·해시를 별도로 고정한다. [v0.40.0 render.h](https://github.com/mpv-player/mpv/blob/v0.40.0/include/mpv/render.h), [OpenGL header](https://github.com/mpv-player/mpv/blob/v0.40.0/include/mpv/render_gl.h), [upstream D3D11 header](https://github.com/mpv-player/mpv/blob/master/include/mpv/render_d3d11.h).

소스 가능성이나 SW 경로의 일반적 비용 설명은 이 호스트의 실측 결과가 아니다. renderer·실행 어댑터를 바꾸더라도 공통 SDK/application 정책·CLI 계약을 복제하지 않는다. whole-parent 영상·별도 mpv 창·임의 영상 숨김을 최종 디자인의 대체로 채택하지 않는다.
