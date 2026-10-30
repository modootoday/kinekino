---
id: evidence:kinekino/20261029113508
title: snapshot CLI owned-process 최초 소비 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# snapshot CLI owned-process 최초 소비 검증

2026-10-21 KST. 중앙 PM의 helper 정확4 gate를 실행하고 heavy/shared 슬롯을 명시 반환했다. root 채택·index·commit은 PM 소유이며 이 담당의 live root/index 변경·push는0이다.

중앙 채택 후속: PM은 exact4를 **778c483e362e496c405528fe0ac4e374c7b8dfc3**로 커밋했다. 이어 별도 검증된 Meari408을 root **17d15767b15256cd4befe18edd56e8a76a37816a**에서 채택했다. 아래 helper gate는 원4b58/Meari405 입력으로 수행한 근거이며 후속408에서 같은18개를 재실행했다는 뜻이 아니다.

## 입력과 책임

기준 root4b58f698294faac95dfe9d6ec5c34d28d36562c0, root Meari gitlink98f6a5c4c72ad07a40ac0eae7f7f113926609fae 및 원405을 보호했다. SDK 최종 union23 r2·producer19·기존 등록 Cargo/lock을 보존하고 반환 기대1367개 hash 일치를 확인했다. 별도 Meari408 gate는 이 입력에 합류하지 않았다.

정확4는 snapshot_cli_evidence.rs·신규 owned_process_evidence.rs와 직접 소비 tests/snapshot_export_api_cli.rs·tests/frame_snapshot_pipeline.rs다. source4 SHA256 **8C9C51090F3241A6EC6CC41CCD83FE520A4B4CD98A5F02925EEA7397C6B77440**를 무수리로 실행했다. 원 before3/부재1과 호출14 대응은 runtime/coordination/snapshot-owned-process-r1에 있다. gate 결과는 runtime/verification/snapshot-owned-process-r1을 따른다.

## 실제 선택·명령과 결과

원 runner EEA9/conservative2, snapshot public-p3-parent-r1 cwd/명시 Cargo.toml, 기존 project-9dc296c4/native target, offline/locked/-j1·thread1을 사용했다. SOURCE_SNAPSHOT_CLI_EVIDENCE_ROOT를 전용 영구 경로에 설정했다. 원 exec terminal 후 다음 명령을 진행했다.

| 명령/인수 범위 | 실제 결과 | runner ms |
| --- | --- | ---: |
| cargo test -p kinekino-player --test snapshot_export_api_cli -- --test-threads=1 --nocapture |18pass/0fail/0ignored/0filtered|107344|
| cargo test -p kinekino-player --test frame_snapshot_pipeline --no-run |컴파일0, 시험 실행0|10085|
| cargo clippy -p kinekino-player --all-targets -- -D warnings |실제 Checking/exit0|2732|
| 같은 strict + --features native-proposal-window |실제 Checking/exit0|3221|
| cargo fmt --all -- --check |exit0|3048|

실제 고유18=새 adapter8+response3+기존 SDK3/실제 fresh CLI4다. 기본 빌드99초와 시험6.47초는 cargo 원 로그의 구분이며 runner 전체107344ms와 다르다. 동일 support 모듈이 frame target에 포함됐다는 이유로 고유11을 두 번 세지 않았다. frame의 기존 매체 시험은 계속 ignored이며 실제 미디어·FFmpeg 인수는0이다.

새11은 finite file/prefix/hash·nonzero exit, empty output/결과파일 export 실패, familyUnknown/원·secondary 오류·Live, invalid budget/비ASCII environment/필드 보존, NotSpawned, receipt 쓰기 실패 원 owner, 동일 deadline 비리셋, stderr probe/coarse cap, 빈/비정상 JSON, nonzero 오류 응답 유지, process/failure evidence 쓰기 실패를 인수한다. Family seam은 실제 producer Supervisor를 소비하지만 OS 장애 재현이나 실제 Windows handle 시험으로 확대하지 않는다.

## 실제 CLI와 raw 증거

이번 Cargo 산출 player-cli.exe는29668864B, SHA256 **c49ddf13c288e970f8135b0fa39732107d489a61b2f41ffab9d749dde21fa65e**다. cli-artifact.json/final-cli-artifact.json의 전후 SHA가 같다. 실제 family receipt는 원 canonical executable 경로·creation_filetime·PID를 남겼다. 기존 CLI4는 루프를 포함해 요청11회를 실행했고 정상0 및 예상 오류2/3/5를 원 JSON/exit와 함께 확인했다. 예상 nonzero는 시험 실패가 아니다.

영구 evidence의 실제 CLI receipt11은 child exit·own family Empty·forced=false를 관측했다. process schema2의 빈 io_workers/file-transport N_A와20초 total/18초 operation+2초 cleanup 구분을 확인했다. Request bytes/context를 재작성하지 않으며 endpoint 주입과 원 host 호출 수·픽셀/PNG CRC·원본/기존 출력 보존은 기존 시험이 검증한다. process exit/family Empty를 seek·미디어·landing 성공으로 바꾸지 않는다.

raw-assets.json은132개 실제 증거 파일의 path/bytes/SHA를 보존한다. request.json/owned stdin.raw, canonical stdout.raw·stderr.raw와 legacy stdout.bin·stderr.bin, process/receipt/failure 및 원 PNG/source/output 전후 사본이 포함된다. prefix는 offset/bytes/hash/complete를 canonical origin과 연결한다. 의도적인 evidence 쓰기 실패 사례는 해당 파일이 없을 수 있으며 시험에서 원 terminal receipt와 secondary 오류 보존을 검증했다.

## 소유·보호 마감과 한계

최종 source4/integrated4·snapshot 비중첩1364/root 추적1002의 hash 차이0이다. producer·SDK·등록 Cargo/lock은 이 보호에 포함된다. runner receipt5는 모두 terminal/exit0이며 기록 PID 수치의 현재 후보0, evidence/native 경로 프로세스0을 확인했다. PID 검색/reopen kill은0이다. receipt5/실제 CLI11 원 identity와 metadata는 final-receipts.json/cli-receipts.json/final-raw-pids.json/final-evidence-processes.json에 있다. 반환 시각은 closeout.json을 따른다.

Live는 실패 owner 보존 상태이며 child가 반드시 실행 중이라는 단언이 아니다. terminal receipt 기록 실패도 원 owner를 보존하고, 실패 closeout은 원 deadline을 갱신하거나 원 오류를 성공으로 바꾸지 않는다. 마지막 Incomplete/harness Drop은 ACK가 아니다. raw common cap과 stderr 최종8192 제한은 구분하며 기존 pipe의 조기 backpressure 동등성을 보장하지 않는다. 동기 FS/OS 호출 자체의 엄격한 wall-clock 종료도 미보장이다.

jobs fixture/playback_owner 전환·skip 소비·CLI ticket wire·Meari408 채택·실제 브라우저/미디어/AI/다운로드는 이 gate에 없다. 이번 구현·회귀·증거 경계에는 modoo.today code-change-review-verification/참조 verdict-table과 plan/plan-authoring의 관측·미검증/원 실패 보존 원칙을 적용했다. 정확 출처·적용 판단은 원 handoff에 보존한다.
