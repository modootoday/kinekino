---
id: plan:kinekino/20261029113528
title: 가상 디스플레이 기반 격리 DPI 검증
status: draft
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# 가상 디스플레이 기반 격리 DPI 검증

## 요구사항과 현재 관측

사용자 지정: 앱 확대 배율 대체 시험이 아니라 가상 모니터 드라이버를 사용한다. 기존 운용 프로세스 영향을 줄이기 위해 필요하면 격리한다. 최상위 [AGENTS.md](../../../AGENTS.md)의 계층 규칙을 유지한다.

2026-10-18 KST 사전 점검: Windows 11 Pro 10.0.26200, NVIDIA RTX 2070 SUPER 및 HP 34f 물리 모니터 각 1개. PnP 및 Display 드라이버 저장소 목록에서 가상 디스플레이 드라이버는 관측되지 않았다. 현재 토큰은 비관리자이며 Get-VM은 권한 부족으로 거절됐다. 따라서 VM 부재로 단정하지 않는다. Hyper-V vmms/vmcompute 서비스 실행 중. WindowsSandbox.exe 없음. 드라이버 설치·디스플레이 변경·다른 앱 종료는 수행하지 않았다. 이전 검증 앱과 mpv 프로세스는 이번 점검 시 관측되지 않았다.

재현 가능한 읽기 전용 점검: [스크립트](../../../scripts/validation/inspect-dpi-environment.ps1). 로컬 결과는 runtime/verification/dpi-environment.json에 저장하며 장치 식별자는 공개 증거에 복제하지 않는다.

## 준비한 검증 경로

