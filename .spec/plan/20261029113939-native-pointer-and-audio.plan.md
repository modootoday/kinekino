---
id: plan:kinekino/20261029113939
title: Native 포인터·무음 결함 수리
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native 포인터·무음 결함 수리

사용자가 영상 hover 때 커서가 사라지는 현상을 수정 대상으로 확정했다. [전환 인수](../../sots/verification/application/native-transition-cli-2026-10-19.md)의 성공과 분리한 후속 checkpoint다. 같은 중앙 단일 heavy 슬롯에서 순차 진행하며 기존 전환 bin과 실패 원문을 보존한다. DPI·VM·새 모델·별도 동시 GUI 실행은 변경하지 않는다.

1. 두 native launch 경로의 고정 `--input-cursor=no`와 미지정 cursor-autohide를 공통 infrastructure pointer_policy로 수정한다. pointer 이벤트 허용·engine 자체 자동 숨김 비활성으로 일관되게 연결한다. 제품 fullscreen 유휴 숨김 정책을 mpv 기본값에 위임하지 않는다.
2. 실제 고정 mpv의 --no-config --list-options와 공식 매뉴얼 의미를 대조한다. 현재 pinned mpv 기본값은 cursor-autohide1000ms/fs-only=no/input-cursor=yes다. 기본값 관측은 사용자가 본 즉시 사라짐의 원인을 단독 확정하지 않는다.
3. fmt/test/strict·GUI/CLI/harness pin을 새 checkpoint로 실행한다. 이전 전환 r2 pass를 수정된 입력에 상속하지 않는다.
4. 고정 CLI의 실제 재생에서 Rust Windows 관측 어댑터로 단일 visible window의 제품 title·실행 파일 경로를 확인한다. 원 cursor 좌표를 보존하고 enter/move/1.5초 idle/영상 아래 가장자리에서 GetCursorInfo SHOWING과 HWND hit target을 관측한 뒤 원 위치를 복원한다. 실제 제품 transport overlay 접근·fullscreen·사용자 외부 앱 동작은 별도 범위다. 실패를 보존하고 기한/완료조건/PNG 조건을 완화하지 않는다.
5. 커서 checkpoint 후 legacy GUI의 초기 `--mute=yes`와 Web→native handoff 뒤 음소거 해제 요청·관측 경계를 수리한다. 초기 mute의 중복 소리 방지 의도를 보존하며, 단순 삭제만으로 두 엔진 소리가 동시에 나게 하지 않는다. GUI state 기본volume80인데 실제 mute가 남는 불일치를 결함으로 다룬다. 공통 volume/mute 제어·관측 계약과 기존 source 일치 조건을 대조하고 GUI 전용 raw 정책을 복제하지 않는다. CLI 실제 검증 volume0은 의도한 설정으로 유지하며, 명시 사용자 mute 선택이 도입되면 존중한다. 현재 mute 선택·지속 저장의 미구현 상태, 설정값·audio stream 관측·실제 청취 성공은 각각 구별한다.
6. 두 경로의 수리와 관측·정확한 남은 범위를 정본화하고 heavy/root 소비 동결을 중앙에 반환한다.

공식 근거: [mpv cursor-autohide](https://mpv.io/manual/stable/#options-cursor-autohide), [mpv input-cursor](https://mpv.io/manual/stable/#options-input-cursor). 포인터 정책 변경은 외부 raw 명령 입력을 추가하지 않는다.
