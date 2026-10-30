---
id: evidence:kinekino/20261029113524
title: VM 대체 가상 디스플레이와 Rust 드라이버 검토
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# VM 대체 가상 디스플레이와 Rust 드라이버 검토

최신 후속: [앱 실제 DPI 매트릭스](sot-016-app-dpi-matrix.md)에서 이전에 미확정이던 앱 HWND96/144/192·네이티브 전체화면·컨트롤·회차 패널의 정해진 표본을 검증했다. 아래의 ‘앱 미검증’은 첫 드라이버 시험 시점의 이력이며 최신 완료 범위와 제한은 후속 정본이 우선한다.

## 사용자 변경 지시

VM 설치 실패로 VM 없이 가상 디스플레이 드라이버 또는 적합한 Rust 오픈소스를 이용해 DPI 검증을 재개한다. 이전 VM 이미지·Windows ISO는 정리 대상이다. 이전 DPI/VM 보류는 이 새 경로에 한해 갱신하며 VM 설치 복구를 재개하지 않는다. ChatGPT/Terminal은 유지하고 모든 자체 개발 코드는 Rust/단일책임/계층 구조를 따른다.

## 후보 근거

[MolotovCherry/virtual-display-rs](https://github.com/MolotovCherry/virtual-display-rs)는 Rust Windows 가상 디스플레이 드라이버이며 Windows10 2004+ x64·복수 모니터/해상도/주사율을 문서화한다. [릴리스](https://github.com/MolotovCherry/virtual-display-rs/releases)의 최신 GitHub API 결과는 v0.3.1/2023-10-22다. 오래된 릴리스라는 사실과 최신 Windows11 호환 실측은 구별한다. 공개 사용 사례/코드 존재가 현 호스트 검증 완료를 뜻하지 않는다.

[VirtualDrivers/Virtual-Display-Driver](https://github.com/VirtualDrivers/Virtual-Display-Driver)는 서명·EDID·HDR 등의 외부 드라이버 후보이며 자체 Rust 드라이버가 아니다. 이전에 받은 25.7.23 x86/VDD 패키지의 cat Authenticode는 Valid/SignPath Foundation을 관측했다. 이것만으로 Windows의 해당 드라이버 설치/로드 성공이나 Microsoft WHQL 인증을 확정하지 않는다. 새 자체 제어·측정 코드는 Rust로 작성해야 한다.

## Rust v0.3.1 실제 파일 검토

공식 portable ZIP만 `runtime/display-driver-review`에 다운로드/압축 해제했다. 배포본·제어 앱·설치 스크립트를 실행하지 않았다. ZIP SHA256: `246cb55f98ff7ff2663384abcf17ac8ff4b1bf90895c110e12ccf8695ff49687`. 릴리스 API digest가 없어 독립 릴리스 digest 대조는 불가하며 로컬 보존 지문이다.

- driver DLL/cat의 발급자·주체: `CN=DriverCertificate`(자체 서명).
- 서명 인증서 지문 SHA1: `5DFA7E68997ED52D4B10420407860D1C29B6F427`.
- 만료: 2040-01-01 08:59:59 KST.
- Windows 서명 확인 메시지: 신뢰되지 않는 루트에서 인증서 체인이 중지됨.
- control.exe는 디지털 서명 없음.
- 설치 안내/배치 파일은 인증서를 LocalMachine Root와 TrustedPublisher에 추가한다. 호스트가 그 인증서를 새로 신뢰하게 되는 변경이다.

사용자는 ‘인증서·서명을 검토한 뒤 변경 여부 결정’을 선택했다. 따라서 인증서 저장소·드라이버·레지스트리·모니터·배율은 아직 변경하지 않았다. 검토 raw 자료는 runtime/display-driver-review/signatures.json 및 certificate-review.json. 실행 정책/서명 검증·Secure Boot·메모리 무결성·테스트서명 모드를 변경하지 않는다. 신뢰 저장소 추가는 사용자 별도 결정 후에만 가능하다.

## 실제 DPI 검증 계약

후속 사용자 승인: 지문 `5DFA7E68997ED52D4B10420407860D1C29B6F427`의 인증서 등록을 명시적으로 허용했다. 동일 지문 확인 후 Root·TrustedPublisher에 각각 추가하고 양 저장소의 존재 1건씩을 확인했다. 2026-10-18 08:57 KST. 이후 DLL Authenticode가 Valid로 바뀌었다. 이것은 신뢰 등록 결과이며 독립적인 드라이버 안전성/현 호스트 호환 인증을 뜻하지 않는다. 드라이버/모니터/배율은 아직 설치·변경하지 않았다. 기록: runtime/display-driver-review/certificate-registration.json. 사용 가능한 장치 설치 도구 devcon은 Microsoft Corporation의 Valid 서명을 확인했다.

선택한 드라이버의 호환/설치가 확인된 뒤 가상 모니터 하나만 확장으로 만들고 물리 주 모니터/기존 앱 위치를 유지한다. Rust Windows 어댑터로 디스플레이 ID/영역·앱 HWND·실제 GetDpiForWindow 값을 기록하고 테스트 앱만 가상 모니터로 이동한다. 100/150/200%에서 실제 DPI96/144/192를 확인해야 해당 행을 완료로 판정한다. 픽셀 해상도/egui zoom/프로그램 내부 배율만 바꿔 실제 DPI로 보고하지 않는다.

회차 패널·제목·시간·진행바·웹/mpv 경계·네이티브 컨트롤/포커스·전체화면 자동숨김을 실제 조작하고 창/모니터 이동 시 잘림/클릭 좌표를 검증한다. 정책은 domain, 매트릭스/인수는 application, Win32/드라이버/화면 관측은 infrastructure, 표시/입력은 presentation에 둔다. 설치/서명·복원 정책을 UI 렌더러에 섞지 않는다. 종료 시 생성한 가상 모니터/설정만 복원한다. 현재는 설치 전 검토이며 DPI 시험 완료가 아니다.

## 실제 설치·DPI 관측과 복원

후속 승인에 따라 인증서 등록 후 Rust 어댑터가 DLL/cat·Microsoft 서명 devcon의 해시를 검증하고 드라이버를 설치했다. VirtualDisplay 상태 OK, ROOT\\DISPLAY\\0000을 관측했다. 미서명 control.exe·배포 배치·install.reg는 실행하지 않았다. 이전 절의 ‘미설치’는 승인 전 검토 시점의 기록이며 현재 상태는 본 절이 우선한다.

Rust named-pipe 어댑터로 소유 모니터 ID9 하나를 추가했다. 드라이버의 최대10개 계약에 맞춰 ID를 정했다. 실제 PMv2 숨김 시험 HWND의 GetDpiForWindow 관측:

| 가상 화면 설정 | 실제 DPI | 물리 주 모니터 |
|---|---|---|
| 1920×1080,100% | 96 | 3440×1440,96 DPI |
| 1920×1080,150% | 144 | 동일 |
| 3840×2160,200% | 192 | 동일 |
| 복원 후100% | 96 | 동일 |

1920×1080에서는 Windows 배율 선택지가175%까지여서 가상 화면만4K로 바꿔200%를 측정했다. 화면의 픽셀 크기나 egui zoom으로 대체한 값이 아니다. 다만 플레이어 HWND 자체의 DPI/이동 도착·잘림/클릭/전체화면 자동숨김 매트릭스는 확정하지 못했다. [이동 시도 화면](../../../evidence/display/app/dpi-150-window.jpg)은 참고 관측이며150% 앱 통과 증거로 사용하지 않는다. 새 빌드에서 정상 카테고리→작품→회차→웹/mpv 전환을 확인했으나 전체 고배율 재생 통과를 뜻하지 않는다.

배율을100%로 복원하고 소유 앱/설정 창 정상 종료, Remove ID9만 수행했다. 최종 관측은 물리 DISPLAY1/3440×1440/96 DPI 한 개로 복귀했다. 드라이버와 승인된 인증서는 설치 상태로 남아 있다. 설치/추가/144/192/복원 raw 기록은 runtime/display-driver-review에 보존하고 [보존한 관측 자료](../../../evidence/display/driver)에서도 확인한다. Secure Boot·메모리 무결성·테스트서명 설정은 바꾸지 않았다.

## 실패 VM 정리 상태

읽기 전용 확인: Source-DPI-Lab은 Off, 경로는 프로젝트 runtime/validation-vm/Source-DPI-Lab, 디스크 os.vhdx와 평가판 ISO가 연결돼 있다. 정리 대상은 해당 VM 등록/디스크·설정 디렉토리, media/windows11-enterprise-eval-ko-kr.iso와 -noprompt.iso다. 다른 VM/WSL·서비스는 대상이 아니다.

경로와 Off 상태를 검사한 뒤 VM 등록·파일·ISO를 일괄 삭제하는 명령을 요청했으나 자동 승인 검토가 실행 전에 거부했다. 구체적 사유는 제공되지 않았다. 동일 삭제를 반복하거나 다른 방식으로 우회하지 않았다. **삭제되지 않았으며 정리 잔여 작업**이다. 인증서 검토와 이 차단을 혼동하지 않는다.
