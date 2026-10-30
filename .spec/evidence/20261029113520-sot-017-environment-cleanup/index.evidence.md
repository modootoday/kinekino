---
id: evidence:kinekino/20261029113520
title: 실패 VM 환경 정리와 실행 도구
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# 실패 VM 환경 정리와 실행 도구

## 최종 확인

사용자가 관리자 PowerShell에서 apply를 직접 실행했다. 결과는 deletion_performed=true, status=complete, target_exists=false다. 후속 파일 시스템 조회에서도 runtime/validation-vm 부재를 확인했다. 아래의 삭제 미실행 설명은 사용자 실행 이전 경과다. [실제 삭제 결과](../../../evidence/environment/cleanup-vm-result-2026-10-18.json)를 보존한다. 현재 드라이버와 독립 devcon 보존본은 유지했다. 실제 디스크 회수량은 측정하지 않았다.

사용자는 추가 재생 안정성 시험을 보류하고 환경 정리부터 우선순위대로 진행하도록 지시했다. 이후 삭제 스크립트를 명시적으로 요청했다. 기존 재생 표본은 보존하며 보류를 새 시험 완료로 표시하지 않는다.

## 실제 처리

2026-10-18 읽기 전용 검사에서 Source-DPI-Lab은 Off, ID16d7358f-f1a7-4f61-a22d-d3953ffaf8a8, 등록 경로는 이 프로젝트 runtime/validation-vm/Source-DPI-Lab이었다. 다른 VM 등록은 없었다. 해당 **VM 등록 제거는 성공**했다. VMMS/vmcompute·WSL·다른 앱을 종료하지 않았다.

파일 정리 전 현재 DPI 도구가 사용하는 devcon.exe의 독립 보존본 runtime/display-driver-review/devcon.exe를 확인했다. SHA256 `77e87e7c4e23d6b9a4f5d86d15ed9ec61a16a242fb299c390af6f179225bfa05`. Rust 드라이버 설치 어댑터의 옛 VM 폴더 복사 의존성을 제거하고 이 보존본의 해시를 직접 검사하도록 수정했다. [ISO 출처 메타데이터](../../../evidence/environment/vm-iso-provenance-before-cleanup.json)는 보존했다.

프로젝트 안의 절대 대상 경로·하위 파일·reparse point 부재를 확인한 뒤 runtime/validation-vm 삭제를 시도했으나 자동 승인 검토가 실행 전에 `blocked by policy`로 거부했다. 구체적 사유는 제공되지 않았다. **디스크·ISO·옛 제어 패키지는 삭제되지 않았다.** 동일 삭제를 다른 실행 방식으로 우회하지 않았다.

## 요청한 실행 파일

[실행 스크립트](../../../../scripts/cleanup-validation-environment.ps1)는 Rust CLI를 호출하는 얇은 진입점이다. 스크립트 자체에 삭제/경로 정책을 구현하지 않았다. 새 창을 띄우거나 현재 PowerShell/ChatGPT/Terminal을 종료하지 않고 작업 디렉터리를 복원한다.

Rust 책임:

- domain/cleanup: 고정 대상과 알려진 파일/디렉터리 허용 목록.
- application/cleanup_vm: plan/apply 순서, 재검사, 부분 실패 감사 기록.
- infrastructure/cleanup_files: 파일·경로·reparse 검사, Hyper-V 등록 읽기 조회, 파일별 삭제와 빈 폴더 제거.

기본은 plan이다. 다른 프로젝트, 알 수 없는 자산, junction/symlink, 등록된 대상 VM, 변경된 파일·보존 도구가 있으면 중단한다. apply는 전체 재귀 삭제 대신 검사한 파일을 하나씩 지운 뒤 빈 폴더만 제거한다. 경로 변경을 받는 옵션은 없다. 대상으로 이름이 정해진 실패 VM 파일·평가판 ISO2개·옛 VDD 제어 배포본만 다룬다. 현재 Rust 디스플레이 드라이버·인증서·AI 모델·mpv·검증 로그는 대상 밖이다.

실제 스크립트 plan을 실행했다. 첫 검사에서 허용 목록에 없던 빈 Virtual Machines 폴더를 거부했고, 내용을 직접 확인한 뒤 정확한 빈 폴더만 허용했다. 최종 plan 성공: 파일20개, 논리 크기35,589,360,589B(약35.6GB). 실제 디스크 회수량은 sparse/compression 등으로 다를 수 있다. [검사 계획](../../../evidence/environment/cleanup-vm-plan-2026-10-18.json). 삭제 수행은 false다. apply 분기는 실행하지 않아 실제 삭제 성공을 주장하지 않는다.

사용자가 관리자 PowerShell에서 직접 실행할 명령:

```powershell
& 'D:\workspace\@myso-kr\com.kinekino.player\scripts\cleanup-validation-environment.ps1' -Mode plan
& 'D:\workspace\@myso-kr\com.kinekino.player\scripts\cleanup-validation-environment.ps1' -Mode apply
```

기록: runtime/verification/cleanup-vm-plan.json, apply 실행 시 cleanup-vm-result.json. 결과의 status=complete 및 target_exists=false를 확인해야 파일 정리 완료로 갱신한다. failed-partial이면 removed 목록과 오류를 기준으로 현재 상태를 확인한다. 자동 승인 검토의 거부를 숨기거나 이 실행 파일로 에이전트가 우회 실행하지 않는다.

검사: 제품19개 테스트와 fmt/Clippy, 독립 Rust 도구6개 테스트와 fmt/Clippy. 허용 목록·부모 경로 거부 회귀를 추가했다. 실제 자산 삭제 시험은 미실행이다.
