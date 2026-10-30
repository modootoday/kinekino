---
id: evidence:kinekino/20261029114550
title: 실제 CLI episode activation 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 CLI episode activation 관측

## 고정 검사

`runtime/quality-review/episode-activation-cli-r1`에 source-product, lock, 로그와 실행 파일을 보존했다. fmt0, strict workspace/all-target Clippy17.47초0, workspace237pass/17ignored, bins0.78초0이다. test compile은2분55초다. lock SHA는 `0cf44760431b804cdf18afd50aa82f57b50579cfe1465f9f6d6e9ee7b1aa96cc`다.

CLI SHA는 `90ee2f192ab78068f43ba4393b7b578ecc45a4d806f7a4b12e5ea068f0932425`, GUI SHA는 `dab429b1728f19fd43c77d108a806e255279ab1db49c7fb0c97b8aca8a137ea6`다. GUI는 실행하지 않았다. test-first.log의 실제 Cargo jobs-46b3a58152122637.exe를 복사하고 --list로 새 exact case 존재를 확인했다. 고정 harness SHA는 `a28449f12d5c1cfc60b5f6f9c72756c06f5113c720ee31f967007b7dcd09c47c`다.

## 실제 관측

고정 harness의 `resolve_cli::actual_cli_resolve_preserves_watch_failure_and_raw_evidence`는 추가 compile 없이1pass/14.60초/exit0이다. 불변 episodes seed에서 첫 회차를 선택해 실제 CLI jobs.submit의 experimental episodes.resolve를 접수했다. 같은 idempotency key의 typed client 재요청은 기존 job을 반환했다. job은 NeedsInput, publication_started=false, output 없음, media_verified=false, native_playback=false다. stdin EOF 뒤 browser_worker_exited=true를 확인했고, 고정 CLI catalog.read_raw로 두 원문의 전체 바이트를 읽어 보존 파일과 비교했다.

초기 DOM136370bytes/SHA `b245d823884194bb9f74e49219e8b0f4714dac97b9b4fcfa4d235ad733bf306c`, metadata ID `a18774eedb6ed53bec73d996d0e85ad42f744b83df17fad86d24b7ac60eb942d`다. engine155.0.4283.62, generation1, native navigation ID2, 실제 main HTTP200/IsSuccess=true/WebErrorStatus0을 별도 진단으로 관측했다. DOM capture.status는 계속null이다. 요청33/차단2다.

고정 watch actuator 뒤 generation2의 **https://frame.example.test origin으로 main navigation 시작**이 관측됐다. 정확 private URL은 capture metadata에만 보존한다. URL 합성이나 선언 media-cdn URL의 fresh GET으로 얻은 결과가 아니다. 이벤트 원문152bytes/SHA `6d6749e9cff792aedaef6ab2f72e44ba7806e5593294ff0838422ba03457dbc2`, metadata ID `c472253f28ff1bd0117fffb6dc63b5e8d32032caa2469a26ff47f927776f95b5`다. 요청34/차단2다.

## 미검증과 다음 수정

현 executor는 generation 변경을 관측하면 즉시 evidence 경계에서 반환한다. 따라서 새 문서 completion/DOM은 기다리지 않았고, navigation 시 초기화된 snapshot에서 activation와 main_navigation은null, candidate_count0이다. 실제 actuator 호출 결과 callback·새 HTTP status·Frame-host player 문서·frame lineage·manifest/media prefix·native 재생 성공을 확인한 것은 아니다. 테스트 통과는 접수/세대 이동/원문 보존/정상 종료/CLI 재읽기의 인수다.

다음 변경은 이전 문서 activation 진단을 별도 불변 기록으로 보존하고, 새 세대의 completion 및 DOM을 bounded 대기한 후 보존해야 한다. 현재 source/binary는 UI bridge 소비 검사 동안 동결하며 기존 관측을 소급 보완하지 않는다. 최종 resolve 결과와 native playback은 여전히 미완료다.
