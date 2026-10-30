---
id: evidence:kinekino/20261029114640
title: episode 대상 정책과 실제 player 문서 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# episode 대상 정책과 실제 player 문서 관측

## 고정 제품 checkpoint

runtime/quality-review/episode-resolver-r1: fmt0·strict workspace/all-target Clippy33.77초0·workspace231pass/16ignored(compile3분10초)·두 bin build0.86초0. root138pass/1ignored, media-catalog16pass, jobs7pass/7ignored를 포함한다. source-product는 root src/tests/examples/assets와5개 package source·examples·manifests/lock/.gitignore를 보존했다. lock SHA는 `0cf44760431b804cdf18afd50aa82f57b50579cfe1465f9f6d6e9ee7b1aa96cc`다.

고정 CLI SHA는 `4d39021f85b81b431bf849c60b8b03307a81ad5554c35110b0b4d33adb0ed692`, GUI SHA는 `e70f7fc91f4d22f801f8612a2e0818f4d72cba2a00e8101303f2cbd453f8a621`다. GUI 제품은 실행하지 않았다. 새 dependency 경계로 Cargo harness 이름이 변경되어 초기 cache 이름 하드코딩 copy가 이전 harness였음을 **실행 전에** 발견했다. 최초 binary-sha256.txt/jobs-harness.exe는 새 관측의 유효 근거가 아니다. test.log가 실제 사용한 jobs-46b3a58152122637.exe를 jobs-harness-final.exe로 고정하고 --list에서 새 case를 확인했다. 유효 final harness SHA는 `18731d37db8dad9878c8276e126d93da1f72c6bdfe48835782aae0f4d9241699`, 정본 hash 기록은 final-binary-sha256.txt다. 초기 copy는 실행하지 않았으며 건강한 compile을 반복하지 않았다.

## 실제 관측과 의미 실패

고정 final harness의 actual_player_document_is_selected_by_rust_from_preserved_episode는1pass/8.90초/exit0이다. 불변 episode 원문을 읽고 공통 application/실제 Rust decoder가 requested/observed/embedded identity·video/iframe·정확 media-cdn document URL 정책을 검증한 뒤 fresh profile로 문서를 읽었다. 추가 compile 없이 실행했으며 기존 source/episode 원문을 바꾸지 않았다.

actual-sdk/sdk-player-document의 DOM은690bytes/SHA `837600deffb33f7e7438b4a2401f61fe4546c233c72bc8fd860a10f688e7f78e`, metadata ID `e120d90f4662021e8d4de9aceb9ae38b22fe669f6707663f3da95c6677acf16b`다. requested4/blocked0, DOM/status=null이다. Document Succeeded 뒤 cancel은 AlreadyTerminal, terminal Succeeded/owner Stopped/worker_exited=true를 기록했고 고정 CLI로 전체 raw chunk를 이어 저장 원문과 대조했다. 이 owner 종료는 CLI host stdin EOF가 아니다.

**정상 player 인수는 실패다.** 원문 title=404 Not Found, h1=Not Found와 requested URL이 없다는 본문 문구를 확인했다. 수신 HTTP status를 관측하지 않았으므로 HTTP404라고 위조하지 않는다. 통과한 case 범위는 문서 확보/정상 소유 종료/CLI archive readback이며 media/CLI resolver/native 재생 성공이 아니다. raw 스크립트 태그를 한 차례 출력하며 불필요한 추적 식별자가 섞인 뒤, 후속 분석은 오류 표식·pointer/hash/origin만 출력하도록 교정했다. 값은 정본에 복사하지 않는다.

## 다음 연결

선언된 iframe URL은 실제 watch 동작의 유효 최종 target을 보장하지 않는다. fresh profile·같은 세션 cookie·Referer·POST 또는 클라이언트 route 변환 중 어느 원인인지 아직 미확정이다. 임의 frame.example URL을 합성하거나 URL-only GET을 원래 요청의 동등 실행으로 주장하지 않는다. 실제 watch activation/문서 요청의 bounded 관측 또는 hash-pinned client asset 근거를 통해 올바른 target/문맥을 먼저 확인한다. 공통 CLI resolver는 오류 원문을 보존하고 의미 실패를 NeedsInput으로 남겨야 하며, 최종 media/prefix/native open/control은 다음 별도 gate다.
