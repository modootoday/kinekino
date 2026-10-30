---
id: evidence:kinekino/20261029114214
title: Native GUI 실제 관측 봉인 중앙 검사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native GUI 실제 관측 봉인 중앙 검사

2026-10-19. [r137 실제GUI/SDK/CLI 제어](native-gui-endpoint-controls-2026-10-19.md)의 봉인 사본91파일을 중앙 Rust checker에서 재검증했다. 고정 manifest SHA256은 `4ff92be2286c187e943b90b3a9662822c7b207e3e382637a3c7e44c9d815e71f`다. 원시 runtime74파일·CLI21요청 및 별도native관측·복원report와 seal-time binary/source 사본의 byte수/SHA·경로유일성을 확인했다.

검사 session57956 exit0. [보고서](../../../evidence/verification/native-gui-central-2026-10-19-r1/build-evidence.json) SHA256은 `480122309ceb47d8fa78ef00b17f5c8e737ae24351f0d0e871260536d5ac4add`, 보존 검사EXE SHA256은 `5c22762f5041e34cd7e85446cff97ecec80d880b317ca5ec4bf8bc5c0503906e`다. 자체fmt/9tests/all-targets strict clippy/release build를 통과했다. 해당 검사시점397문서의canonical 링크오류0개, 보존 원본미해결3개를 [링크 보고서](../../../evidence/verification/native-gui-central-2026-10-19-r1/links.json)에 기록했다. source-at-audit 사본은 당시소스이며 compiler입력전체 attestation으로 표현하지 않는다.

## 증거의 범위

`saved_scenario_passed=true`, `saved_restoration_passed=true`는 저장한 실제 관측report의 상태다. 이번 checker가 GUI를 다시 열거나 CLI/IPC/모델을 실행한 것이 아니다. `historical_executable_identity_verified=false`를 명시적으로 유지한다. GUI/mpv는 당시fixture경로와seal-time해시로연결되며 CLI/test harness는 당시실행SHA가없어 현재경로의candidate일뿐이다. 과거실행바이트·당시컴파일입력을 새사본으로 소급확정하지 않는다. read-only속성·해시는서명이나WORM이아니다.

## 후속

다음 실제통합시험에서는 실행전 GUI/CLI/mpv/테스트바이너리를복사·고정하고실제로해당복사본을실행해야한다. 종료된r137 PID/endpoint/lease를재사용하지않는다. SDK담당의prospective하드닝과재검사후 새실제fixture가필요한경우에만별도검증한다. receipt접수/독립실제property·CAS/완료ACK/장시간품질의경계를유지한다.
