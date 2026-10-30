---
id: sot:kinekino/jobs-2-2-2
title: 숏폼 원 Jobs 연결
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 숏폼 원 Jobs 연결

R160 제품 연결 WIP: CLI `--serve-jobs --job-root ROOT --clip-render-config FILE --clip-render-config-sha256 SHA`는 strict `clip-render-host/1` 로컬 설정 파일을 원 file/ancestor guard와 SHA로 고정한다. 설정은 project_root, source ID→절대 경로/container, render의 encoder/probe 경로·SHA/inventory SHA·temporary/artifact root·명시 색 source SHA 집합이다. 1MiB/256 sources 제한과 schema를 검증하고 파일 전체 SHA를 host capture에 결합한다. 요청 JSON은 이 설정을 공급하지 않는다. 원 guards는 실행기 수명까지 유지한다.

기존 subtitle 실행기는 registry에 한 번 등록하고 선택한 card/clip 실행기를 함께 등록한다. 기존 VAD와 card/clip의 동시 구성은 명시 Unsupported로 유지하며 누락시켜 성공으로 표시하지 않는다. SDK는 같은 clip project store를 공유하고 `submit_clip_render`로 기존 jobs.submit을 호출한다. direct production.render_short RPC는 아직 지원하지 않는다. jobs.submit schema는 설정된 operation에 ClipRenderRequest를 사용한다.

R160 실제 CLI 시험은 기존 고정 저장 프로젝트와 원 미디어·도구를 소비하고 CLI 제출 및 typed SDK 제출을 별도 작업으로 렌더한다. 기존 owned host/request harness의 동일 deadline과 정상 EOF/원 family 종료를 확인한다. 이 추가는 아직 컴파일·실제 실행 전이며 지원 완료가 아니다. VAD/card/clip 통합 전체, source probe의 제품 API 노출·프로젝트 작성 제품 여정·전체 취소/실패/재시작/ACK는 후속 범위다.

2026-10-28 중앙 PM, R158 private overlay WIP. 원 소유권·기존 WIP·첫 실패·보류를 유지한다. 최신 완료 기준선은 R157이며 이 연결은 검증 전이다.

`application/clip_render/job_request.rs`는 strict 저장 프로젝트 pin과 review 전용 요청, 현재 수동 renderer의 명시적 지원 범위를 검증한다. `infrastructure/clip_render_jobs.rs`는 기존 JobExecutor를 구현한다. prepare/capture는 저장 publication과 신뢰된 Rust host 설정의 SHA를 source blob에 고정하며 source probe·encoder·새 worker를 실행하지 않는다. capture는 batch admission에만 재사용되며 독립 실행기가 아니다.

원 worker의 execute는 원 attempt 신원과 전체 input/source blob SHA를 대조하고 같은 저장 publication/host 설정을 다시 읽는다. 실제 미디어 관측·encoder inventory pin·원 decoded frame 경계·후보 렌더·typed output verifier를 연결한 뒤 기존 ClipArtifactStore와 원 ExecutionControl의 단일 reserve_publication/begin_verification을 사용한다. manifest에는 기존 Jobs의 job/attempt/input/source/request 신원만 기록한다. ID는 원 job ID와 attempt로 분리하며 기존 partial/완료물을 덮어쓰지 않는다. postcommit 늦은 취소와 deadline 조정은 R157 발행 계약을 유지한다.

출력은 VerifiedJobOutput의 manifest path/SHA/bytes 및 review receipt다. quality_approved=false/color_verified=false를 유지하고 host의 명시 색 해석을 사용자 품질 승인으로 승격하지 않는다. native source catalog와 tool/output 경로는 Rust assembly가 공급하며 요청 JSON은 임의 경로나 색 선언을 주입할 수 없다. preset 출처 검증·자막·구간 crop 등 남은 기능을 지원으로 표시하지 않는다.

실제 시험은 기존 Sintel·고정 FFmpeg/FFprobe로 프로젝트를 만들고 원 retaining Jobs host에서 capture→submit→Completed→동일 key replay→원 owner 종료→read-only job 및 manifest pin 재열기를 확인하도록 작성했다. 아직 실행 결과가 없으며 CLI/제품 host 등록과 구별한다. 원 worker 실패/취소/ACK/재시작 전체 행렬은 후속 검증 범위다.

기존에 읽은 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100)와 code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용한다. R157 fixture publication 성공과 원 Jobs 실제 실행·제품 SDKCLI 노출의 근거를 각각 구분한다.
