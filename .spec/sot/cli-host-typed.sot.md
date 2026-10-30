---
id: sot:kinekino/cli-host-typed
title: 카드 묶음의 명시 CLI host와 typed 제출
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 카드 묶음의 명시 CLI host와 typed 제출

R112 입력 규격 보완: schema(jobs.submit)은 실제 등록 operation마다 parameters를 선택하며, production.export_cards에는 공통 CardBundleRequest::schema를 사용한다. 프로젝트 document_id/publication_revision/publication_sha256, frames의 artifact_id/path/png_sha256, output_directory, review_artifact_only가 필수다. 중첩 객체도 알려지지 않은 필드를 허용하지 않는다. publication revision은 양수 u64 십진 문자열, SHA는 소문자64자리이며 실제 저장 publication/프레임/경로 검증은 admission이 수행한다. 현재 review_artifact_only=true와 제안 템플릿만 허용한다. JSON schema 자체가 파일 실재·권리·품질을 입증하지 않는다. 기존 자막 operation schema와 카드 없는 host 동작은 보존한다. 실제 R112 CLI 검증은 진행 중이며 R111 통과와 구분한다.

2026-10-28 R110 연결 계약. [Jobs 실행/발행](card-bundle-jobs.md), [실제 자산](card-bundle-file-adapter.md), [편집 프로젝트 API](card-project-sdk-cli.md)를 같은 공통 application으로 연결한다. 모델/GUI/네트워크 실행 없이 저장된 프로젝트와 명시 PNG를 입력으로 받는다.

card-review-export 제품 구성을 빌드한 CLI에서 다음 host를 명시 실행한다. 두 root는 기존 로컬 절대 디렉터리이며 별도 역할을 가진다.

```text
player-cli --serve-jobs --job-root ABS_JOB_DIRECTORY --card-project-root ABS_CARD_PROJECT_DIRECTORY
```

card-project-root가 없으면 기존 자막 host 구성을 유지한다. 해당 feature가 없는 CLI에 root를 지정하면 Unsupported로 거절하며 renderer를 다운로드/자동 설치하거나 일부 기능을 조용히 누락하지 않는다. 현재 VAD와 카드 root를 함께 지정하는 composition은 Unsupported다. VAD 단독 기존 경로는 보존한다. 혼합 registry 구현은 별도 남은 범위다.

구성된 host는 기존 SubtitleJobExecutor의 subtitles.convert/export_range와 CardBundleJobExecutor의 production.export_cards를 하나의 ExecutorRegistry에 등록한다. 원 retaining host·exclusive Jobs execution owner·current-user endpoint·stdin EOF 종료 경계를 재사용한다. ready의 execution_operations와 card_bundle_configured가 이 구성을 표시한다. SDK에는 같은 Arc<CardProjectStore>를 공유하므로 card_projects.create/inspect/save_revision도 같은 endpoint에서 사용할 수 있다. 원 단독 명령/기본103 카탈로그 수를 늘리지 않는다.

실제 작업은 JSON Request의 jobs.submit으로 제출한다. context.idempotency_key가 필수이며 payload.operation=production.export_cards, payload.parameters=CardBundleRequest다. execution_timeout_ms는 Jobs의 독립 실행 예산이며 RPC context.timeout_ms가 아니다. expected_source_sha256는 고정 준비 descriptor SHA다. jobs.get/list/watch/cancel/retry는 기존 수명 계약을 유지한다.

ApplicationClient의 object-safe submit_card_bundle(context, input, optional_execution_timeout_ms)은 같은 jobs.submit 유스케이스로 직렬화한다. UI 이벤트나 CLI 전용 renderer/저장 정책을 만들지 않는다. direct production.export_cards RPC는 원 durable admission/owner를 요구하기 때문에 여전히 Unsupported이며 작업 제출을 즉시 PNG 생성 응답으로 위장하지 않는다. CLI에서 기능을 쓰려면 원 host의 endpoint로 jobs.submit 요청을 전송한다. 별도 host 자동 생성/재시도/endpoint fallback은 없다.

기술 결과는 review_artifact이며 품질/템플릿/원미디어 검증을 자동 승인하지 않는다. 고정 PNG hash와 저장 cue snapshot은 실제 AI 품질/미디어 PTS/권리 인증과 구분한다. 작업 성공과 전체 카드 기능의 인수는 별개다. 부분 성공 재사용/실패만 retry·staging cleanup·발행 후 ACK 복구·강제 중단/disk/permission·빈/다중 템플릿/GUI·사용자 품질은 남는다. 기존 WIP·소유권·원 증거/private overlay·C-only/heavy1·보류 유지.
