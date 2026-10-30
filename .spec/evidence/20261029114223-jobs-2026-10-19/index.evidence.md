---
id: evidence:kinekino/20261029114223
title: 공통 jobs와 명시 CLI host 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공통 jobs와 명시 CLI host 검증

2026-10-19. [jobs 계약](../../player/application/api/jobs.md)을 media-jobs context와 root의 얇은 API/CLI 조립으로 구현했다. 모델 없는 SRT/VTT executor만 지원하며, 전체 영상 처리·모델 기능 완료가 아니다. GUI 실행·입력·스크린샷은 이 기능 검증에 사용하지 않았다.

## 패키지와 제품 경계

media-jobs는 domain 상태/attempt/immutable input·application 명령/포트·파일 journal/source blob·owned worker host·기존 자막 유스케이스 executor adapter를 소유한다. root SDK/GUI에 역의존하지 않는다. rooted-directory/reparse/file/lock 정책은 명시 RootedJobPaths 포트로 기존 root 정책을 주입해 재사용한다. no-overwrite publication은 기존 공통 crate 한 구현을 사용한다.

oneshot --job-root는 get/list/watch만 제공하고 submit/cancel/retry는 unsupported다. --serve-jobs는 명시 worker와 current-user endpoint를 만들고 stdin EOF로 정상 종료한다. endpoint CLI와 typed ApplicationClient가 동일 application 정책을 사용한다. watch는 영속 event page이며 대기 stream/list 별칭이 아니다. 기본 backend는 unsupported를 유지한다.

## 실행과 원문

초기 패키지 검사3→6→10→15→16개는 각 당시 checkpoint다. 실제 파일 변환·원본 보존·source 변경·손실 NeedsInput·출력 충돌·worker cancel ack/publication 경합·owner lease·재시작 Interrupted·idempotency·명시 비동기 host를 포함한다. source/로그는 runtime/quality-review/jobs-package-*에 보존하며 제품 CLI 성공과 구분했다. r4의 최초 Clippy collapsible_if2건 원문과 최소 정리 후 통과도 남겼다.

제품 root 통합 근거는 `runtime/quality-review/jobs-sdk-r1/`이다. all-target check9.07초, 전체 workspace **184 pass/9 ignored**가 통과했다. 실제 `player-cli --serve-jobs`와 endpoint CLI가 변환·idempotency/충돌·readonly backend·cancel 완료 경합/잘못된 attempt·명시 retry/오래된 revision·watch 재연결·모델 미지원·정상 EOF·소유 host 강제종료 후 Interrupted/무자동 실행을 통과했다. SDK가 강제종료 직전 Running/publication 전을 관측한 후 해당 소유 child만 종료했다. 과거 project/proposal 실제 CLI 회귀도 같은 suite에서 통과했다.

첫 strict Clippy는 UI 프로젝트 조회 테스트의 field_reassign_with_default1건으로 exit101이었다. 원문 clippy-initial.log를 보존했고 UI 담당이 initializer만 수정했다. 제품 기능 코드는 바꾸지 않았다. 후속 strict Clippy·해당 자동회귀·최종 bins/pin은 별도 기록으로 이어진다. ignored9개를 성공으로 세지 않는다.

## 최종 자동 검사와 고정 실행본

최종 fmt/strict workspace all-targets Clippy11.61초·해당 Escape 회귀1개·bins/examples build0.63초가 모두 exit0이다. 전 범위184 pass/9 ignored와 별도 최소수정 회귀를 구분하며185개 서로 다른 테스트로 세지 않는다. 제품 기능 소스는 최초 전체 통과 이후 바뀌지 않았다.

CLI SHA는 `f9f8281268c3eec9936341ee29cb57eb673e0292a4f32b586313da5711f5210f`, GUI SHA는 `77b5e24ca02dbfadf2c7b45dead0b463b235c7d7836725b28360c34878995df1`이다. 최종 build와 후보 파일의 byte SHA가 동일하다. final-bin/final-pins.txt에 보존했다. 이 GUI는 실행하지 않았다.

