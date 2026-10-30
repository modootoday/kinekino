---
id: evidence:kinekino/20261029114443
title: 실제 CLI의 주입형 VAD 작업 소비 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 CLI의 주입형 VAD 작업 소비 검증

2026-10-21 root `34be4b3dc679de7cf5c66eaa18319e8ebc4efd97`, Meari `4331ff3ea5cad71a7159bdb9e3f86fbfccb16220`에서 실행했다. [admission43](vad-job-admission-2026-10-21.md)의 구체 executor와 공유 release-file fixture를 기존 retaining jobs→PlayerSdk→LocalHost에 주입했다. 기본 제품 registry나 `audio.vad` 지원을 활성화하지 않았다. 원 증거는 `runtime/verification/vad-cli-consumer-r1/`이다.

## 입력과 실행

별도 source4 SHA256 `BEC5C7EDD6DA5EDA909288252FC8C623609524E6ED59AB7D2B8D7DC6693F104B`, 기존3/신규1의 before4와 raw final-source4를 보존했다. 앞서 작성한 r1~r4도 보존했다. 원음원 변경의 상태 기대를 실제 admission 관측에 따라 typed NeedsInput/SourceExpired로 정정하고, 채택한 constructor 참조 한 줄을 계승했다. 기존 CLI supervisor에 raw 출력 저장·forced 관측과 기존 content_digest를 사용하는 선택적 CLI pin 대조를 추가했다. 제품 정책·프로세스 제어기를 새로 구현하지 않았다.

root lock721586F9를 그대로 offline/locked 사용했다. 새 dependency·lock 재생성·다운로드0이다. fmt, root CLI/vad_jobs strict1975ms/0, 선택 시험 no-run10318ms/0이 통과했다. 새 vad_jobs BIN `451D3A64BD9A548AEAC5D701312694CBC010E91E1CC1144A28F013F1AC8A7710`의 실제 목록에서 정확한 ignored 시험을 확인하고 `--exact --ignored --nocapture --test-threads 1`로 실행했다. 실제1pass/0fail/0ignored, 기존24개는 필터로 제외했다. 원 runner2116ms/0, 시험 본문2.09초이며 컴파일/실행 수리 없이 통과했다.

실제 CLI는 `5DB7B549361B55056E06CA68B9BAA1DE4C61C063103F19877A7A923A95AEA133`이다. 이 바이너리는 현재 Cargo 산출물과 캐시 일치(`fresh=true`)를 확인한 뒤 고정했고, 각 호출 전에 lowercase JOB_CLI_SHA256과 실제 CARGO_BIN_EXE 파일을 대조했다. 이번 시험 BIN은 새로 컴파일됐다(`fresh=false`). 기존 CLI3E13의 실행 근거를 승계한 것이 아니다.

## 관측한 동작

| 실제 CLI 호출 | 관측 |
|---|---|
| jobs.submit |공개 매개변수·원 음원 SHA·명시 키로 created=true, attempt1/queued 접수|
| jobs.get |두 번째 admission 중 원음원 변경으로 needs_input/source_expired, 직접 SDK record와 동일|
| 같은 키 jobs.submit |created=false, 같은 job/prepared SHA, 추가 admission 없음|
| 원 endpoint 종료·저장소 재개방 뒤 jobs.retry |새 host/endpoint에서 expected_revision으로 attempt2/queued 접수|
| jobs.get |attempt2도 needs_input/source_expired, 원 prepared SHA·음원 pin 보존|

접수된 queued와 종결 needs_input을 구별한다. 모든 응답의 prepared SHA는 `1bd2b3fc379540acd553733a6e2c6dcd2652756877f0d4a8a030bc6a1eb941c8`, 원 음원 pin은 `89ef1aff0ba6786123c6897dc2833d6d4d6fdcedef02164dced94dfd57f5fdc1`로 유지됐다. output=null/publication_started=false이며 attempt evidence 디렉터리는 비어 있었다. 원 파일·event6·descriptor/diagnostic blobs와 CLI evidence를 통째로 사본 보존했다. 이는 모델 실행 성공이 아니라 입력 변경을 안전하게 거부한 실제 CLI 소비 근거다.

실제 CLI child5개 모두 원 handle wait 종료0·forced=false·stderr0bytes·stdout1line이었다. 응답은1125/1335/1362/1101/1335bytes로 기존 reader cap보다 작아 stdout EOF까지 읽었고 stderr도 EOF를 확인했다. stdin writer와 stdout/stderr reader, supervisor thread join을 완료했다. 요청 파일은 직렬화된 invocation intent이며 별도로 관측한 child stdin 원문이라고 주장하지 않는다. `process.json`의 PID와 원 raw stdout/stderr를 유지했다.

첫 endpoint와 새 endpoint 모두 원 PendingLocalHost에서 pipe=Joined/sdk=Joined/dispatch=Completed, observation_expired=false를 관측했다. 그 뒤 각각 원 retaining host를 명시 종료해 receipt2와 first_error=null을 확인했다. 두 receipt의 cancelled/completed/interrupted totals는 모두0이며 작업 성공이나 취소 ACK를 뜻하지 않는다. 의도된 첫 작업 오류 SourceExpired는 durable record와 응답에 남았다.

## 종료·보호와 한계

원 runner receipt5개가 모두 terminal0이고 실행 도구의 원 세션도 종료 회수됐다. 기록PID15개는 마감 시 부재였다. 원 handle 종료·thread join·host receipt와 구별해 보조 근거로 기록했다. root1158/Meari456(기존 docs2 포함)/양 index/S 비입력/Meari 고정BIN14쌍 보호 차이0, 새 BIN 원본·사본 SHA 대조0이다. 의도된 vad_jobs 시험 BIN 재생성1은 별도로 기록했으며 원 D895는 admission gate의 immutable artifact로 보존했다. CLI5DB는 변경되지 않았다. heavy/shared 명시 반환 뒤 문서를 마감했다.

이 실행은 ordinary-file synthetic fixture와 실제 CLI/LocalHost/Rust jobs host만 사용했다. AI worker/runtime load·모델·브라우저·미디어·GUI·네트워크 실행은0이다. 실패/미완료 carrier 보존 코드가 작성돼 있으나 이번 성공 실행이 강제적인 panic/cleanup 실패/임의 Drop·abort를 실증한 것은 아니다. default operation 등록, 실제 AI 입출력·품질·전체 제품 기능은 계속 후속이다.
