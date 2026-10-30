---
id: evidence:kinekino/20261029114715
title: 제안 초안·채택 연결 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제안 초안·채택 연결 검증

기준 계약은 [제안 API 정본](../../player/application/api/proposals.md)이다. SDK 소유는 domain/application/API·Rust 파일 adapter·CLI·main 조립이며 presentation 수정은 UI 담당자의 별도 작업이다.

## 구현 상태

correction/cards/highlights의 source snapshot·생성 raw_text·수정 draft를 분리한다. task별 구조와 source cue 참조를 검증하며 모르는 시간 필드를 거부한다. malformed 수입은 needs_repair와 import_error를 유지한다. 유효한 사용자 수정본은 reviews 판정 없이 저장·명시 채택할 수 있다. 출력은 typed JSON revision/receipt이며 SRT·카드 렌더 파일이나 기존 프로젝트를 수정하지 않는다.

파일 adapter는 local absolute non-reparse root를 startup/CLI의 명시 옵션으로 받는다. root guard와 배타 writer handle 아래 최신 revision을 비교한다. revision별 단일 SHA 이름 파일을 create_new/write/sync_all로 기록하고 기존 버전을 덮어쓰지 않는다. 중간 디렉토리·부분 파일·SHA 불일치는 수동 복구가 필요한 오류이며 이전 성공 상태로 숨기거나 건너뛰지 않는다. OS 장애 후 디렉토리 entry의 영속성까지 증명한 것은 아니다.

수입 source_sha256은 호출자가 고정한 외부 원문의 식별자다. 이 기능이 임의 외부 원문 파일을 재수집·인증하거나 프로젝트의 최신 상태를 조회하지 않는다. target_source_revision은 저장된 불변 snapshot과 대조한다. snapshot과 생성 raw_text의 실제 직렬화 SHA 보존은 회귀에서 별도로 비교한다.

## 검증 근거와 한계

### 문서 source S2 및 실제9 CLI 수입 r3

실제 ASR source는 cue ID/시간 없는 문자열 배열이다. SourceSnapshot의 기존 cue wire는 유지하고, 별도 DocumentSourceSnapshot/DocumentTextRef/DocumentCorrection을 추가했다. selector·SHA·실제 원문 배열 pointer/order/text를 대조하고 model raw는 수정하지 않는다. 기존 잠금 sha2=0.10.9를 직접 의존으로 선언했으며 새 다운로드는 없다. SourceSnapshot은 untagged enum으로 v1 cue JSON을 그대로 직렬화하고 문서 source의 신규 document만 version2다.

SourceS2/공용 UI DTO의 첫 cargo check --all-targets를 통과한 뒤, 최종 **fmt --check / cargo test 155pass·9ignored(test-thread1) / clippy --all-targets -- -D warnings / 두 bin·example build가 통과**했다. `runtime/quality-review/proposal-sdk-r3/`에 source/tests/examples/Cargo 감사 복사와 각 원시 로그를 보존했다. 처음 CPU 조건에서 기다린 CIM/Rust의 서로 다른 측정창, report 부모 경로 누락 실패도 보존했다. 이후 사용자의 비추론 정책 정정(CPU60 조건 해제, freeRAM≥12GiB·디스크·최대2compile)을 적용했고 HMR 자연 terminal 뒤 freeRAM 약30.44GiB를 재확인해 BelowNormal/-j1로 단일 묶음을 실행했다. 기존 건강한 작업·앱·서비스·전원·VM/DPI는 변경하지 않았다.

고정 `examples/proposal_actual_import.rs` 실행 파일을 실제 실행해 **exit0/9개 모두 원문·원생성 SHA 보존/별도 CLI 재조회 receipt 일치**를 확인했다. actual-nine-r1/input-store는 원래9생성 raw와 needs_repair만 보존하고, technical-store는 별도 명시 scripted 수정→채택→fresh CLI 재조회를 기록한다. 기존 모델 응답 schema와 제품 typed proposal의 차이로 needs_repair인 것이며 9개 모두 JSON 문법 오류라는 판정이 아니다. 입력 assets/package는 read-only이며 사용자 품질 판정은0개, 품질 승인false, 새 추론·다운로드0이다. cards/highlights source의 원래 시간은 raw asset에 보존하고 proposal 검증 시간으로 승격하지 않았다. 옛 r2 S1 store도 새 CLI에서 schema1/receipt3을 성공적으로 읽었으며 자동 이관하지 않았다.

