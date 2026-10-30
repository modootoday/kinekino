---
id: evidence:kinekino/20261029114727
title: ResolveEpisode 포트의 실제 문서 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# ResolveEpisode 포트의 실제 문서 관측

상태: tests/jobs/episode_capture.rs 준비·fmt 통과, 컴파일/실제 관측 전. 이 검증은 기존 공통 Rust provider가 노출한 Running.document를 수집하는 중간 SDK 관측이다. 제품 CLI episodes.resolve 지원이나 미디어 재생 성공을 대신하지 않는다. 제품 source/bin 변경0이며 [series 실제 CLI 근거](series-cli-2026-10-19.md)의 불변 결과를 입력으로 쓴다.

## 입력과 범위

EPISODE_SEED_ROOT는 runtime/quality-review/series-cli-r1/actual-cli/provider-series/catalog, EPISODE_SEED_RESULT_SHA256은 `b1a2af4afa4491ef5743f2a1095d7b9973b6bc4268f1edf70dd9482446bea3bc`다. 공통 CatalogResults.read가 참조 raw/result/source SHA를 검사하고 기존 순서의 첫 episode URL을 공통 PageLocation/ResourceKind로 검증한다. 새 series/category 요청이나 임의 링크를 만들지 않는다.

별도 private profile·catalog roots에서 기존 browser_provider::start와 BrowserOperation::ResolveEpisode를 사용한다. document를 얻으면 공통 CatalogArtifacts.save로 원문 SHA/metadata를 게시하며 parser_revision=browser-dom-uninterpreted-v1·DOM/status=null로 기록한다. 파싱/해석을 실행했다고 표시하지 않는다. 고정된 snapshot의 ticket/document generation·실제 navigation/engine/filter/count를 보존한다.

## 수명과 판정

문서 또는 terminal 상태·deadline까지 관측한 뒤 cancel disposition과 실제 후속 상태를 구별한다. Requested/AlreadyRequested이면 worker Cancelled를 확인하며 AlreadyTerminal을 새 취소 acknowledgment로 바꾸지 않는다. owner.request_shutdown 뒤 worker_exited를 확인하고 receipt를 남긴다. 여기에는 CLI host stdin EOF가 없으므로 이 종료를 EOF 검증으로 기록하지 않는다. GUI player.exe·사용자 브라우저를 실행/종료하지 않는다.

정상 종료 후 기존 v3 고정 CLI catalog.read_raw로 모든 chunk를 이어 SDK에서 보존한 문서와 대조한다. 이 CLI raw readback과 SDK capture를 합쳐 CLI resolver 성공으로 표시하지 않는다. 실패 시 state/failure·가능한 raw evidence와 stop receipt를 남기고, 확인된 DOM을 기반으로 Rust extractor/명시 staged navigation 및 [수명 ADR](../../../adrs/player/browser-media-document-lifetime.md)을 검토한다. 실제 redirect/POST/iframe/worker 구조에 따라 ADR을 재평가한다.

## 필요한 검사

제품 source 변경 없이 새 ignored Rust case만 추가했으므로 예약된 SDK 슬롯에서 fmt/strict workspace all-target Clippy/jobs --no-run으로 새 harness를 만든 뒤 실제 SDK case를 한 번 실행한다. 기존 정상5개를 의미 없이 반복하거나 새 모델 추론/다운로드를 실행하지 않는다. 현재 문서 준비를 관측 성공으로 세지 않는다.

## r1 최초 관측의 fixture 결함

runtime/quality-review/episode-dom-r1의 fmt0·strictClippy1.31초0·jobs --no-run6.12초0 뒤 source-fixture와 별도 jobs-harness.exe/SHA를 고정했다. 실제 case는3.33초/exit101이다. port.poll의 일시 ResourceExhausted(Browser state is busy)를 새 fixture가 unwrap하여 panic했다. 이는 사이트/파서 실패나 CLI resolve 실패 근거가 아니며 원문 취득·취소 acknowledgment·정상 owner 종료의 성공도 기록하지 않는다. seed/profile/원 로그/고정 harness는 보존했다.

