---
id: evidence:kinekino/20261029113639
title: Gemma270 실제 호출 경로의 내부 trace 구현
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Gemma270 실제 호출 경로의 내부 trace 구현

2026-10-19. [관측 계약](generation-boundary-observation-contract.md)과 [호출점 연구](../../player/infrastructure/inference/research-followups/gemma270-internal-boundary-trace.md)를 바탕으로 진단 전용 opt-in recorder를 구현했다. 현재는 source 구현·컴파일 검사·보존 단계이며 **실제 trace 추론 및 full boundary gate는 미완료**다. 기존 baseline/fixed EXE 및 실제 결과를 교체하지 않았다.

## 책임과 실제 연결

기존 `diagnostics.rs` 아래 `boundary_trace.rs`를 연결하고 journal과 저장소 어댑터를 하위 파일로 분리했다. 새로운 `diagnostics/mod.rs`는 만들지 않았다. 전체 source 변경은 기존11개를 포함해21경로이며3개는 신규다.

| 책임 | 연결 |
|---|---|
| 요청과 렌더 | 정규화 messages SHA로 고정3case를 매칭하고 actual NormalRequest.id를 등록한다. 동기 processor 호출 한 구간의 RAII TLS로 실제 BOS 분기·UTF8/U32LE IDs를 관측한다. count Tokenize에는 scope를 설정하지 않는다 |
| admission과 sequence | 기존 검사 결과를1회 계산해 기록하며 request ID와 별도 Sequence.id를 연결한다. 거부와 sequence 생성/forward를 구분한다 |
| forward와 position | 실제 row 순서의 seq ID를 Gemma 전용 args와 ModelForwardContext로 전달한다. 실제 forward 입력·offset/context/hostposition·반환을 기록하고 기존 text_positions 결과를18개 attention layer에서 관측한다 |
| 샘플링과 마감 | 기존 is_done/add_token을 다시 호출하지 않는다. raw token vector 길이·구체 StopReason 전/후/최종variant와 응답전송 뒤 case 마감을 구분한다 |

opt-in은 trusted child 환경의 `SOURCE_GEMMA_BOUNDARY_TRACE_MANIFEST`다. 고정 trace-plan.json과 workspace/run label을 검증하고 create-new journal, manifest read-sharing lock, ancestor directory의 delete/rename 방지 handle 및 reparse 거부를 적용했다. 경로/원문을 HTTP 입력이나 운영 logger로 전달하지 않는다. 활성화하지 않으면 recorder/파일을 만들지 않는다. event1024개/직렬화8MiB 한도와 invalid/dropped accounting을 두고 journal lock 중 모델·await·파일 I/O를 호출하지 않는다. terminal snapshot은 별도 sink lock으로 저장한다. 실제 경로·오류·병행 안전의 추가 시험과 수집 artifact 검증은 별도 게이트다.

## source와 검사 상태

CLI/server를 포함한 `cargo check --locked --offline -p mistralrs-cli --all-targets --no-default-features -j2`는exit0/1분40초였다. 첫 CLI/server strict clippy는exit0/1분31초였다. 이후 추가한 Windows 저장소 시험을 포함한 전체 core 시험도 아래와 같이 통과했다. 후속Clippy와 CLI 빌드는 별도 진행 중이며 source check·단위시험·실제 trace 추론을 구분한다.

후속 전체core 시험은 **1379passed/0failed/5ignored**, 컴파일13분54초·실행14.36초였다. journal overflow/incomplete, Windows manifest read-lock/create-new, traversal/duplicate identity, 상세길이reason/U32LE, nested/panic TLS 복구5개 신규 시험을 확인했다. trace CLI build와 후속Clippy는 같은target 잠금 순서로 계속 진행 중이다. 빌드 중인 source는 아래 고정manifest 이후 수정하지 않았다.

[source manifest](../../../evidence/ai/engine-context-cap/gemma270-internal-trace-source-r1/manifest.json) SHA256은 `ef2f4f69ce76a1eece36c6355ec4cf1fbd7c9df73a8d09f459b9dfa6be291931`, tracked patch는 `ada5336a6f3c43e5bc3609edbfdb5dbcf14e0c12b106fb973bba30a5f2656deb`다. 기존18개 source before/after, 신규3개 before_absent/afterSHA, 설정·라이선스4개를 보존했다. source snapshot 도구는 untracked 목록까지 exact allowlist로 검사한다. 빈 원본 파일을 조작하지 않는다. tool 자체fmt/2tests/strict clippy/build를 통과했다. snapshot은 compiler 전체 attestation이나 실제 신경망 실행이 아니다.

## 동일 입력 준비

최신 입력은 아래의 r3다. r2는 초기 준비 이력이며 실제 engine 매칭에 사용하지 않는다.