고정 CLI SHA pre/copy/post 검사 뒤 compiled Rust integration test를9.70초/exit0으로 실행해 `actual-cli-r1/`에 요청·stdout/stderr/exit·입력·journal·실행파일 사본을 보존했다. 정상 EOF는 completed_total2/cancelled_total0/interrupted_total0 receipt로 종료됐다. 별도6,408,894-byte 합성 source의 worker Running/publication 전을 SDK가 관측하고 소유 host를 강제 종료했다. 다음 명시 host는 같은 job/attempt1을 revision3 Interrupted/output null/publication false로 읽었고 자동 재실행하지 않았다. 정상 종료와 강제중단 증거를 합치지 않는다.

## 실행 중 취소의 실제 CLI 보완 checkpoint

후속 `runtime/quality-review/jobs-sdk-cancel-r1/`은 전체 **185 pass/9 ignored**, all-target check7.77초·fmt·strict Clippy13.14초·bins/examples0.91초가 모두 exit0이다. 첫184개 실행과 새185개 실행을 합산하지 않는다. CLI SHA `3fefb4562708ac9d88e65687d34cb7d43d594c98669f8a1986160bd01933e27d`, GUI SHA `b1b82a3f9acbc932fb88ece0f0dcd77ce1ea74b7558b7f64a6c552d032cee5c8`를 고정했다. GUI는 실행하지 않았다.

actual-cli/lifecycle·cancel-before·cancel-after가 각각 실행 전/복사 후 SHA를 확인한 동일 CLI를 사용했다. 제품 CLI가 실제 named-pipe/공통 SDK에 submit/cancel/get을 요청하고 실제 SRT executor가 파일을 처리했다. SDK 서버 composition의 publication 직전/직후만 Rust test gate로 고정해 경합 순서를 결정했으며 제품 테스트 플래그·가짜 executor·GUI 입력은 없다.

cancel-before는 Running에서 CLI cancel의 Cancelling/requested 응답 뒤 worker ack의 Cancelled/output null·출력 미생성을 확인했다. cancel-after는 publication_in_progress/Running 응답 뒤 Completed와 실제 출력 SHA receipt를 확인했다. 두 경우 원문은 동일했다. 요청·stdout/stderr/exit·source/output·journal과 파일 SHA를 첫 실행부터 보존했다. 이전 terminal-only CLI 범위에 대한 공백은 이 별도 근거로 보완하며 과거 실행에 소급하지 않는다. 정상/강제종료 lifecycle도 같은 새 실행에서 통과했다.

## 남은 범위

초기 f9f828 CLI의 cancel은 terminal-only였고 실행중 취소는 후속3fefb4 checkpoint가 보완했다. 강제 OS file-I/O interrupt·미구현 frame/model의 취소나 모든 외부 엔진 종료를 이 SRT worker ack 근거로 확대하지 않는다.

frame jobs·다운로드·provider/영상/회차 공통 CLI 이관과 AI 모델 executor는 후속이며 현재 jobs task 지원으로 확대하지 않는다. 모델 source/profile/engine pin·exact I/O/raw/provenance·실제 process cancel/exit는 [AI CLI 인계](../../player/infrastructure/inference/research-followups/model-engine-cli-contract-handoff.md)에 따라 코어 준비·중앙 자원 배정 후 검증한다. GUI 단계는 모든 CLI 기능 인수 이후이며 이번 완료의 선행 조건이 아니다. ready stdout 실패/LocalHost 생성 실패의 실제 fault injection은 이번 CLI 시험에 포함하지 않았으며 RAII 소스 경로만 존재한다.

source-at-audit는 검사 시작 시점 사본이며 전체 compiler-input/process-memory attestation은 아니다. 최소 Cargo/Rust source whitelist만 추가했고 target/runtime/evidence/secret 공개와 stage/commit/push는 하지 않았다. 초기 syntax 오류 `for cue in1`은 fmt에서 잡아 `in 1`로 고쳤고 로그에 별도 보존했다.