후속 test-only 수정은 busy 오류만 기존 호출의 남은 deadline 안에서 짧게 기다리고, 다른 오류를 즉시 보존해 실패시키며 job이나 navigation을 재시도하지 않는다. 순수 busy→성공/비busy오류 거부 회귀도 추가했다. 제품 source/CLI는 바꾸지 않는다. 새 r2 관측은 이 수정의 검사 후 별도 증거 경로에서 실행하며 r1을 덮어쓰지 않는다.

## r2 관측과 종료

r2는 ResourceExhausted 중 구체 adapter의 Browser state is busy 진단만 기다린다. ticket/active-work 한도와 EngineFailed는 한 번의 호출로 오류를 보존한다. tests/jobs/observation_guard.rs는 panic 중 자기 ticket 취소·소유 owner 종료·worker_exited/미관측 receipt를 기록하는 RAII guard다. 순수2회귀는 busy/한도·오류 구분과 합성 owner의 panic cleanup wiring만 증명하며 실제 COM/OS 종료의 증거가 아니다. r1 profile 인자가 있는 WebView2 프로세스는 후속 조회에서 없었지만 r1 정상 종료 receipt 부재는 유지한다.

runtime/quality-review/episode-dom-r2: fmt0, strictClippy1.36초0, 새 harness compile5.27초0·순수2pass(0.01초). 최초 request 지역변수 shadow 진단은 이름을 고친 뒤 해소했고 clippy.log와 clippy-fixed.log를 구별한다. 고정 harness SHA는 `77c1a3a33c41d669969cbfd39d17446d606d7ba35d50e891523d4b019ea0420d`다.

실제 SDK case는1pass/11.60초/exit0이며 DOM136159bytes/SHA `f98c71a4e95bb47fe9f26455c138a0e9595c51d45351cdf8afd53d35a35f10f9`, metadata ID `927b24447b93da0a47df76597a96f65f0312382abc80b2939da913aacb51a1c2`를 보존했다. Running.document·document/request generation1, WebView2 155.0.4283.62·requested34/blocked2, DOM/status=null이며 media_resolved=false다. stop.json은 Requested→Cancelled·host Stopped·worker_exited=true를 기록했다. 고정 v3 CLI에서 전체 raw chunk를 이어 원문과 대조했고 seed 결과를 다시 검증했다. 이 owner shutdown은 CLI stdin EOF가 아니다.

## 보존 DOM 후속 해석

문서에는 button.watch-now와 script[data-page='app'][type='application/json']가 있다. 직접 iframe/frame-host 이동값은 아직 해석 전이며 추정 URL을 만들지 않는다. 현재 media-catalog의 기존 의존성만 사용하는 offline Rust example inspect_episode_data를 준비해 원 DOM SHA를 확인하고 field pointer·값 타입/길이/SHA·HTTPS origin만 출력하도록 했다. 실제 값·URL path/query·token은 출력하지 않으며 아직 이 example의 컴파일/실행 전이다. 이 구조를 확인한 뒤 공통 CLI resolver와 최소 staged adapter를 구현하고 cookie/POST/iframe 호환성을 별도로 검증한다.

후속 offline r1은 fmt0/Clippy7.16초0/비밀값 비출력1pass·DOM run0, r2는 fmt0/Clippy1.50초0/1pass·DOM run0이다. 각각 episode-data-r1/r2에 source/log/구조 출력을 보존하며 새 네트워크0이다. r2 structure SHA는 `86d1cd217a2cc30fd411cee3913fb11a69fa71077b46a1b357892e4ba1ec5f21`다. 실제 /props/playing/body의 video·iframe=true와 media-cdn /video/32hex·query 없음 형태를 확인했고 opaque 값은 출력하지 않았다. [공통 application 계약](../../player/application/episode-resolve.md)의 Rust 정책/decoder와 player-document 목적 소스 준비로 이어졌으며 이 제품 변경은 아직 컴파일/실제 이동 검증 전이다.
