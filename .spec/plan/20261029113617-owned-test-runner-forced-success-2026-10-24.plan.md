---
id: plan:kinekino/20261029113617
title: 원 outer 강제 종료와 exit0 성공 판정 후속
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 outer 강제 종료와 exit0 성공 판정 후속

최신 실제: [OS·직접CLI 차등4 정본](../../../sots/verification/production/owned-test-runner-forced-success-os-cli-2026-10-24.md)에 원red 반례와 green거절을 기록했다. red OS의 정확unwrappanic, green OS 1pass, 직접red 최종passed=true/exit0, 직접green false/exit1을 각각 관측했다. 아래 source-only/actual0는 작성 당시 이력이며 이 frozen tools 범위만 갱신한다. C2는 다른 producer로 계속 미검증이다.

2026-10-24 중앙 source-only 배정. 목표는 SDK의 물리 종료 Complete 계약을 유지하며 외부 시험 감독 도구의 forced+exit0 성공 공백을 닫는 것이다. [private 후계 인계](../../../../runtime/development/owned-test-runner-forced-success-r1/HANDOFF.md)의 경로는 workspace root runtime 기준이다.

r3 소스에서 Complete+forced+exit0가 selected1/EOF/기록/남은 budget을 만족하면 passed=true가 되는 경로를 확인했다. 실제 반례 실행은0이다. SDK windows_family의 parent-exit 계약은 이 receipt를 허용하며 변경하지 않는다. 기존 normal/nested OS2 성공은 유지한다. timeout8과 별도 exact2 후계이며 다른 여섯 파일은 frozen parent와 같다.

private source2 SHA256 `1080B107F5FD9A956B0B894C47C5C3F019A38FF2E1748A283B540E24794C96F8`, effective8 `310535659BF454959B9306BB5437E65A66E6EA6DB63D4DCBEE16E5805C451DF7`. 순수 모델1, ignored regression1/stimulus1 작성과 direct rustfmt2/check2만 완료했다. Cargo fmt/test/strict, type/list, red/green OS 및 direct CLI exit1 모두 미검증이다. 공유 S/T/canonical/index 변경0. 다음 gate는 중앙 자원 배정 뒤 새 unit/normal provenance 고정, default8, 별도 OS1, 별도 direct CLI 실패 확인 순서이며 HANDOFF가 정확한 입력과 보존/반환 계약을 갖는다.

modoo.today `modootoday/software-qa/code-change-review-verification` 본문과 필수 verdict-table을 읽고 supplied claim의 호출 경로·SDK 호환·실제 관측 여부를 대조했다. 정확한 출처 SHA와 판정표는 HANDOFF에 기록했다. 신규 추론·다운로드·worker 빌드는 계속 보류한다.

후속 source-only로 private GATE-PLAN/expected20 및 red/effective8 `0F7A1DAF35D28F92A0D5577A818E8906907AE7B912237A404975C7F6F61141C1`를 작성했다. green1080/310535 불변, timeout 순수3 혼합0이다. red는 원r3 nonzero primary를 동작 동등 함수로 추출한 별도 소스이며 tests는 green과 동일하다. green 순수1+기존7, OS red/green 각1, 직접 CLI red/green 각1을 독립 분모로 계획했다. red selftest의 예상 unwrap panic만으로 실제 passed=true를 주장하지 않고 별도 red CLI 관측을 요구한다. 빌드/타입/목록/OS/CLI actual은 계속0이며 다음 heavy는 Meari다.

후속 중앙 순서 갱신: SDK N 최초 실패 기록·정리 후 heavy 반환을 접수했다. **다음 후보는 원 forced red/green 타입·순수 gate이며 C2는 그 뒤**다. 현재는 준비 확인만, Cargo/OS/directCLI 미배정이다. 위 Meari heavy 문장은 작성 당시 이력이다.

좁은 재확인으로 green effective8 `310535659BF454959B9306BB5437E65A66E6EA6DB63D4DCBEE16E5805C451DF7`와 red effective8 `0F7A1DAF35D28F92A0D5577A818E8906907AE7B912237A404975C7F6F61141C1`의 모든 raw SHA/bytes 일치를 확인했다. GATE-PLAN SHA `983403FB977FB42DB8A0E8C04DBE0B52AA4F883A9672F557F27CA181B711C99A`, expected20 SHA `A1980BB346E66B565FDA20B4C0316442204A6428DBFF414F9A011F10EEB6CB0D` 불변이다. list20/default8/ignored12, green 신규pure1→원r3 영향7, red는 타입/목록만이라는 기존 계획을 유지한다. red 순수1의 의도된 실패를 OS 반례로 세지 않는다. timeout신규3/C2test23 혼합0이다.

최신 SDK `runtime/verification/default-native-nc-r1/slot-return.json`을 읽었고 rootabb63c8/Mearif85a68d/lock8F, 공유 prior return5C59E38E의 S6081/T589/carry138 ACBA73E0, Meari 독립반환7AEC8C0B, writer0/pendingowner0을 보고 범위로 인수했다. 이 slot 자체는 full-final S/T/cache SHA를 주장하지 않는다. 이번에도 광범위 스캔0이다. 최신 진단 WIP/새 root는 실제 입장 때 중앙이 명시한 최신값과 대조하며 자동 sync하지 않는다.

공유 S의 정확 app/tests만 fresh SHA/bytes/mtime를 읽어 기존 r2 preimage(2ADE39A3/12920B/639283707752993271, D5FCE63D/10621B/639283704412768106)와 일치를 확인했다. standalone lock0B2562/8041B와 runnerEEA9/422912B도 일치했다. T의 도구 normal978/915456B와 timeout unit221F/1817088B만 정확 경로로 확인했으며 이것은 새 red/green producer가 아니다. SDK final-protected6의 bytes/mtime/기록 FileID는 fresh stat6 일치, fresh hash6 전수 재실행은 하지 않았다.

명시 입장 후에는 app/tests 원본2를 별도보존하고 red→green exact2만 overlay/복원한다. 다른6·owned-process baseline23·lock 불일치를 자동 교정하지 않는다. 기존 shared S/T·runner conservative2 CPU2/BelowNormal/offline locked j1/thread1, fresh RAM16GiB/GPU3GiB/C12GiB/writer0가 gate 조건이며 현재 자원 수치를 다시 측정해 실행 허가로 해석하지 않았다. fmt/strict/unit JSON·등록20/ignored12/greenexact8이 기본 범위이고, normal JSON build 포함 여부는 중앙 실행 배정에 명시한다. 기존 계획의 새 normal/unit pin 요구는 향후 OS를 위한 것이며 미배정 OS/directCLI는 실행0을 유지한다. 첫 실패 원 raw/bin/owner/receipt 보존·중지, 완료/실패 후 original2 bytes/mtime 원복·기존 carry/보호·Tproducer 감사/slot반환은 그대로다. C2r3 source18 5696BCB8과 이전 AEA/BDF8 후계는 보존·미실행이다.