[준비 manifest](../../../evidence/ai/generation-boundary-fixtures/gemma270-boundary-trace-input-r2/manifest.json) SHA256은 `25304da23150e4bd6bb82330b4c5e72f5f1ea07ca8769c29b3d57ffa7f90ddb7`다. [수정판 실제API manifest](generation-gemma270-context-boundary-fixed.md)의 고정2047/2048/2049 payload를 그대로 사용한다. requests.json SHA는 `1aa40075fc6eceab49e075ed8c56f554c8fd6b24b97820e6cb9faf84ddd4d5e4`, trace-plan.json SHA는 `cb617e42f54bec0c33a37e8d4d35f15f22221728747d2e471de5b2f4dd4ad4b6`다. 생성 budget2/ignore_eos true/truncatefalse/seed42/temperature0은 유지한다.

첫 준비 명령은 JSON0.0을 정수0과 직접 비교해 잘못 거부했다. neural0이며 실패 로그를 보존했다. domain의 as_f64 수치 판정으로 수정하고 정수0/실수0.0 허용·문자열0/품질budget256 거부 회귀를 추가했다. 수정 워커 SHA `d777ed690cc91c9ac3ef4e40553cad6c42dcb2891658404d61eb1cb0261a88da`는 compiler inputs203개/EXE1개 [선보존](../../../evidence/ai/model-spike-builds/d777ed690cc91c9ac3ef4e40553cad6c42dcb2891658404d61eb1cb0261a88da/build-manifest.json) 후 준비를 실행했다.63tests/fmt/clippy/locked offline build를 통과했다. 준비에는 모델 추론·실제render/IDs/position 관측0회다. 일반 server adapter는 해당 trace 환경변수를 명시 제거해 off 실행이 ambient 설정을 상속하지 않도록 했다.

## 다음 실제 게이트

진단 소스의 최종 CLI all-targets strict Clippy와 core+CLI all-targets strict Clippy도 각각 exit0으로 완료했다. 로그의50분05초와28분37초는 target lock 대기를 포함한 명령 전체 경과 시간이며 순수 컴파일 성능으로 해석하지 않는다. 원문은 위 core 검증 로그 디렉토리에 추가 보존했다.

2026-10-19 05시 KST 후속: 진단 CPU CLI 빌드는 exit0/3220.671146초로 완료했다. 새 EXE는98,991,616바이트, SHA256 `d56e22092f0f0b1a5a9c914f8b3d1d6e3bcde3062fe8329618a58dd5cb3f80b0`다. [EXE와 빌드 receipts 봉인](../../../evidence/ai/engine-cli-builds/gemma270-internal-trace-cli-r1/manifest.json) manifest SHA256은 `5cab94ba82a16311377fa0e0b8239274d12e9a86ec0366f1499b299dba551cc1`다. 기존 baseline/fixed EXE를 대체하지 않고 별도 engine ID `gemma270-internal-trace-cpu-r1`로 구분했다. 봉인 도구의 fmt/2tests/all-targets strict Clippy/locked offline build는 통과했다. 실제 journal 수집 워커와 고정 Revision 연결은 새 변경이며 자체 gates 진행 중이다. 이 시점까지 새 모델 추론은0회다.

완료된 core 시험과 입력 준비 게이트 원문은 [검증 로그 디렉토리](../../../evidence/verification/gemma-internal-trace-core-2026-10-19-r1/)에 별도 보존했다. core stdout SHA256은 `c36f5450cc3dd750a2e4ae7f73661c0a07f7a0ee7c5ff6117bd23e765f262778`이며 1379 passed / 0 failed / 5 ignored다. 이것은 실제 모델 추론이나 내부 추적 캡처의 통과를 의미하지 않는다.

입력 준비의 최신본은 [r3 manifest](../../../evidence/ai/generation-boundary-fixtures/gemma270-boundary-trace-input-r3/manifest.json) SHA `85009c8ea7f6267daca32b3e25b39463443c6f2c0219b31ed366e9e185296b95`다. 원래 requests SHA1aa40075는 그대로이며 trace-plan SHA는 `e3408fbfd0f10736856459875a499215f6170231b5aa8f7cce353718decc5e9e`다. r2의HTTP message 표현은 내부 Either::Left 태그와 달랐으므로 실제 engine 매칭용으로 사용하지 않는다. 현재 core는 serde_json preserve_order가켜져role→content 순서로 직렬화하므로, 준비기에서 구조체 직렬화로 그 순서를 명시했다. source의Either1.15.0 Serialize 파생과 CLI cargo feature tree에서 확인했으며 actual matching은 다음추론에서검증한다. 준비워커 `ac32932c0c337fab51b6dcf978a3b0539f7b232490224034941be29312604f35`는203inputs/EXE1 선보존,64tests/fmt/strictClippy/lockedofflinebuild를통과했다. r1거부/r2준비는이력으로보존하고고쳐덮어쓰지않는다. 새모델추론0이다.

