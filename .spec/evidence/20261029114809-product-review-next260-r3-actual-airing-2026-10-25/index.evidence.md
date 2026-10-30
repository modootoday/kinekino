---
id: evidence:kinekino/20261029114809
title: Source260r3 CPU 및 실제 Airing 1회
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Source260r3 CPU 및 실제 Airing 1회

- 선택 소스 SHA256: 1448D972F9A7C024F5ED35821783A8956239CF8A3DEF0BB4852E7BD6513E2A95.
- CPU gate: %USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/product-review-next260-checks-r3-gate. fmt/type/strict clippy/no-run 통과, exact71 전부 통과. 임시 소스260 복구 완료.
- 실제 gate: %USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/product-review-next260-airing-r1-gate. 원 jobs exact1, 원 GUI owner, 새로운 nonce/root, 자동 재시도0. 종료0, 36.15초.
- 원 ReadyV1 activated, frame scene2. FinalV2 first_error null, provider jobs completed3/cancelled0/interrupted0, browser 종료, dispatcher joined/fault0, execution stopped/unacknowledged0, host pipe/sdk joined/dispatch completed, cleanup_complete true, normal_os_exit true.
- 실제 mpv native observation: 1920x1080 h264/aac, 오디오2채널, position_us 213278→1214278 진행. airing-frame.png 생성. 시각 일치/수동 사용자 품질 승인을 뜻하지 않는다.
- 보호394 metadata/FileID 불변 및 CLI 독립 사본 SHA 일치 확인 → 보호395 반환 SHA256 9E9DCBAF3BE7DA02958C49F70137DA49E7B83BA6525446DE4B6AB7BB7AF7ED95. WIP5 SHA/mtime 불변, parent streams closed, 잔여 writer0. 상세 10-return.json.
- 미검증: 전체 무필터 cargo test, 다른 category, 물리 fullscreen/cursor/OS timer, 사용자 AI9 원본/후보 품질 승인. AI9 root 쓰기·모델 추론·다운로드0.
- 적용 스킬: modoo.today code-change-review-verification(body 9f73865f96ec0cefcf3a4023b294400258473640df3b1d2add03dcca8192d64b, verdict-table 9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9), test-fixture-and-mock-build(body a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100). 적용: exact 등록/실행과 실제 Ready/재생/정리 근거 및 미검증 범위를 분리. 정확한 원문 출처는 기존 SDK213 HANDOFF 참조.

## 후속 배정: 완료작·영화·인접 회차

동일 source260r3/fixed binaries/budgets로 직렬 실행했으며 재빌드·자동 재시도 없이 모두 종료0.

| 원 exact 범위 | 관측 | 반환 |
|---|---|---|
| completed 첫1 | 33.68초, jobs3, 1280×720/audio2, 최종 position1168378us, frame1971244B | 396 B3FBFF7372C3BFDFAA68A49E979A09D0B8D752F2FBC8092CF9AC015020E1388E |
| movies 첫1 | jobs3, 1280×720/audio2, 최종 position1106556us, frame4073429B | 397 639D8D63BEE15D7ACCFF5F7EE539DAA5414AFCB11BD3B45E0273F0C80A71770A |
| airing adjacent next/previous | 70.90초, jobs5, engine1→2→3, 양 전환 이전 engine/IPC/stdout/stderr 종료, next1339400us/previous1381111us, 각각1920×1080/audio2 및 frame | 398 10A8E37A5BF7A6A906E5206584725FDBF1508D55CC43BB9D458D45879CDA2CE3 |

각 gate는 공통 C 캐시의 product-review-next260-{completed,movies,adjacent}-r1-gate. 원 fixture evidence는 evidence/product-review-next260-{scope}-r1 하위 routed-gui-{category}-{first,adjacent}. 각 Ready activated, FinalV2 cleanup_complete/normal_os_exit true/first_error null, 소유 receipt family Empty/forced false/child_exit0. root 상태 및 WIP5 불변, streams4 독점 읽기 가능, 잔여 writer0. 최종 398 반환은 adjacent gate 09/10 참조.

위의 '다른 category 미검증'은 이 세 범위에 한해 후속 근거로 대체된다. 전체 기능 인수·실제 음성 청취·수동 GUI controls·사용자 AI 품질 승인·물리 OS timer/fullscreen/cursor 검증으로 확대하지 않는다. 테스트 볼륨0이며 사용자 AI9 root 쓰기0. 기존 스킬 적용 판단과 출처를 유지한다.
