---
id: evidence:kinekino/20261029113912
title: 프로젝트 캡처 편집 진입 차단 CPU 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 프로젝트 캡처 편집 진입 차단 CPU 검증

root `e9ff30f131e0f97a7df11b48edd95ebb7c7eebc9` / Meari `4331ff3ea5cad71a7159bdb9e3f86fbfccb16220` 기준 독립 snapshot에서 source7 `F73F5EA866E4B7ADA4940F247FB7124C431E3169394C77485643BAA00D1842C0`을 원 byte 수리 없이 검증했다. 기존6/신규1, 부모 source5 D80B0BFE + 시험 delta2 CBDCBDD7이며 root 채택은 PM의 별도 단계다.

실제 Native BIN `CCDB8F209EE88415345C6C426823AC6B318724190BDD247BC2E689CAB7047241`에서 등록918·중복 없는 선택56을 확인했다. 같은 BIN에서51 통과/0실패/5제외다. 기존75 전체나 SDK7을 반복·합산하지 않았다. `runtime/quality-review/ui-capture-readonly-boundary-r1/`의 원 stdout·actual-tests.json·results.json이 실제 명단/결과 근거다.

실제 parse된 소스 버튼→원 HostCallbacks export→ViewerCallbacks의 local key 확인 직후 PermissionDenied를 반환하는 회귀가 통과했다. queue를 이미 mut borrow한 중에도 Capacity 대신 거부하며 queue0·SDK forwarding 명령0·retained operation0·원 read attachment와 scope/caller capture 보존을 검사했다. 이 콜백의 거부를 SDK plugin으로 fallback하지 않는다. 일반 live Card의 실제 제품 소스 기반 edit/save/명시 accept·output-review operation/ACK 및 primitive Viewing/read 재등록·stale 선택 회귀도 통과했다.

공개 request_selected_card_route와 retained opening→finish_inspected_card의 차단 소스는 타입 검사됐지만, 실제 concrete ViewerHost의 부정 회귀2는 LocalHost가 필요해 이번에는 제외했다. body panic 때 원 host 정리를 관측하는 보강도 남았다. 해당2를 실행 성공이나 전체 actor의 queue/route/registration 보존 근거로 표시하지 않는다. 그 시험의 legacy queue 직접 주입은 우회 상태 검증용이며 실제 콜백 거부 증거와 구별한다.

fmt3800/default strict1305/Native strict25778/fresh build97359ms가 모두0이다. root1804/S1805/library456의 SHA·UTCmtime 및 양 index/docs2/SDK5/raw7/before7 보호 차이0, 원 normalCLI5DB·nativeCLI7A54·사본 불변, 의도 Native 출력4081→CCDB와 원4081 고정 사본 보존을 확인했다. 원13 receipt terminal0·원5 exec handle terminal/EOF·관측29 PID 부재를 함께 확인하고 실행 슬롯을 반환했다. LocalHost·창/실제 프레임·GUI/browser/media/model/VM/DPI를 실행하지 않았다.

[프로젝트 API 정본](../../player/application/api/projects.md)에 따라 capture-origin live edit/accept는 원자 capture/source mutation guard가 없어서 미지원이다. 이 결과는 임시 차단 검증이며 SDK guard 구현 성공이나 기능 요구 제거를 뜻하지 않는다. UI의 같은 tuple 비교·현재 조회1회·origin 이름 변경으로 권한을 대신하지 않는다. 후속 SDK origin/동일 writer guard·지원 복구와 실제 selector/복귀·현재 저장본 재열기·모든13화면군 시각 인수는 계속 남아 있다.
