---
id: plan:kinekino/20261029113745
title: 네이티브 영상과 공통 UI 합성 비교
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 네이티브 영상과 공통 UI 합성 비교

2026-10-20. 비교 계획이며 backend 변경·성능 우위·사용자 시각 승인이 아니다. 기존 ADR002의 texture 재검토 조건과 ADR005의 단일 창 요구를 구체화한다. ADR018의 기존 egui/Glow 제안은 현재 Meari 제품 renderer 채택 증거와 구분한다.

## 요구와 현재 관측

동일 영상 위 반투명 컨트롤, OP/ED 버튼, 편집 자막, 전체화면 자동숨김, Tab·pointer·접근성·캡처가 공통 상태와 함께 동작해야 한다. HWND 호환 경로만 완성해 이 요구를 대체하지 않는다.

root Meari494 및 c21 합성 소스의 NativeWindowDriver는 VelloCpuWindowRenderer/winit/softbuffer다. checked Scene의 CPU 제출은 영상 texture 합성이나 실제 표시 ACK가 아니다. SDK 현재 process/IPC와 --wid 경로는 호환 재생 후보로 유지한다. libmpv Render API와 공통 compositor는 별도 검증 대상이다.

## 비교 후보

| 후보 | 이점 | 확인할 비용·한계 |
|---|---|---|
| mpv process와 child HWND | 기존 IPC·실행 격리 재사용 | parent DOM의 안정적인 alpha overlay를 보증하지 않음, mpv 자체 child의 input/수명 추가 확인 |
| libmpv SW와 CPU Scene | 현재 renderer와 데이터 경계가 가까움 | 공식 단일 스레드 변환·스케일·OSD 비용 경고, 기본 선택이나 성능 우위로 확정하지 않음 |
| libmpv OpenGL FBO와 공통 합성 | 동일 렌더 target 합성 후보 | 실제 context·GL 상태·device loss·배포 ABI와 기존 renderer 전환 검증 |
| libmpv D3D11 texture와 공통 합성 | Windows 중심 후보 | 공식 master API 존재와 실제 고정 배포 binary/header/binding 지원을 구분, 같은 device·동기화 검증 |
| wgpu와 외부 texture 상호운용 | backend 중립 확장 후보 | 직접 mpv wgpu API는 확인되지 않음, 외부 texture·fence·색공간·복사 비용 검증 |

## 권한과 소유 단계

1. 원 document/node/window/slot generation, parent lease, 생성 thread를 확인한다.
2. 원 borrowed parent에서 배치·표시·route·미디어 open 권한 없는 owned hidden probe만 만든다. Unknown을 Supported로 바꾸지 않는다.
3. 실제 paint·geometry와 OS parent/style/thread/rect/input 관측을 원 owner에 결합한다.
4. fresh slot과 배치·합성 조건을 다시 확인한 뒤 표시를 허용한다.
5. 같은 owner/generation의 ready 증거 뒤에만 재생을 활성화한다.

완전한 display gate를 hidden probe 생성 전 조건으로 사용해 관측 증거를 만들 수 없는 순환을 피한다. 각 단계 실패는 원 lease·slot·부분 생성 자원을 반환한다. HWND의 engine 종료→surface 정리 순서를 libmpv render context/core 수명에 그대로 복사하지 않는다. WndProc나 render callback에서 SDK 정책·join을 재진입하지 않는다. 원 thread/generation의 WM_NCDESTROY와 단순 IsWindow(false)를 구분한다.

## 다음 검증과 담당

- SDK는 기존 고정 mpv/libmpv 산출물·header·exports·Rust binding을 읽기 확인하고 버전/해시를 고정한다. master 문서만으로 제품 지원을 판정하지 않는다.
- Meari는 mpv 제어 정책을 포함하지 않는 opaque frame lease/Scene 삽입 계약과 실제 renderer 확장 비용을 제시한다.
- 제품 plugin은 frame·slot·paint generation과 overlay 상태를 연결하고, stale/hidden/폐기 후 옛 frame이 재게시되지 않도록 한다.
- 동일 기존 로컬 미디어·크기·자막·overlay 조건에서 CPU/GPU·메모리·dropped frame·A/V sync·입력 latency·색공간·종료 실패를 비교한다. 미지원 계측은 미측정으로 남긴다.
- GUI/OS 실측은 별도 중앙 배정 뒤 수행한다. 현재 단계는 리서치/소스 경계이며 새 다운로드·컴파일·GUI·OS 비교 결과가 없다.

## 근거

- [ADR002](../../adrs/player/adr-002-native-playback.md), [ADR005](../../adrs/player/adr-005-native-first-production-workspace.md), [ADR018](../../adrs/player/adr-018-gpu-canvas-rendering.md).
- [mpv Render API](https://raw.githubusercontent.com/mpv-player/mpv/master/include/mpv/render.h), [OpenGL](https://raw.githubusercontent.com/mpv-player/mpv/master/include/mpv/render_gl.h), [D3D11](https://raw.githubusercontent.com/mpv-player/mpv/master/include/mpv/render_d3d11.h), [--wid](https://mpv.io/manual/master/#options-wid). master 기준 조사이며 고정 제품 지원은 별도다.
- [Windows 창 구조](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features), [CreateWindowExW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw), [DestroyWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroywindow), [WM_NCDESTROY](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-ncdestroy).

## 2026-10-20 로컬 공급물 읽기 관측

기존 공급물은 runtime/mpv/mpv.exe 121051136 bytes, PE v0.41.0-1200-gdde067107, SHA256 F8BF7E8FD8D6696E35441C55091498060E4594F724A132FD6C46579341BC0790다. 기록된 shinchiro 20261017 release의 mpv-x86_64-20261017-git-dde0671072.7z는 33999565 bytes, SHA256 2821D9EEF879215E99975FAE9616E1777F5E3747168EDD3A571C41E21B8ABBBC다. 이는 subprocess 공급 바이트의 근거이며 전체 source/build dependency closure나 게시자 서명·재배포 의무 충족 증거는 아니다.

확인한 runtime/mpv root/doc/installer 목록에는 libmpv DLL·개발 header가 없었고 root Cargo에는 mpv FFI crate가 없었다. 호스트 전체 DLL 부재를 뜻하지 않는다. 현재 libmpv GL/SW/D3D11 API/ABI 지원은 미확정이다. 실제 후보의 DLL·대응 정확 header/revision·arch·exports·동적 의존·빌드 옵션·라이선스/NOTICE를 기존 subprocess manifest와 분리해 고정한 후 중앙 배정된 실행 검증이 필요하다. 이번 읽기 관측은 다운로드·컴파일·실행0이다.
