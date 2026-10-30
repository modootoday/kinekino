---
id: evidence:kinekino/20261029114409
title: Host 소유 join dispatcher 집중 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Host 소유 join dispatcher 집중 검증

2026-10-20 관측. [P3 계획](../../../plans/player/native-resolution-handoff.md)의 후속8파일 revision이며 [기존 parent broker 검증](native-parent-broker-2026-10-20.md)과 구분한다. 실제 NativeChild·OS parent·main 조립은 아니다.

## 입력과 보존

실제 cwd는 `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1`이다. 직전 [UI11 검증](../presentation/historical-source-display-2026-10-20.md)의 root1fd8+공개 overlay4+P3a50905e+UI298f02310/fixture8d15f671, 원 Meari2c08832를 명시 base로 사용했다. 새 live UI P1b/P2 및 Meari0589 변경은 섞지 않았다.

직전 UI input/handoff/overlay11 핀을 별도 복사하고, 덮어쓸8경로의 존재 여부·before SHA와 기존 파일 bytes를 보존했다. 정확8경로만 byte copy했으며 원 archive나 줄바꿈/BOM을 다시 쓰지 않았다. 원 base의 archive export 차이61건은 UI가 원 archive member bytes와 대조한 별도 근거를 유지한다.

소스 r2핀 기록 SHA는 `A8E65EDE052699A142EFA2EE68C640D27A0FA475D2A97563D404DACD63A6D31A`다. r1 B890 기록은 미검증 초안이며 원 source/pin JSON 사본이 없고 기록 hash·tool 이력만 있다. r2는 bounded rejection 시험에서 유일 Arc owner의 weak reference 확인을 보강한 차이다. 원 P3 및 UI 성공을 r1이나 새8파일의 성공으로 소급하지 않았다.

최종 live/snapshot8파일·동결 r2 핀의 차이는0이고 UI11 보호 경로 차이도0이다. 최종 overlay8 기록 SHA는 `CB7153070ED0B8F7F543A501FD8D6AF45F203DAE43DF70EDCD2005A67B1C972A`다. 원로그·before bytes·입력 핀·receipt는 `runtime/quality-review/sdk-join-dispatcher-r2/`에 보존한다.

## 실행 결과

동일 fixed runner `692C22751FAC7F9999F2D833571DAB6605AFD58E66711370494E62DD8A62C463`, 기존 C drive root default 캐시, offline·locked·j1·test thread1·affinity3·BelowNormal·descendant Job을 사용했다. 원 handle 종료 전 다음 검사를 시작하지 않았다.

| 범위 | 결과 | 시간 ms |
|---|---|---:|
| `-p kinekino-player --lib infrastructure::thread_completion` | 4 pass, 종료0 | 62963 |
| `-p kinekino-player --lib infrastructure::mpv::parent_binding` | 6 pass, 종료0 | 2547 |
| `clippy -p kinekino-player --lib --bins -- -D warnings` | 종료0 | 10425 |
| `cargo fmt --all -- --check` | 종료0 | 2518 |

이번 gate에는 실패나 source 수리가 없었다. 10개는 이 입력의 집중 범위이며 이전 고유441·P3 재검증과 합산하지 않는다. lib/bins strict이며 전체 all-target/전체 native·bridge 회귀를 새로 실행한 기록이 아니다.

## 소유와 의미

std bounded dispatcher가 accepted 작업의 child thread를 실제 join한 뒤 결과를 보존하고 wake callback을 실행했다. TLS destructor에서 child 종료를 지연해 함수 결과만 먼저 준비돼도 결과·wake를 노출하지 않는 반례를 확인했다. Queue Full/Closed는 원 미시작 closure를 반환하고 accepted 작업은 host close 후에도 drain했다. Notification panic 및 join 뒤 dispatcher fault에서도 이미 join된 child 결과를 유지하고 host fault와 구분했다.

ParentCleanupTask의 dispatcher/wake 주입과 worker-only Condvar 대기를 확인했다. Inflight zero가 제출 전이거나 후여도 동일 predicate lock 아래 확인·통지해 wake를 잃지 않았고, revoke 후 late admission을 거부했다. UI 관측은 cell을 읽을 뿐 wait/join을 하지 않는다. child completion과 별도의 host dispatcher shutdown/join을 각각 관측했다.

4receipt는 모두 terminal0이며 마지막 확인에서 해당8개 launcher/child PID가 모두 부재했다. 원 exec55714/21006도 종료됐다. Host owner는 자체 Joined 관측까지 보존해야 하며 Drop을 완료 receipt로 사용하지 않는다.

## 한계

검증 소스8파일은 `3a754b4fee555562db6db144eed2e6b8b45f1b57`로 커밋했다. Cached check와 exact8 범위 확인 후 커밋했으며 사전/사후 source pin 차이0, staged 잔여0으로 index·heavy·snapshot 소유를 PM에게 반환했다. Live Meari0589 gitlink와 새 UI/P2 소스는 이 커밋에 포함하지 않았다.

주입된 callback의 반환은 GUI의 event 소비 확인이 아니다. 실제 Meari CompletionSignal closure·NativeChild stamp/attempt 조립, !Send parent lease·OS destroy 경합·geometry/focus·동일 SDK main 조립은 후속이다. Windows pool·child별 supervisor·GUI timer polling·새 SDK/wire/Cargo·실OS·mpv·실사이트·AI 실행은 추가하지 않았다. 실제 async GUI cleanup이나 전체 제품 완성으로 이 제한 gate를 표현하지 않는다.