1. 관리 가능한 별도 Windows 11 VM을 선택하고 체크포인트 및 기존 디스플레이 구성을 기록한다. 호스트 물리 모니터 배율은 변경하지 않는다. VM 생성·드라이버 설치에는 관리자 환경이 필요하다.
2. [VirtualDrivers 공식 배포](https://github.com/VirtualDrivers/Virtual-Display-Driver)의 서명된 릴리스를 게스트에만 설치한다. 실제 선택 버전·다운로드 URL·SHA-256·서명 검증 결과를 기록한다. 현재 다운로드/설치는 하지 않았다. 테스트 서명 활성화나 보안 설정 변경은 검증 경로에 포함하지 않는다.
3. 게스트 가상 화면을 1920×1080 및 3840×2160으로 구성하고 Windows 디스플레이 배율 100/125/150/200%를 각각 적용한다. 가상 모니터 드라이버 자체가 배율 검증의 증거는 아니므로 창 DPI와 적용 배율을 함께 기록한다.
4. 각 배율에서 웹 및 내장 mpv의 경계 정렬, 컨트롤 가림·클릭 위치, 회차 목록·한글, 최소 창, 최대화, 전체화면 숨김/복원을 실제로 검증한다. 서로 다른 DPI의 가상 화면 사이 이동도 시험한다. 앱 zoom 결과로 대체하지 않는다.
5. 같은 게스트에서 회차 1↔2를 완결 전환 10회, 로딩 중 재선택 10회, 카테고리 취소 5회 시험한다. 재생/일시정지 의도·최종 선택·mpv 잔존·늦은 이벤트를 기록한다.
6. 60분 이상 길이의 극장판 표본을 탐색 없이 mpv 60분 연속 재생하고 웹은 별도 30분 연속 시험한다. 단조 시간 기반 경과와 미디어 시간, 멈춤/오류, 앱·자식 프로세스 메모리를 표본화한다. 탐색 이동을 재생 시간에 포함하지 않는다.
7. 발견 문제는 재현→책임 계층 분리→수정→동일 조건 재검증한다. fmt/test/Clippy 통과 후 SoT와 Plan을 갱신한다. 게스트 결과를 호스트 GPU/물리 모니터 전체 호환성으로 확대하지 않는다.

## 실행 전 필요한 항목

현재 환경에서는 관리자 권한이 없어 게스트 관리와 드라이버 설치를 진행할 수 없다. 사용할 Windows VM의 접근 경로 또는 관리자 권한의 작업 환경이 필요하다. 사용자 환경 제공 전에는 DPI·반복·장시간 시험을 완료로 표시하지 않는다. 실제 드라이버 설치나 격리 환경 생성은 아직 수행하지 않았다.

## 재실행 후 권한 재점검

2026-10-18 KST: 사용자 재실행 후 다시 점검했으나 실제 도구 셸의 관리자 토큰은 false, 무결성 수준은 Medium, Administrators 그룹은 deny-only였다. Get-VM 및 Get-VMHost가 계속 권한 부족으로 거절됐다. 셸의 부모 계보는 pwsh.exe → codex.exe → ChatGPT.exe → explorer.exe였다. 사용자 재실행 사실과 별개로 현재 도구 실행 경로에는 상승 권한이 전달되지 않았다. 드라이버 설치와 VM 생성은 진행하지 않았으며, 상승된 앱 프로세스에 연결된 도구 세션이 필요하다.

## 두 번째 재실행 점검 및 실행 스크립트

2026-10-18 KST: ChatGPT 전체 재실행 후에도 도구 셸 administrator=false, Medium, 관리자 그룹 deny-only 및 Get-VM 권한 오류가 유지됐다. 원인이 공유 서버 재사용인지 앱 권한 처리인지 현재 증거로 확정하지 않는다. 공식 문서는 elevated 샌드박스의 설치 권한과 명령 실행 권한을 구분한다. [관리자 실행 스크립트](../../../scripts/start-admin-validation.ps1)와 [사용법](../../../scripts/README-admin-validation.md)을 추가했다. 관리자 PowerShell에서 직접 점검 가능하며 선택적으로 공식 --no-daemon CLI를 실행한다. 에이전트 자식 셸 권한은 별도 확인하며 보장하지 않는다. 실제 관리자 실행·VM·드라이버·DPI 시험은 미검증이다.

## 관리자 PowerShell 점검 성공

2026-10-18 07:14 KST: 사용자가 직접 실행한 관리자 점검 파일에서 administrator=true, hyperV=available, machines=[] 확인. Desktop의 도구 셸 상승과는 별개이며, Desktop 대화를 유지하고 필요한 관리자 작업만 검토 가능한 스크립트로 실행하는 경로를 사용한다. 기존 관리자 접근 부재는 이 수동 실행 경로에서 해소됐다. 설치된 VM은 없으며 D:/Virtual Hard Disks, D:/Downloads, 사용자 Downloads의 최상위 파일에서 ISO/VHD/VHDX는 발견되지 않았다(하위 전체 검색은 아님). 격리 VM 생성을 위한 Windows 설치 ISO 또는 준비된 Windows VHDX 경로가 다음 필요 항목이다. 실제 드라이버·DPI·반복·장시간 시험은 여전히 미실행이다.

## Desktop 실행 방식으로 교체

사용자 요청에 따라 start-admin-validation.ps1의 CLI 실행 경로와 -LaunchCodex 옵션을 제거했다. 기본 동작은 설치된 OpenAI.Codex 패키지 manifest의 ChatGPT.exe에 RunAs 관리자 실행을 요청한다. 실행 중인 ChatGPT가 있으면 중단하며 자동 종료하지 않는다. -InspectOnly로 기존 점검 기능을 제공한다. 구문 및 실행 중 앱 보호 분기를 검증했으며 실제 앱 상승 실행과 자식 도구 권한은 미검증이다. 이전 CLI 권고는 이 경로로 대체한다.

## Desktop 관리자 실행 확인

2026-10-18 07:19 KST: 수정 스크립트로 재실행한 뒤 실제 도구 셸에서 Administrator=True 및 High Mandatory Level 확인. Hyper-V Get-VM/Get-VMHost 조회도 성공했다. 도구 권한 전달 제약은 해소됐다. 등록 VM은 없으며 설치 이미지 경로/공식 평가판 선택을 확인 중이다. 장치/배율 변경이나 게스트 설치는 아직 하지 않았다. 로컬 점검 파일: runtime/verification/dpi-environment-tool-admin.json.

## 평가판 준비 진행

Microsoft 공식 한국어 평가판 ISO 수신 및 로컬 해시 기록, VirtualDrivers 25.7.23 자료 수신·서명 검사, 격리 VM 생성·부팅까지 완료했다. [환경 관측 정본](../../sots/verification/environment/sot-009-isolated-validation-environment.md)과 생성 스크립트를 추가했다. 게스트 OS 화면 설치 직전 computer-use의 Windows 설치 확인 규칙에 따라 사용자 확인을 요청했다. 실제 재생/DPI 검증을 완료로 표시하지 않는다.

UEFI 후속: 원본 부트 파일 서명과 VM 가상 키보드 정상 상태를 확인하고 키 대기 없는 ISO 사본 생성·내용 확인까지 수행했다. 현재 Hyper-V 미디어 변경 ConcreteJob이 Running/0%이며 요청이 완료되지 않는다. 서비스 재시작은 다른 작업에 영향을 줄 수 있어 수행하지 않았다. 관리 작업 해소 후 실제 부팅/입력, OS 설치, DPI/반복/장시간 시험을 순서대로 재개해야 한다.

## VMMS 복구 결과

VMMS만 재시작하고 일시적으로 정상 조회를 확인했으나 DVD 변경/관리 조회 정지가 다시 발생했다. 자체 변경 대기 명령은 취소, ISO 두 파일 보존 확인. VMMS/vmcompute 서비스 Running이며 다른 가상화 작업은 중지하지 않았다. Windows 설치 조건(정상 부팅/입력)이 충족되지 않아 검증은 보류한다. 사용자 작업 저장 후 호스트 재부팅 및 관리/ISO 조회 재검증을 다음 복구 후보로 기록한다.

## 사용자 보류

사용자 지시로 DPI 및 VM 설치 작업을 보류했다. 이후 검증은 호스트 앱의 반복 회차/연속 재생에 한정하며 VM이나 서비스를 변경하지 않는다. 사용자 측 Windows 설치 진행 상태를 검증 완료로 취급하지 않는다.

## VM 대체 지시 — 최신

후속 [앱 DPI 실기](../../sots/verification/display/sot-016-app-dpi-matrix.md): 실제 HWND96/144/192, 네이티브 전체화면 진입·해제/자동숨김·복원, 회차 패널 및 컨트롤 조작의 표본 검증 완료. 배율 복원·소유 모니터 제거를 수행한다. 최소 창·분수 배율·혼합 모니터 이동/재연결·크기 복원 누적·고배율 장시간은 별도 잔여다. 아래 ‘앱 미완료’는 이전 probe 시험의 상태다.

사용자가 VM 실패를 확정하고 VM 없는 가상 디스플레이/Rust 검증으로 변경했다. 이전 VM/ISO는 정리 대상이며 설치 복구를 진행하지 않는다. [Rust 드라이버 기록](../../sots/verification/display/sot-015-host-virtual-display-review.md)을 따른다. 후속 사용자 승인으로 인증서 등록과 설치를 완료했고 실제 probe DPI96/144/192를 확인했다. 앱 전체 고배율 매트릭스는 미완료다. 시험 모니터는 제거했고 물리 화면은 기존96 DPI다. VM/ISO 삭제 명령은 자동 승인 검토 거부로 실행되지 않았다. 신규 개발은 Rust로 통일한다.