R1 전체 core 시험·최종 strict Clippy·trace CLI 빌드·새EXE 선보존과 child-only server의 첫 실제 실행을 완료했다. 수집워커64tests/fmt/strictClippy/locked offline build 통과 후 같은3개 요청을 실행했고, [실제 trace](generation-gemma270-internal-trace-actual.md)의 normalized message hash 매칭도 확인했다. 준비 단계의 neural0 이력과 후속 실제 실행을 구분한다. 첫 journal은 invalid2/completefalse이므로 완결성 실패다. 다음은 원인 구분 진단·재수집과 shared validator 변환이다. 완전히 observer-off인 HTTP 응답에는 raw IDs와 상세StopReason이 없으므로 content 재encode로 동일성을 꾸미지 않는다. [terminal witness 연구](../../player/infrastructure/inference/research-followups/gemma270-terminal-witness-noninterference.md)에 따라 최소 witness 비교와 완전off의 미관측 범위를 구분한다. 전체 원래 연산 호출횟수·비간섭·저장실패 accounting을 검증하기 전 full gate를 통과시키지 않는다. 품질max256/3task·23역할은 변경하지 않으며 품질 검토 준비를 사용자 승인 대기로 표현하지 않는다.

## R2 오류 원인 구분 진단

06시42분 후속. R2 전체 core 시험은1380passed/5ignored/0failed(20.96s), core+CLI all-targets check와 최종 all-targets strict Clippy도exit0으로 완료했다. [세 gate 원문](../../../evidence/verification/gemma-internal-trace-diagnostics-core-2026-10-19-r2/)을 보존했다. CLI CPU build는exit0/3746.7042046초이며 lock 대기 포함이므로 성능 측정값으로 쓰지 않는다. 새 engine ID는 `gemma270-internal-trace-diagnostic-cpu-r2`, EXE99,019,776B/SHA256 `97a365ae4349d58fc56831ba94fe314ca47ba4f5252a20e87fd8e402d607ccc6`다.

[R2 실행 파일과 빌드 receipts](../../../evidence/ai/engine-cli-builds/gemma270-internal-trace-diagnostic-cli-r2/manifest.json)의 manifest SHA256은 `21da40af68db9a249ef312da520d69133989ddae04f8a93d1dcbfd9cc7fb3348`이다. build 도구가 고정 source를 전후 대조했고 snapshot 도구는fmt·2tests·all-targets strict Clippy·locked offline build 후 새 경로로 보존했다. R1 EXE·결과는 유지했다. 워커에 별도 Revision/명령을 연결했고 자체 gates·source/EXE 선보존 후 동일3개 요청을 실행할 예정이다. 이 체크포인트의 R2 모델 실행은0회이며 full boundary gate는 통과하지 않았다.

06시44분 후속. 새 워커 `d5b0daecb63131eb7d4a6ede8a8193feea87998360b01b3618a61291bd490195`는64tests·fmt·strict Clippy·locked offline build를 통과했고 [source204개+EXE1 선보존](../../../evidence/ai/model-spike-builds/d5b0daecb63131eb7d4a6ede8a8193feea87998360b01b3618a61291bd490195/build-manifest.json)과 [gate 원문](../../../evidence/verification/gemma-diag-worker-2026-10-19-r2/)을 남겼다. 첫 `gemma270-boundary-internal-diagnostic-r2` 시도는 CPU96.9231%/free33,451,905,024B에서 admission exit1로 중단됐고 추론 worker/server 실행에 들어가지 않았다. [자원 거부 원문](../../../evidence/verification/gemma-diag-admission-2026-10-19-r2/)을 보존했다.

다음 별도 label `gemma270-boundary-internal-diagnostic-r3`은 실행 직전 CPU47.7539%/free31,794,167,808B로 admission을 통과했다. worker/server가 실제 실행 중임을 PID로 확인한 뒤 [실제 진단 결과](generation-gemma270-internal-trace-diagnostics-actual.md)를 보존했다. 이것은 correctness-only 실행이고 성능 비교 적격은false다. 두 label과 R1 원시 실패는 덮어쓰지 않는다.

[R2 source manifest](../../../evidence/ai/engine-context-cap/gemma270-internal-trace-diagnostic-source-r2/manifest.json) SHA256은 `d61d063a6b06ff02093855737591f9f6bd4140d7f3b5c4f741f45279f3e1090d`, patch SHA256은 `e2c6344025365fda94112508de1c8d878a8b8848c0f8c4df16d88262a5509d18`다. 원래 R1 소스·EXE·실패 journal은 덮어쓰지 않았다. 모든 invalid 증가에 이유·bounded context를 연결하고, row 준비 phase/등록 상태/origin과 forward 진입을 연결했다. 최초 저장 실패의 operation chain·flush 횟수·mutation revision을 기록하며 기존 invalid 카운터를 유지한다. 알려진 flush failure도 complete를 차단한다. raw journal schema는2다.

원인 구분 초안의 all-targets check는 통과했고, 최종 regression 추가본은 전체 core 시험→최종 check→strict Clippy를 진행 중이다. 새 CLI 빌드는 이 소스 해시를 고정해 target lock에서 대기 중이다. 빌드 도구는 기존 단계와 분리한 `build-trace-diagnostic-cli`이며, 자체 fmt/2tests/strict Clippy/build를 통과했다. reviewed source를 빌드 전후 다시 확인하도록 보강했지만 이를 전체 compiler 입력 attestation으로 부르지 않는다. 아직 R2 진단 EXE의 모델 실행은 하지 않았다.
