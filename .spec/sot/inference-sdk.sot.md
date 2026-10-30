---
id: sot:kinekino/inference-sdk
title: Inference 도구 패키지와 공통 SDK 포트 경계
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Inference 도구 패키지와 공통 SDK 포트 경계

2026-10-19 PM 요청의 정적 검토다. 실제 패키지 이동·Cargo 변경·새 빌드·다운로드·추론은 수행하지 않았다. 아래 Cargo 관계는 현재 파일을 읽은 사실이며, 포트 이름과 분리 단계는 구현 전 제안이다. SDK 소유 세션의 contracts/publication/subtitles 추출 이후 PM과 실제 이동을 조율한다. 읽은 manifest 6개와 application source 2개 및 SHA 원장은 `.spec/evidence/verification/inference-package-boundary-static-2026-10-19/`에 보존했다. 이는 경계 표본이며 전체 compiler closure나 실행 검증이 아니다.

## 현재 패키지와 의존 방향

| 패키지 | 관측한 경계 | 제품으로 가져오지 않을 책임 |
|---|---|---|
| kinekino-player | root workspace가 media-contracts, media-file-publication, media-subtitles에 path 의존 | 연구 도구와 vendor 전체를 제품 dependency로 연결하지 않음 |
| player-engine-lab | 자체 workspace, publish=false. anyhow/json/sha2/tar/flate2 기반 source 준비·동결·외부 Cargo gate·보존 도구 | vendor patch, 컴파일/검사 실행, 연구 evidence snapshot |
| player-model-spike | root workspace exclude. 자체 Cargo manifest와 광범위한 미디어·ORT·HTTP·Windows 의존. diarization-jer-reference에 path 의존 | 여러 모델의 다운로드·측정·품질 표본·참조 구현·개발 산출물 |
| diarization-jer-reference | 자체 workspace. diarization-evaluation에 path 의존 | 참조 데이터 해석·평가 근거 생성 |
| diarization-evaluation | 자체 workspace. Rust 평가 계산과 음성 파일 의존 | 연구 평가 지표 계산 |
| player-runtime-controls-spike | 자체 workspace, publish=false. reqwest/json/sha2/Windows 기반 별도 R4 worker source | 실험 서버 소유·controls/commit/ACK 수집·연구 receipt 조립 |

위 표본에서 관측한 로컬 path DAG는 product → media-contracts/publication/subtitles, model-spike → diarization-jer-reference → diarization-evaluation이다. 후속 전체24도구 조사에는 diarization-der-reference → diarization-evaluation 및 jer-function-reference → diarization-jer-reference 소비도 확인되어 있다. engine-lab과 runtime-controls-spike에서 제품 SDK로 향하는 path dependency는 없다. 외부 mistralrs source는 engine-lab이 별도 프로세스로 빌드하는 대상이며 engine-lab 자체 Cargo library dependency가 아니다. 이는 Cargo manifest 관측이고 전체 런타임 호출 그래프 검증은 아니다.

독립 Cargo 경계가 책임의 완전한 분리를 뜻하지 않는다. model-spike의 넓은 dependency 집합은 다수 실험을 함께 담는다. R3 동결 worker와 evidence를 유지한 채, 향후 실제 채택된 adapter만 작게 추출해야 한다. runtime-controls-spike는 pending engine pin이며 source fmt와 offline lock 준비만 완료했다. test/clippy/build·실행 및 완성된 worker로 인정하지 않는다.

## 공통 계약의 제안

후속 source 갱신: 최초 조사 이후 media-jobs가 추출되어 JobExecutor/ExecutionControl·JobStore/owner lease·source blob·retry/cancel/recovery 계약과 실제 자막 CLI 근거가 생겼다. 아래 최초 시점의 jobs 부재 설명을 현재 상태로 재사용하지 않는다. 새로운 AI 상태 머신을 추가하지 않고 [현행 접합 검토](ai-executor-current-jobs-integration.md)의 기존 포트를 재사용한다. models registry/inference adapter의 미연결과 jobs 자체 부재는 다르다.

현재 application/api/capabilities.rs에 models.*와 jobs.* 이름은 등록되어 있으나 지원 판정에는 model registry/실행 scheduler 포트가 없다. application/model_review는 기존 결과 package/assets를 검증하고 사용자 결정을 기록하는 별도 기능이다. reviews 또는 proposals가 있다는 사실로 inference 실행·model install·job 재개를 지원한다고 표시하지 않는다.

