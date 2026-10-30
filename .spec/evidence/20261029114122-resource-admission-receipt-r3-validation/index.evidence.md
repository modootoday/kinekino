---
id: evidence:kinekino/20261029114122
title: Resource admission receipt r3 검증 반환
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Resource admission receipt r3 검증 반환

PM 단일 CPU 배정에서 [동결 후보 source24](../../../../runtime/development/resource-admission-receipt-r3/source24.json) SHA3B9598A929A512EE6D2D52594571DE7700E882512CCB0AB414C23CB36DC2510F, parent22 SHA10DE258E… 및 delta9 SHA472E4238…를 확인했다. 독립 standalone Cargo2는 변경하지 않았다. C: 소형 도구 전용 target `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/resource-admission-r3-target`을 이번 배정으로 신설했으며 제품 S/T나 기존 runner를 교체하지 않았다.

원 증거와 반환: [C: gate](%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/evidence/resource-admission-r3-verification-r1/slot-return.json), slot SHA D32D21BD1C743B173380BD24DF52A75B71B3CB44EC8FF38F52FD5453E221CC1C. 현재 status=returned/writer0/pending0이며 CPU 슬롯은 prose 작성 전에 PM에 반환했다.

## 실제 수행 범위

| 단계 | 원 receipt | 결과 |
|---|---|---|
| cargo fmt --all --check | 02-fmt | terminal/exit0,216ms |
| cargo test --all-targets --locked --offline -j1 / test-threads1 | 04-test | terminal/exit0,2856ms; 실제 등록·실행51, passed51/failed0/ignored0/filtered0 |
| cargo clippy --all-targets --locked --offline -j1 -- -D warnings | 06-strict | terminal/exit0,1134ms |

각 단계 전 CPU 두 표본·RAM·C 여유·GPU 여유·명시 도구 writer 부재를 C preflight에 기록했다. 기존 고정 runner EEA9DEE6…의 conservative2/mask3/CPU2/BelowNormal/descendant Job을 사용했다. 기존 runner 자체 receipt의 admission intervals=[] 우회는 기존 동작이며 새 후보의 관측을 실행했다고 보지 않는다. 후보 PDH/normal binary/probe·중앙 자원 수치 설정은0이다.

신규 순수 시험6(Backend 순서/전 단계 실패/close-only, application 원인+close, receipt typed evidence·null/escaping/golden)이 실제 통과했다. inherited `real_owned_child_is_reaped_when_started_receipt_fails`도 명시 배정대로 실제 rustc --version 자식1과 기존 owned cleanup assertion을 실행해 통과했다. 원 test stdout은51 실행 목록과 결과를 보존한다. 별도 --list 재실행은 하지 않았다.

receipt 검증은 Rust의 std-only 정확 빈 JSON golden·escape/null·typed evidence 필드 시험과 기존 receipt3 시험 범위다. 외부 JSON parser로 전체 새 receipt를 roundtrip하거나 실제 PDH receipt를 생성한 검증은 아니다. ABI가 컴파일·링크됐어도 실제 counter 수집·지원·정책 headroom·read-only probe 성공으로 승격하지 않는다.

## 산출물·감사·예외

원 Cargo JSON의 profile.test=true executable1을 C gate/bin/resource-runner-tests.exe에 한 번 독립 보존했다.1133568bytes/SHA9B790282A3139533E6640D06FF234C3C7DF4EC79C7B2A2A829603E10EF734130. producer 원본·독립본 SHA/FI는07-test-producer.json에 있다. normal executable 빌드0, 독립 PDB 복사0, D: EXE/PDB 복사0이며 중복 산출물 보존을 하지 않았다.

09-final-audit.json: source24/before22 SHA·길이 차이0, 원 stdout/stderr/receipt9의 exclusive EOF, 기록된 runner launcher/Cargo PID6 부재, 명시 Cargo/rustc/clippy/rustfmt/link/runner writer0. inherited rustc 자식의 숫자 PID는 별도 기록하지 않았으며 원 Rust 소유·reaped assertion이 그 자식 정리 근거다. 전체 후손 PID 관측·임의 OS writer 전수 검사를 주장하지 않는다.

기존 runner는 입장 시 선택 hash를 확인했고 종료 시 크기/UTC ticks/FileID가 같음을 확인했다. 종료 fresh content hash는 재수행하지 않고 baseline SHA+fresh 선택 stat/FI 재사용을 명시했다. shared carry/cache 전체 재해시·D 큰 쓰기·기존 runner 복원/교체는 하지 않았다.

감사 작성 중 PowerShell 해시리터럴 오타는 parse 단계에서 실행 전 실패했으며08-audit-setup-correction.txt에 남겼다. 이후 첫 감사는 이미 test exe를 C에1회 보존한 뒤 `deployed runner stat/FileID changed`에서 멈췄다. 실제 baseline mtime은 ConvertFrom-Json의 System.DateTime이었다. 문자열과 직접 비교한 검사 오류였고 양쪽 UTC ticks/422912bytes/mtime2026-10-20T11:27:18.5219777Z/FI0x00000000000000000001000000462199는 동일했다. 08-runner-mtime-comparison-correction.json이 원 오류·실제 타입·양쪽 값을 보존한다. 정규화 비교 후 기존07 producer를 재사용해 감사만 마쳤으며 exe 재복사·제품 시험/빌드 재시도·원본 수리0이다. PM의 독립 stat 관측과도 일치하나 그것을 새 content hash라고 표시하지 않는다.

## 미완료와 적용 근거

실제 PDH/read-only probe·실제 정책값·정책 적정성/실디스크 매핑·전체 JSON 호환·현 runner rollout과 runtime throttle/cancel/kill은 미완료다. 제품 전체 goal 완료·AI 사용자 품질 승인으로 계상하지 않는다. 사용자 확인 원인 InventorySvc와 기존 I/O 관측을 구분하며 WSL/빌드를 원인으로 확정하지 않는다.

modoo.today에서 탐색·본문을 읽은 `modootoday/build-concurrency-guard/concurrency-policy`를 재사용했다(필수 참조 파일 없음). source와 단일 슬롯·worker/CPU/부모 fan-out·unknown headroom 경계에 적용했으며 출처/연구 기록 fingerprint는 [r3 HANDOFF](../../../../runtime/development/resource-admission-receipt-r3/HANDOFF.md)를 유지한다. Rust가 기능 assertion을 수행하고 PowerShell은 원 command/receipt/metadata/선택 파일 감사를 조율했다. 신규 외국어 구현·비 Rust 기능 검증 로직·추론·다운로드는 도입하지 않았다.
