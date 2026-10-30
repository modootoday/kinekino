---
id: sot:kinekino/diarization-metrics
title: Diarization metrics 추출의 소유 범위와 소비자 회귀
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Diarization metrics 추출의 소유 범위와 소비자 회귀

2026-10-19 PM 인계에 대한 정적 검토다. 실제 추출·manifest/lock 수정·컴파일·추론은 하지 않았다. 기존 R4 건강 bundle 자연 완료와 근거 보존을 먼저 수행하며, 공유 root Cargo는 SDK와 교대할 때만 변경한다. 전체 도구 분류는 `.spec/sots/architecture/tools-package-promotion-research-2026-10-19.md`, 순서는 `.spec/plans/architecture/tools-package-promotion-2026-10-19.md`를 따른다.

## 최초 소유 범위

`tools/diarization-evaluation/src/domain/{time,intervals,assignment,metrics}.rs`의 순수 계산을 private `media-diarization-metrics` 후보로 한정한다. application의 Request(PathBuf), 파일 cap/read/hash/new-output, WAV 검증, RTTM/UEM 파싱, hypothesis provenance와 report publication은 기존 도구에 남긴다. runtime scorer·download·Perl/Python oracle·reference pins·model-spike 전체를 옮기지 않는다. 실제 라이선스 출처와 notice 확인 전 새 license 값을 추정해 넣지 않는다.

첫 단계는 계산 source·공개 signature·Scores/Mapping 필드·오류 의미를 보존하는 기계적 분리다. 기존 evaluator의 domain 모듈 경로는 명시 re-export하여 두 reference 소비자가 동일 Rust 타입을 쓰도록 한다. 새 package가 old evaluator나 제품 SDK를 역참조하지 않으며, evaluator → metrics로 향한다. DER/JER reference는 기존 evaluator의 입력/file adapter를 여전히 필요로 하므로 단순히 evaluator dependency 전체를 metrics로 치환할 수 없다.

현재 pure API는 anyhow Result와 Serialize DTO이고 독립 typed Protocol 타입은 없다. report의 protocol은 application JSON이다. 첫 이동과 동시에 새로운 오류 타입·가변 Protocol·새 float 알고리즘을 도입하지 않는다. 명시 profile 타입이 필요하면 기본 RTTM grid와 frozen legacy endpoint grid를 구별하는 후속 호환 설계로 검토한다. package semver와 기존 report schema는 서로 다르다.

## 보존해야 할 실제 계산 계약

| 경계 | 현재 source 사실·유지 조건 |
|---|---|
| time/interval | decimal microseconds, 소수 최대6자리·지수/음수 거부, one-day 상한. Interval은 start<end, 반열린 구간. 일부 필드는 pub이므로 evaluate가 입력을 다시 검증하는 동작도 유지 |
| UEM/정규화 | 같은 화자 구간 union·인접 구간 병합, UEM과 reference/hypothesis 양쪽 교차. 최대32화자·각 입력4096구간/UEM4096 제한. 미해결 화자로 reference/UEM을 제외하지 않음 |
| DER | 연속 microsecond 경계, collar0·overlap 포함. Hungarian maximum cooccurrence 전역 할당, speaker-time 분모와 miss/false alarm/confusion, 100% 초과 DER 허용 |
| JER | 0.010*k binary64 left-search grid, UEM explicit endpoint. 기본 evaluate는 RTTM onset+(end-start) 각각 f64 변환 후 덧셈. legacy 함수는 end 직접 변환. 서로 대체하거나 integer-grid로 정리하지 않음 |
| JER 할당 | DER와 별도 Hungarian minimum 1-IoU, dummy cost1, reference별 평균, 0 frame reference는 오류. 최대60초·6000grid. BTreeMap 화자 순서·mapping·contingency matrix 포함 |
| assignment | 정방행렬1..32, maximum duration 비음수 bounded i64, minimum finite0..1. pathfinding=4.16.0·ordered-float의 tie/순서와 기존 dependency 고정 유지 |

