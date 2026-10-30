---
id: evidence:kinekino/20261029113518
title: 격리 검증 환경 준비 기록
status: archived
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# 격리 검증 환경 준비 기록

최신 상태: 사용자 확인으로 VM 설치 실패, 복구 대신 [호스트 Rust 드라이버 경로](../display/sot-015-host-virtual-display-review.md)로 대체했다. 아래는 준비 이력이다. VM/ISO 정리는 자동 승인 검토 차단으로 남아 있다.

사용자 승인: Microsoft 공식 평가판으로 별도 VM을 준비한다. 실제 DPI·반복 회차·장시간 재생 결과와 환경 준비를 구분한다. [최상위 규칙](../../../AGENTS.md), [검증 계획](../../../plans/player/plan-002-isolated-dpi-validation.md)을 따른다.

## 관측한 권한과 호스트

2026-10-18 KST, 실제 도구 셸 Administrator=True, High Mandatory Level 및 Hyper-V 조회 성공. 등록된 VM 없음. Default Switch와 WSL 스위치가 있으며 게스트에는 기존 Default Switch만 연결한다. 호스트 모니터/드라이버/배율/다른 프로세스는 변경하지 않는다.

## 설치 자료 출처

- [Microsoft Evaluation Center 한국어 Enterprise ISO](https://www.microsoft.com/en-us/evalcenter/download-windows-11-enterprise)의 공개 다운로드 링크 LinkId=2382804. 관측된 리디렉션은 software-static.download.prss.microsoft.com의 26300.9457.260913-1737.26h2_ge_release_svc_refresh_CLIENTENTERPRISEEVAL_OEMRET_x64FRE_ko-kr.iso. 평가판은 90일이며 실제 OS 로그인·활성화 요구가 있을 수 있다. 사용자 계정 인증은 사용자에게 넘긴다.
- [VirtualDrivers 25.7.23 릴리스](https://github.com/VirtualDrivers/Virtual-Display-Driver/releases/tag/25.7.23)의 VDD.Control.25.7.23.zip. SHA-256 A701F2272E9FCF382849B24F913C6DD07597B3B1116525F2E90182F019609154. 호스트에서는 압축 해제와 서명 검사만 수행했다.
- VDD Control.exe와 x86/VDD/mttvdd.cat Authenticode=Valid, 서명자 SignPath Foundation. Dependencies/devcon.exe=Valid, Microsoft Corporation. INF의 NTamd64 대상 및 Root\MttVDD 확인. 서명 검사 성공과 게스트 드라이버 로드 성공은 별도 판정이며 설치는 아직 하지 않았다. 보안 설정이나 테스트 서명 설정을 변경하지 않는다.

## VM 구성 계약

[생성 스크립트](../../../../scripts/validation/new-dpi-lab.ps1): Source-DPI-Lab, Generation 2, CPU 4개, 메모리 시작/최대 8GiB·최소 4GiB, 동적 디스크 96GiB, Secure Boot MicrosoftWindows 및 vTPM. 기존 이름/디스크가 있으면 덮어쓰지 않는다. 저장 경로는 프로젝트 runtime/validation-vm 아래이며 버전 관리 제외다. 호스트 부팅·물리 디스크 설치를 수행하지 않는다.

## 아직 검증하지 않은 항목

게스트 OS 설치 완료, 가상 디스플레이 드라이버 설치/로드, Windows 100/125/150/200% 배율, 서로 다른 DPI 사이 이동, 반복 회차 전환 및 웹 30분/mpv 60분 연속 재생은 아직 미검증이다. VM의 가상 GPU 결과를 호스트 NVIDIA 디코딩 성능으로 해석하지 않는다.

## 생성 완료 관측

ISO 8,225,404,928바이트 수신 완료, 로컬 SHA-256 B7EC51F0D332AC2AA4F7FAEFFBEFAC961CE6EB7204B5876B48FA8D9F1CF0A9A6. 게시자 해시 대조는 아직 아니며 공식 HTTPS 링크에서 수신한 파일의 로컬 식별값이다. ISO 볼륨 CENA_X64FREE_KO-KR_DV9 마운트 확인 후 호스트 마운트 해제. new-dpi-lab.ps1로 VM 생성 성공, Start-VM 후 Running 및 CPU 4개 관측. vmconnect 창에서 설치 ISO 부팅 프롬프트를 관측했다. OS 설치는 아직 완료하지 않았다. Rust 17개 테스트·fmt·Clippy 통과.

## 설치 진입 대기

사용자가 VM 내 Windows 설치 화면 조작을 승인했다. DVD 부팅 입력 제한시간을 놓쳐 UEFI Boot Summary에 도달했고 실제 OS 설치는 시작되지 않았다. VM 연결 창 네이티브 메뉴 입력은 동작했으나 게스트 부팅 키 입력을 성공시키지 못했다. 마지막 툴바 조작은 리셋이 아닌 저장으로 동작하여 게스트 상태 저장을 요청했다. 후속 시작/재설정과 DVD 부팅 키를 직접 입력한 뒤 설치 화면을 다시 확인해야 한다. 이를 ISO 손상이나 애플리케이션 결함으로 단정하지 않는다.

## UEFI 및 입력 진단

사용자가 물리 입력도 전달되지 않는다고 보고했다. Boot Summary의 SCSI DVD 'The boot loader failed', Network 'A boot image was not found', SCSI Disk 'did not load an operating system'을 직접 확인했다. Generation=2, SecureBoot=On/MicrosoftWindows, DVD 경로·SCSI 0:1 및 가상 키보드 EnabledState=2/Status=OK 확인. 설치 전 합성 마우스는 EnabledState=3/Status=OK였으며 그 자체를 생성 오류의 증거로 삼지 않는다. bootx64.efi 3,087,400바이트, boot.wim 641,406,657바이트, install.wim 7,302,389,399바이트 및 bootx64.efi Authenticode 유효 확인. Hyper-V Worker 이벤트 18603(게스트 OS 부팅 실패) 관측. 이 증거만으로 물리 키보드 입력 실패 원인을 확정하지 않는다.

## 자동 부팅 사본 및 Hyper-V 관리 작업 대기

pycdlib 1.22.0으로 원본 El Torito EFI 항목의 LBA 555(오프셋 1,136,640)를 읽고, 해당 1,474,560바이트가 원본 efisys.bin과 같음을 확인했다. 원본을 복사한 뒤 동일 ISO에 포함된 efisys_noprompt.bin으로 해당 영역만 교체하고 다시 읽어 일치를 검증했다. 교체 이미지 SHA-256 bc1df11a9148b4e3b60b32095ca3dc5d400ca0d16e2b43d3a8a0282a9e66c4d5. 원본은 변경하지 않았다. 이 사본은 Microsoft 원본 ISO와 구분한다. 생성 스크립트 및 provenance를 로컬 보관한다.

Set-VMDvdDrive 후 시작 요청은 반환되지 않은 상태다. Msvm_ConcreteJob에 '리소스 수정 중', JobState=4, PercentComplete=0이 관측됐으며, 새 vmconnect는 MainWindowHandle=0으로 창이 없었다. 따라서 수정 사본 부팅 성공 및 입력 복구는 미검증이다. 가동 중 vmwp가 Lab VM ID와 일치하지 않아 중지하지 않았다. 호스트 다른 작업 보호를 위해 VMMS/vmcompute 서비스나 시스템을 재시작하지 않았다.

## VMMS 재시작 복구 시도

사용자 승인 후 VMMS만 Restart-Service로 재시작을 요청했다. StopPending이 지속되어 서비스 PID/프로세스 이름을 확인한 뒤 VMMS 프로세스만 종료하고 서비스를 시작했다. vmcompute 및 다른 vmwp는 종료하지 않았다. Get-VM/Get-VMDvdDrive가 한 차례 정상 반환하여 Lab VM Off와 원본 ISO 연결을 확인했다. 이후 Set-VMDvdDrive를 재시도하자 Resource modification Running/0%가 재현됐다. 해당 ConcreteJob은 Cancellable=false였다. 정상 종료 요청 시도를 포함한 일부 관리 조회도 응답 대기 상태가 됐다.

VMMS를 다시 시작했으나 이후 관리/스토리지 조회가 재차 응답 대기에 들어가 완전 복구로 판정하지 않는다. 파일 교체를 시도하려던 자체 PowerShell과 DVD 변경 후 VM 시작을 포함한 자체 대기 명령을 정확한 명령 문자열/프로세스 ID로 식별해 종료했다. 최종 파일 목록에서 원본 ISO와 noprompt ISO 각 8,225,404,928바이트 존재, source.iso로 이동되지 않은 상태를 확인했다. 최종 서비스 조회에서는 VMMS와 vmcompute Running. Lab 게스트의 최신 상태는 관리 조회 지연으로 재확인하지 못했으며 마지막 정상 조회는 Off였다. OS 설치·입력 복구·DPI/재생 시험은 진행하지 않았다. 호스트 재부팅은 사용자 작업 보호를 위해 수행하지 않았으며, 작업 저장 후 재부팅과 관리 경로 재검증이 다음 복구 후보이다. 재부팅이 확정 해결책이라는 증거는 없다.
