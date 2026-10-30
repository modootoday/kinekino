---
id: evidence:kinekino/20261029114343
title: Resource runner 토폴로지 진단 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Resource runner 토폴로지 진단 검증

2026-10-20 중앙 단일 실행 슬롯에서 독립 Rust 도구만 검증했다. 전체 모델 품질·제품 기능·성능 개선 인수와 구분한다.

원결과: [runtime evidence](../../../../runtime/verification/resource-topology-r1/). 구현 책임·읽기 리뷰·적용 스킬과 명령은 해당 경로의 `source-review-and-gate-plan.md`에 보존했다. modoo.today `modootoday/software-qa/perf-experiment-loop` 본문을 적용해 진단 사실과 병목·가속률을 분리했다.

| 단계 | 결과 | elapsed ms |
|---|---|---:|
| cargo fmt --all --check | 0 | 279 |
| cargo test --all-targets | 10 passed, 0 ignored | 1814 |
| cargo clippy --all-targets -- -D warnings | 0 | 830 |
| cargo build --bin resource-runner | 0 | 5271 |
| 새 실행기의 rustc --version child 1회 | 0 | 64 |

Cargo는 독립 manifest, `runtime/build-cache/resource-runner-topology`, locked/offline/-j1, 시험 thread1을 사용했다. 기존 canonical 실행기 SHA `692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463`가 네 Cargo 단계의 예산을 적용했다. 5 receipt 모두 affinity3/logical_cpus2/priority16384/Job descendants/terminal true/exit0이며 동기 child wait를 마쳤다. `owned-return.json`에서 receipt의 launcher·child 10 PID 부재를 확인했다. 후손 전체를 개별 조회한 증거는 아니며 Job 상속 계약과 구분한다.

새 고정 binary SHA `37eab315f04c413baa7b9a4c72bef494a716e12897a4aad18f576f387286a370`. 기본 실행기로 교체하지 않았다. 수정·새 소스 8개 전후 목록 SHA는 둘 다 `2cf9e4664de151edef9f485820b52a62b5714c7e59c97de0a9b9ebc941cf0b2f`다. 이전 `.spec/plans/player/resource-topology-source-pin-20261020.json`을 삭제하지 않고 동일 bytes의 runtime `source-before.json` 및 `source-after.json`으로 연결했다. Cargo.toml/lock도 불변이다.

직접 `--topology` 1회는 outer budget 없이 같은 PowerShell PID86284, cwd `D:\workspace\@myso-kr\com.kinekino.player`, shell affinity65535에서 실행했다. `topology-context.json`, `topology.stdout.json`, stderr와 exit 파일을 보존했다. exit0, group0, process/system/thread/allowed mask65535, core masks3·12·48·192·768·3072·12288·49152이며 각 SMT=true였다. 이 문맥에서 기존 기본 mask3은 같은 물리 코어의 두 logical CPU이고 비교 후보는5다. 후보는 적용하지 않았으며 기본 정책은 first_two다. 조회 결과를 다른 실행 문맥의 허용 범위나 성능 개선으로 일반화하지 않는다.

오류·불완전 topology·잘린/과대/unknown 레코드·group·부분 허용·좁은 thread scope는 순수 시험으로 확인했다. 실제 OS 조회 실패·multi-group·32bit 호스트를 실행한 것은 아니다. 신규 추론·모델 다운로드·worker 빌드·benchmark·전역 affinity 변경은 없었다. 전체 AI 목표의 사람 품질 검토·실제 정밀도/라이선스 및 제품 후속 연결은 남아 있다. 중앙에 명시 슬롯을 반환했고 추가 실행을 시작하지 않는다.