| 책임 후보 | 공통 application 계약 | infrastructure 책임 |
|---|---|---|
| ModelRegistryPort | 안정 model ID, HF revision, license, precision, artifact SHA/bytes, 설치·검증 상태와 capability 조회 | 허용된 저장소 접근·다운로드·파일 검증·설치 publication |
| InferenceExecutionPort | typed task/input artifact reference, 고정 model profile, context/resource 요구, 취소와 typed 결과 artifact | 실제 engine/ORT adapter, protocol 변환, owned process·timeout·정리 |
| JobStore/JobExecutor 포트 | job ID, 상태 전이, revision, idempotency, event cursor, retry/cancel 결과 | 영속 저장·복구·작업 queue·프로세스 실행 |

이 이름은 새 공개 trait/schema가 아니다. 실제 추출 때 media-contracts의 기존 값 객체와 product application 유스케이스를 우선 재사용한다. contracts는 engine executable path, CLI flag, HTTP endpoint, vendor 타입, HWND 또는 GUI widget에 의존하지 않는다. application은 contracts/domain에 의존하고 engine별 infrastructure adapter가 application 포트를 구현한다. GUI와 player-cli는 같은 application client로 요청하며, GUI 이벤트 자체를 job executor로 사용하지 않는다.

입력은 임의 URL/셸 문자열 대신 검증된 source/artifact 식별과 작업별 DTO로 제한한다. engine flavor, model revision/precision, context cap은 결과 provenance에 결합한다. 환경 threads=4는 설정값이며 실제 CPU/pool cap 증거로 바꾸지 않는다. model 설치 검증·job 접수·engine 성공·typed 결과 검증·사용자 품질 승인은 각각 구별한다.

## 오류·취소·수명과 인수 조건

없는 adapter는 unsupported이고 실제 미설치 모델은 설치 상태로 표현한다. 잘못된 입력, stale revision, 검증 실패, 자원 대기, 실행 실패, 취소 요청/확정은 합쳐서 빈 성공 결과로 반환하지 않는다. queued 접수는 completed가 아니다. retry는 이전 job/output을 덮어쓰지 않는 새 attempt 식별과 원인 연결이 필요하다. 취소의 경합·종료 후 재요청·부분 output publication 규칙을 먼저 정한다.

영속 job ID와 engine process/session ID는 다르다. crash 후 completed로 추정하거나 남의 process를 종료하지 않는다. output은 검증·불변 publication 후에만 결과로 참조하고, 재시작 시 SHA/bytes와 상태를 확인한다. stdout 로그만으로 completion·실제 적용·정상 종료를 대체하지 않는다. raw evidence와 typed 결과를 별도 보존하며 type 오류의 사용자 수정은 원본을 변경하지 않는 proposal 경로를 재사용한다.

GUI/CLI에서 같은 task가 같은 model/profile/input pin·오류·revision 정책을 사용해야 한다. provider 부재, 잘못된 artifact, cancel/finish 경합, 같은 idempotency key 재요청, process 종료 실패, 재시작 후 상태와 output 재검증을 담당 세션이 확인해야 한다. 이 문서는 해당 테스트를 새로 실행했다는 근거가 아니다.

## 진행 중 R4와 유보 범위

기존 R4 vendor bundle은 자연terminal0으로 완료했다. source manifest 2d91345c6767bc1070d2cbb939953e580434b816180dfaaca4075bbbb694fbe4의 check/test/strict Clippy/CLI build 전부exit0이며 test는1388passed/0failed/5ignored다. 각 단계 뒤 source/fork pin 일치도 재검증했다. EXE ec3d9371632dc7b134c3655432a6fe585463dd2b84b768e7a59b8412af815156/99,183,104B와 원문을 native copy/hash로 별도 보존했다. worker pin·native gate·실제 R4 측정은 아직 없다. 새 cold worker compile과 AI 다운로드/추론/성능 측정은 CLI 코어 준비와 중앙 배정 이후로 유보한다. 사용자 품질 검토는 마지막 별도 단계다. 모델 기능·품질은 실제 Rust CLI/SDK 고정 입출력과 근거로 판정하고 GUI는 결과 표현·비교·접근성의 UI/UX 검토로 구분한다. GUI 결과 재생을 모델 기능 완료의 추가 gate로 만들지 않는다.

기존 결과의 [source·오류 계약](text-r3-import-error-contract.md), [catalog/episode 공통 API 이관 자료](catalog-episodes-playback-api-handoff.md), [R4 근거 경계](gemma270-runtime-control-evidence.md)를 함께 사용한다. 패키지 분리는 이 문서로 실행 승인하거나 구현 완료 처리하지 않는다.
