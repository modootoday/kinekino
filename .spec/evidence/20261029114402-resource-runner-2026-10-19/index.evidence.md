---
id: evidence:kinekino/20261029114402
title: 소유 검사 실행기의 자원 제한
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 소유 검사 실행기의 자원 제한

상태: `tools/resource-runner`의 최소 Rust 실행기를 구현하고 현재 frame checkpoint에서 사용 중이다. 일반 validation 측정기의 대체 기능 전체를 구현한 것이 아니다. 사용자 앱의 affinity/priority나 시스템 서비스를 변경하지 않는다.

실행기는 이름 없는 Windows Job Object에 affinity·priority-class 제한을 설정하고 읽어 대조한 뒤, 자기 프로세스를 배정한다. 기존 허용 CPU mask의 첫 두 bit 이내를 선택하고 BelowNormal을 사용한다. 이 안에서 직접 자식을 실행하며 자식 핸들의 실제 affinity/priority도 읽어 대조한다. breakaway/kill-on-close를 사용하지 않는다. 실패 시 임의 fallback을 하지 않고 자신이 만든 자식만 정리한다. 부모와 자식의 PID·실제 mask/priority·종료값·경과 시간은 지정 receipt 파일에 남긴다. 후손 전체를 별도 프로세스 조회한 것은 아니며, 후손 제한은 Job Object의 상속된 정책에 따른다.

bootstrap은 전용 PowerShell 프로세스가 자기 affinity3/BelowNormal만 설정한 뒤 std-only Rust 소스를 rustc로 컴파일했다. 기존 캐시 무효화용 compiler flag·전역 설정을 추가하지 않았다. smoke rustc child100312는 affinity3/두 CPU/priority16384/exit0, 첫 Cargo child37272도 동일 제한/exit101을 기록했다. 최초 product 실패는 COM 매크로의 windows-core 직접 의존 누락이었으며 원로그를 보존했다. 기존 lock의0.62.2 직접 의존을 추가한 repair Clippy는 Cargo child100284에서 같은 제한/exit0/31.893초를 기록했다.

실행기 자체 mask 회귀1개는 통과했다. 최초 실행기 strict Clippy는 manual_isolate_lowest_one 경고로 실패했고 로그를 보존했다. 표준 메서드로 수정한 repair strict Clippy와 수정 후 자체 test1개·fmt check·최종 build도 통과했다. 최종 exe SHA는 `692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463`다. [product 전체 및 실제 frame CLI 검증](frame-document-cli-2026-10-19.md)은 별도이며 이 실행기 근거만으로 기능 성공을 선언하지 않는다.

호출 인수는 `resource-runner <stdout-path> <stderr-path> <receipt-path> <executable> [arguments...]`다. shell 명령을 조합하지 않고 OS 인수를 그대로 전달한다. 현재 checkpoint 로그는 `runtime/quality-review/frame-document-cli-r1/`에 둔다. runner-smoke.json·clippy-first-receipt.json·clippy-repair1-receipt.json·test-first-receipt.json에서 실제 적용 값을 확인한다.
