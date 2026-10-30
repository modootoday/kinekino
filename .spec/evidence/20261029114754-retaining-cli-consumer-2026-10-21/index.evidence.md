---
id: evidence:kinekino/20261029114754
title: Retaining CLI 실제 소비3 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Retaining CLI 실제 소비3 검증

2026-10-21 root `2fd3e94eea4d32a4189b2dfdb331f8af05d0bf7f`, Meari `0b79d57d7a59b3146b7ec72d02e2651b21b166fd`에서 실제 normal CLI의 jobs/provider/playback retaining 경로를 검증했다. [P1b 초기 타입 검증](job-owned-attempt-p1b-type-2026-10-21.md)의 후속 실제 소비 근거다. 과거 --serve-native ready 성공은 다른 경로이며 이번 호환성 근거로 사용하지 않았다.

## 고정 입력과 빌드

시험 source2 r3 SHA256 `3BF8F770A15899B1F268F009C7D5C82EB0C5321AA95672778A0E41FF25BCD4D0`. 기존 OwnedHost/BoundedNativeOwner/원 Deadline/try_call_until/try_terminal_until/ScenarioFailure를 소비하는 시험 glue이며 새 process controller나 제품 API를 만들지 않았다. 원 r1/r2는 보존했다. 사전 읽기로 발견한 canonical Windows 경로 비교와 private module 접근을 r2/r3에서 수정했다. 이는 실제 compiler/runtime 실패가 아니며 이번 gate에서는 수리·재시도0이다.

관련 default lib/player-cli/jobs strict2279ms/0, 루트 패키지 cargo fmt9391ms/0, normal CLI fresh build11974ms/0, jobs 시험 fresh build23202ms/0. 실제 목록52개 중 신규 ignored3을 확인해 정확한 이름으로 하나씩 실행했다. 전체52 실행은 아니다. native-proposal-window/test-support/validation feature는 사용하지 않았다.

현재 CLI SHA256 `3E13B00AF14D07A89785CD893C04B868CF8BE079707D8C76AD1A1B1D26E190E6`,30379520B. jobs 시험 BIN `1F2235B4D5D0CCBE23C422DB854DB69858E02B5EF15450214EDBE8B8004CA14D`,25952768B. 원/fixed/각 Fixture 사본이 일치했다. 기존 normal CLI341E…는 의도한 build 출력으로 교체하기 전에 bytes/hash를 별도 보존했다. 과거 F7 고정 CLI와 DEF SDK 시험 BIN도 보호했으며, 현재 소스의 바이너리라고 재분류하지 않았다.

## 실제 결과

| 정확한 시험 이름 suffix | 원 runner 결과 | 검증한 소비 |
|---|---|---|
| actual_retaining_jobs_short_conversion_and_owned_eof |3609ms/0,1pass|실제 --serve-jobs, 기존 subtitles.convert1, 원 source 보존, 실제 output bytes/SHA/readback, completed1·cancelled0·interrupted0|
| actual_retaining_provider_empty_ready_and_owned_eof |3362ms/0,1pass|실제 --serve-provider 기존 browser 시작, 요청 없는 ready→EOF, browser_worker_exited=true·job totals0|
| actual_retaining_playback_empty_ready_and_owned_eof |3329ms/0,1pass|실제 --serve-playback 기존 browser 시작·고정 mpv 검증, playback.open0, native Ok(null)·transition Ok(null)·provider browser 종료·job totals0|

모든 이름의 prefix는 `retaining_consumer::`다. 세 원 host와 submit 요청의 실제 process identity는 다음과 같다.

| 원 소유 범위 | PID | creation FILETIME | 관측 |
|---|---:|---:|---|
| jobs host |119268|134370276473813532|exit0/Empty/unforced/stdinEOF|
| jobs submit CLI |119280|134370276479067750|exit0/Empty/unforced/stdinEOF|
| provider host |78228|134370276726912525|exit0/Empty/unforced/stdinEOF|
| playback host |42400|134370276872047436|exit0/Empty/unforced/stdinEOF|

세 host의 pipe/SDK joined·dispatch unpoisoned를 확인했다. jobs는 dispatch completed, provider/playback은 idle이다. 원 stdout/stderr raw export는 bounded complete이며 성공 closeout의 원 receipt와 연결된다. 실제 jobs-owner/profile-owner 파일 lock은 host가 살아 있을 때 경합하고, 원 physical closure 이후 다시 획득됐다. 이 lease 관측만으로 원 프로세스 종료를 추론하지 않았다.

playback의 정확한 stop 원문은 `runtime/verification/retaining-cli-consumer-r1/evidence/retaining-playback/logs/native-stop.stdout`이다. jobs/provider는 각 `logs/stop.stdout`에 있다. 원 PID 부재는 보조 확인이며 원 receipt를 대체하지 않는다.

## 보호와 범위

증거는 `runtime/verification/retaining-cli-consumer-r1/`의 closeout.json, family-receipts.json, 원 runner receipt9, logs와 evidence에 보존했다. receipt9 terminal0, 원 family4, 기록22PID 부재, 원 exec 회수/live0이다. root2155/S2155·immutable bundles56·index·역사적 BIN·build전 CLI backup·현재 original/fixed/Fixture BIN·input16의 원본/기존 사본/이번 사본·mpvF8BF 해시 보호 차이0을 확인하고 heavy/shared를 문서 작성 전에 반환했다.

RAM29.79GiB/GPU5726MiB/C171.64GiB, EEA9 conservative2/affinity3/BelowNormal/descendant Job, 기존 S/T/offline/locked/-j1/thread1을 유지했다. 각 원90초/cleanup10초 및 ready10/request10/observation15/stop15 cap을 연장하거나 재생성하지 않았다. 시험/요청 실패도 기존 caught action→원 owner finish→conclude 경로를 사용한다.

기존 browser 시작은 이번에 명시 배정됐다. 카탈로그/외부 network 요청 명령, media open, GUI 조작, VAD/model worker 실행은 하지 않았다. 이 결과는 정상 retaining CLI 소비와 EOF만 입증한다. 실제 owned AI executor queue/retry/lease, 임의 carrier Drop/host finalization panic, VAD admission·정식 release manifest, 세 분류 전체·응답 손실·재생/회차 전환·자막/제작/AI 품질의 전체 완료는 여전히 별도다. [전체 SDK·CLI 계획](../../../plans/player/plan-006-sdk-gui-cli.md)을 유지한다.