r3 고정 GUI SHA는 `8ad628c969c7aa9f580740d734e10655f97a55834161bac96b81fa4e05fd1779`, CLI는 `bc6483acf78e4a973b5716387e6c0ae59659ea8d365d7986f72941dec5594b00`, 실제 importer는 `e0db78ad9545f95327a1b9d4746cfa48dd5e4d8f448aef9da3488035a72d953c`다. UI 담당자에게 고정 bin과 원본 input-store를 인계해 별도 복제본의 실제 S2 인수를 진행한다. r164 GUI 관측은 해당 담당자의 별도 근거이며 이 CLI 통과로 대신하지 않는다. rendered 파일·프로젝트 최신 revision 검증도 여전히 미지원 범위다.

### 재시작 receipt 조회 보완 r2

UI의 실제 r1 인수에서 inspect가 document만 반환하여 저장된 acceptance를 재시작 후 복원할 수 없는 계약 누락을 발견했다. revision 숫자로 채택 여부를 추측하지 않도록 inspect만 StoredProposal { document, acceptance } 읽기 모델로 보완했다. create/edit/accept 응답과 기존 저장 파일 형식은 유지한다. UI 담당자가 typed worker와 복원 상태 회귀를 수정했다.

새 근거는 `runtime/quality-review/proposal-sdk-r2/`다. fmt --check, **150pass/9ignored**, clippy --all-targets -- -D warnings, 두 bin build를 통과했다. 실제 새 CLI subprocess의 inspect-after-restart와 새 SDK adapter의 읽기 결과가 저장된 receipt를 포함해 동일하다. 별도 actual-cli 보존 실행도 통과했다. source-at-audit와 final-bin을 새 경로로 보존해 r1 실행본에 소급하지 않는다.

r2 GUI SHA는 `05284974561addb3fbd973e9da8b284c2fe45607e2d09af08a288d62a31b670e`, CLI SHA는 `e9adca1f41268fd188cc99593590152a3aba03cfe146509b47d5728775b504a4`다. UI의 최종 고정 GUI 재시작 인수와 실제9 source 수입은 별도 후속 근거다. [UI 중간 실제 인수](../presentation/proposal-ui-cycle-2026-10-19.md)를 최종 r2 EXE 인수로 대신하지 않는다.

### 초기 구현 r1 이력

작업 근거 경로는 `runtime/quality-review/proposal-sdk-r1/`이다. 초기 전체 검사에서 backend unavailable인데 absent payload를 먼저 해석해 proposals.accept가 exit2를 반환한 discovery parity 실패를 발견했다. entry에서 Unsupported를 우선 반환하도록 최소 수정했다. 최초 실패는 `intermediate-failure.txt`의 관측 발췌이며 전체 원시 stdout으로 표현하지 않는다. 이후 148건/9ignored 전체 통과는 tests-first-complete.log로 보존했다. 최초 Clippy의 UI nested-if 경고도 clippy-first-failure.log로 보존했고 UI 소유자가 let-chain으로 최소 수정했다.

최종 source에는 직접 SDK로 4MiB를 넘는 데이터를 수입할 때 publication 예약 전 실패하도록 순서를 보완하고 해당 회귀를 추가했다. **최종 fmt --check, cargo test 149pass/9ignored, cargo clippy --all-targets -- -D warnings, 두 bin build 통과**를 각각 fmt.log/tests.log/clippy.log/build.log로 보존했다. source-at-audit는 동결한 src/tests/Cargo.toml/Cargo.lock의 복사이며 전체 compiler inputs attestation은 아니다.

final-bin/player.exe SHA는 `7bf52ac96f3c7e9d46ba1cb2c13edc36b343e0a96a63b8d977640f0e11afc5cc`, player-cli.exe SHA는 `49c78824c6b19eab7249d4d2d7d7a87d0aeb59bedb71a2d089fc82ec01c7388f`다. 최종 CLI의 별도 보존 실행은 actual-cli.log 및 actual-cli 아래 request/response/summary/store와 고정 실행 파일이다. 수입 오류→수정→채택→동일 receipt 재반환, 오류 코드, 지원/schema의 typed client 일치를 실제 subprocess로 확인했다. cards/highlights의 정상 typed 수입도 확인했다. 최초 합성 원문/생성 원문의 전후 SHA는 summary.json에 별도 비교했다.

실제 Rust 파일 adapter 회귀는 needs_repair→manual edit→accept, 동일 revision edit/accept 경쟁의 단일 승자, 원문/생성 SHA 보존, 잘못된 sourceRef·stale revision·다른 target revision 거부, 부분/변조 파일 거부를 다룬다. 응답 소실 재시도는 현재 저장된 같은 draft/source receipt를 다시 반환하고 새 출력이 생기지 않는지 검사한다.

실제 CLI subprocess와 typed ApplicationClient 비교는 합성 source를 사용한다. GUI의 실제 화면·입력 인수, 기존 실제 9개 모델 출력 연결, 모델 품질, 렌더 파일은 이 회귀 통과로 대신하지 않는다. 해당 관측은 담당자의 별도 근거로 연결한다. 새 추론이나 다운로드는 실행하지 않는다.