synthetic_metrics의 실제 source에는 같은 구간에서 기본 JER frame941 대 legacy940을 기대하는 회귀가 있다. 이는 이번 테스트 실행 결과가 아니다. 기존 float 비교 tolerance 1e-8과 정수 frame/mapping 정확 비교를 유지한다. 알고리즘 이동 후 숫자가 같아도 report의 executable SHA는 새 빌드에서 달라지므로 historical 원문 전체 bytes 동일성을 주장하지 않는다.

## 소비자와 변경 지도

| 대상 | 최초 변경 후보 | 유지/회귀 |
|---|---|---|
| 새 metrics | 위4모듈·최소 anyhow/serde/pathfinding/ordered-float 의존, publish=false | 계산 fixture·실패 경계, 모델/OS/파일 의존 없음 |
| diarization-evaluation | dependency 추가와 domain re-export, 기존 application 호출 유지 | synthetic_metrics/inputs/cli. schema diarization-evaluation/0.2.0·protocol/한계·nooverwrite 유지 |
| diarization-der-reference | 원칙적으로 import 변경 불필요 | frozen5202 schema0.1.0와 EVALUATOR_SHA admission, raw report/source/runtime pins 불변. inputs/render/output가 기존 evaluator 타입/adapter 사용 |
| diarization-jer-reference | 원칙적으로 import 변경 불필요 | legacy 재계산으로 원본 scores 대조 후 corrected 기본 계산을 별도 보존하는 흐름 유지. external parity false를 임의 승격하지 않음 |
| model-spike, jer-function-reference | 기존 jer-reference 경유 소비 | path DAG와 wrapper compile 회귀. 고정 R3 EXE/source archive는 변경하지 않음. 실행이 필요한 신규 inference/oracle는 이번 검사에 포함하지 않음 |

DER/JER reference는 0.1.0 frozen report를 명시 요구하며 evaluator의 현재 writer는0.2.0이다. 이것을 오류로 보고 원문 schema/pin을 일괄 갱신하면 안 된다. JER inputs는 legacy 재계산과 corrected scores/contingency를 분리한다. 기존 raw 실패·caller-supplied 관측·runtime gate 및 external parity 한계도 유지한다.

## 실행 전 회귀 계획과 중단 조건

공유 Cargo lease와 검사 슬롯을 PM/SDK와 확정한 뒤 이동표·원본 source/lock SHA를 보존한다. 먼저 pure package와 evaluator의 fmt/test/strict all-target Clippy를 실행하고 DER/JER reference의 synthetic/precision/derivation/contracts를 같은 freeze에서 확인한다. model-spike/jer-function-reference는 영향 경로 compile과 순수 기존 tests 범위를 별도로 합의한다. 외부 oracle 실행·모델 다운로드·추론은 시작하지 않는다.

같은 synthetic fixture의 정수 분모·frame·matrix·mapping은 정확 비교하고 float는 기존 의미와 IEEE 연산 순서를 유지한다. 실제 frozen report replay는 기존 artifact와 pin을 그대로 사용해 available/offline 범위에서 별도 수행하며, 실행하지 못한 항목은 미검증으로 남긴다. 새로운 CLI 결과는 기존 source/audio/reference hash·schema/protocol/scores·실패 종류를 대조하고 새 executable hash만 새 provenance로 기록한다. 새로운 decoder로 historical 원문을 다시 써서 덮어쓰지 않는다.

소비자 gate 실패, float/grid/tie 차이, schema/오류 변화, source snapshot의 새 path 누락 또는 라이선스 미확정 시 승격 완료로 표시하지 않는다. 다른 세션 작업이나 원시 evidence를 reset/delete하지 않고 해당 이동 단계만 복구한다. 실제 구현 완료·제품 AI 지원·품질 승인은 이 정적 검토의 범위가 아니다.
