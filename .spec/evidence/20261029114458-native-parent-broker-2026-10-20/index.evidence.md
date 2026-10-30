---
id: evidence:kinekino/20261029114458
title: Native parent broker 집중 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native parent broker 집중 검증

2026-10-20. [P3 연결 계획](../../../plans/player/native-resolution-handoff.md)의 infrastructure broker/forwarding 소스5파일에 대한 제한 검증이다. 실제 NativeChild·OS parent·mpv·미디어 연결은 하지 않았다.

## 입력

실제 cwd는 저장소 밖 `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1`이다. [기존 공개 입력 검증](sdk-public-input-2026-10-20.md)의 root/Meari archive와 overlay4를 복원하고 P3 소스5개만 반영했다. 최초 `.spec`·runtime은 없었으며 live UI historical 표시10/fixture1 및 격리 Meari P1b는 섞지 않았다.

대상은 `src/infrastructure/mpv/{parent_binding.rs,deferred_engine.rs,mod.rs}`와 `parent_binding/{tests.rs,cleanup_task.rs}`다. 최초5파일 pin 기록 SHA는 `C0B3DA3CDF48E699B256B4955D5A0BD99F130115D7D1BD8CA251401859BFAF88`, 최소 수리 후 기록 SHA는 `710DEAFEB3DF6CA94B6368C23AC27EDE646B51E2521ED3540A3C5FCAC1DEA38D`다. 원핀·소스 사본과 새핀을 구분했다. 마지막 live/snapshot의5파일 SHA 차이는0이었다.

기존 fixed runner `692C22751FAC7F9999F2D833571DAB6605AFD58E66711370494E62DD8A62C463`, root default target 캐시, offline·locked·j1·test thread1, 논리CPU2개·BelowNormal·descendant Job을 사용했다. 신규 runner·SDK instance·application DTO·CLI wire·Cargo·main·UI·Meari 변경은 없다.

## 실행 결과

| 검사 | 결과 | 시간 ms |
|---|---|---:|
| 최초 snapshot fmt | 종료0 | 17228 |
| `cargo test --offline --locked -j1 -p kinekino-player --lib infrastructure::mpv::parent_binding` | 5 pass / 0 failed, 종료0 | 95130 |
| 최초 `cargo clippy ... -p kinekino-player --lib --bins -- -D warnings` | 큰 bind Err128bytes 진단1건, 종료101 | 11290 |
| 동일 필터 최소 수리 후 | 5 pass / 0 failed, 종료0 | 30192 |
| 동일 lib/bins strict 최소 수리 후 | 종료0 | 10395 |
| 최종 snapshot fmt | 종료0 | 2675 |

수리는 `Result<(), ParentBindFailure>`를 `Result<(), Box<ParentBindFailure>>`로 바꾸고 같은 실패 객체를 Box에 넣은 두 표현뿐이다. 오류·반환 엔진 owner·정책·assertions는 유지했다. 첫 실패와 원핀은 삭제하지 않았다. 최종 source와 원로그·6receipt는 `runtime/quality-review/sdk-parent-broker-r1/` 및 `runtime/coordination/sdk-git-baseline-r1/`에 보존한다.

## 입증 범위

fake5는 준비 전 NeedsInput·stale epoch·중복 bind의 반환 owner, 같은 엔진에서 media close/reopen 후 stale instance 제어 거부, slot이 잠긴 상태의 빠른 revoke와 late bind 거부, inflight open 중 Pending cleanup과 동일 owner의 실패→불완전→완료 재시도, 결과가 준비돼도 worker가 실제 반환하지 않으면 Pending을 유지하는 경계를 확인했다. caller-driven observe는 is_finished 전에 join하지 않고 Finished는 worker를 join한 뒤 반환한다. 그 결과 안의 Pending/Incomplete/Err를 엔진 정리 Complete로 바꾸지 않는다.

6receipt가 모두 terminal이고 마지막 확인에서 해당 launcher/child PID는 모두 부재했다. 이 결과는5개 집중 회귀와 lib/bins strict이며 전체446개나 all-targets 회귀를 새로 실행한 기록이 아니다. 기존 고유441과 중복 재검증을 합산하지 않는다.

## 남은 연결

현재 ParentCleanupTask는 caller-driven 관측 포트이며 GUI가 반드시 다시 관측한다는 liveness 증거가 없다. NativeChild의 begin 후 Wait에서 최종 worker 종료·joined receipt를 전달할 completion/wake 포트를 연결해야 한다. 결과 전달 wake가 thread 종료보다 빠른 경우 다음 wake가 없을 수 있으므로 정기 GUI polling을 암묵적으로 가정하지 않는다.

원 !Send lease의 parent 파괴 조율, revoke와 이미 admit된 launch의 OS 수명 경합, raw handle 재사용 방지, geometry/focus 및 실제 같은 SDK/client/main 연결은 미검증이다. CLI의 None standalone은 유지했고 GUI fallback은 만들지 않았다. 실제 OS·mpv·실사이트·AI·다운로드·품질 승인·UI historical 표시 인수로 이 fake gate를 확대하지 않는다.
