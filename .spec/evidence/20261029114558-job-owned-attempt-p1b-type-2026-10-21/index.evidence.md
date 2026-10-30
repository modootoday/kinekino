---
id: evidence:kinekino/20261029114558
title: P1b retaining host 초기 타입·CPU 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# P1b retaining host 초기 타입·CPU 검증

2026-10-21 root `768669b66ff25502d8cd557a4f7d95a04b2452bd`, Meari `0b79d57d7a59b3146b7ec72d02e2651b21b166fd`에서 검증했다. 최종 source35 SHA256 `630BE5E420756D45045D3A9384C55A9B0FDB6D02E21448F7F6C69EFEED14C6B6`, 원문과 실행 증거는 `runtime/verification/job-owned-attempt-p1b-type-r1/`에 있다. 이 기록은 전체 제품 또는 P1b 전체 인수가 아니다.

## 구현 경계

공통 jobs worker는 원 start CAS의 attempt/input/source/submission identity와 deadline을 결합해 no-launch session을 먼저 소유하고 실행한다. 원 session closure 관측 뒤 cancellation ACK 또는 publication CAS→candidate release→publish→verify→completion CAS로 진행한다. 원 실패의 진단 저장·재조회·CAS 실패와 adapter unwind는 carrier를 Pending으로 보존한다. provider/workspace의 retaining 타입과 부분 초기화 오류가 원 browser/profile/native 의존성을 보유하며 CLI dependency-only incomplete 경로까지 연결된다. 외부 AI operation 등록은 없다.

## 원 실패와 최소 수정

원 checkpoint35 `338E4CBB…64BCA9D`와 panic 후속3 `EE42993A…9115D7`를 고유35로 합성했다. 원본들은 보존했다. 초기 패키지 all-target strict는3108ms/0, 루트 lib/player-cli/jobs 소비자는15674ms/101로 Clippy8건이 발생했다. 부분 시작 오류의 owner와 workspace Complete를 Box로 보유하고 provider factory 함수 타입을 책임 있는 별칭으로 분리한4파일 수리 후 소비자 strict13301ms/0이다. allow로 억제하지 않았다.

원 패키지 시험72개는69통과·신규3실패였다. 세 시험은 원 Exited/Empty/EOF→Settled 변환 후에도 session Drop이0이라고 잘못 가정했다. 실제 계약은 종료된 session 해제를 허용하며, 미완료 원 carrier와 candidate를 유지해야 한다. 구현 수명을 늘리지 않고 시험1파일만 수정해 동일 Box 주소, 종료 후 session Drop1, durable 결정 전 candidate Drop0와 결정 후1, publisher0 및 first error를 확인했다. 첫 실제 실패 로그/실행 파일을 보존한다. 이전 후속 HANDOFF의 session Drop0 주장은 이 관측으로 정정된다.

## 성립한 검증

- 패키지 실제 등록72. 원 실행에서 변경 없는62개 통과, 최종 r3 실행 파일의 owned driver10개 통과로 고유72개를 검증했다. 최종 실행 파일 전체72 재실행이라는 뜻은 아니다. 새 panic3과 직접 영향7을 함께 재실행했다.
- 원 root lib/player-cli 시험 빌드는206214ms/0. 고정 BIN에서 provider3·CLI dependency3을 선택 실행해 모두 통과했다. 루트 전체636·CLI 전체25 시험 실행은 하지 않았다. 이후 패키지 cfg(test) 수리만으로 이 빌드를 반복하지 않았다.
- 최종 패키지 all-target strict2990ms/0, cargo fmt 및 변경35 rustfmt check 통과. 최종 root production은 앞서 통과한 consumer strict와 동일하다.
- 패키지 문서3: 실제 E0599 reserve_publication/clone 및 E0277 DeserializeOwned를 확인했다. retaining provider/workspace 문서2: 실제 E0599 shutdown을 확인했다.

최종 패키지 BIN `14F31C46878CFAE64C3FBF5FF66757BB9B182916318220CFDBBA038B8544DF56`/2925056B. 원 패키지 고정 BIN `CA747F5C…86E6DB`는 첫 실패 근거다. root BIN `06D0BA1B…7737BC9`/98437632B, CLI 시험 BIN `F9C1D985…54ECA44`/11862016B. 완전한 SHA·원/fixed 경로·fresh:false는 gate의 binary JSON에 있다. fresh:false는 Cargo가 실제 새로 컴파일했다는 필드다.

## 자원·보호·한계

기존 S/T, EEA9 conservative2·affinity3·BelowNormal·offline/locked/-j1/thread1을 사용했다. 최초 RAM29.69GiB/GPU5699MiB/C173.84GiB. 원 runner receipt21은 모두 terminal이며 원 실패2개를 제외한19개는0이다. 기록42PID 부재, 원 exec 회수/live0, root1513/S1513 비입력 차이0, index·기존 정상 CLI/SDK·고정 BIN4 해시 불변을 확인하고 heavy/shared를 명시 반환했다.

fake session/browser/lease, 실제 ordinary-file profile lock 및 기존 subtitle 파일 시험은 실제 Windows process family/EOF 또는 VAD 추론의 증거가 아니다. 전체 owned-host dispatch/queue/retry/lease 통합, 임의 Drop/abandonment, host finalization adapter panic, destructor panic/abort와 실제 VAD aggregate의 Send/guard/EOF 연계는 남아 있다. public DTO/정식 모델 manifest/원 음원 producer/실제 CLI AI 지원과 품질 검증도 별도 미완료다. [전체 SDK·CLI 계획](../../../plans/player/plan-006-sdk-gui-cli.md)을 유지한다.
