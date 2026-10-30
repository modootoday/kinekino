---
id: evidence:kinekino/20261029114638
title: 실제 CLI의 같은 세션 player 문서 완료
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 CLI의 같은 세션 player 문서 완료

## 고정 소스와 검사

`runtime/quality-review/player-document-cli-r1`에 source-product/lock/final-bin/harness와 첫 로그를 보존했다. fmt0, strict workspace-alltargets Clippy17.00초0, 전체240pass/17ignored(root147pass/1ignored), test compile2분57초, bins0.79초0이다. lock SHA는 `0cf44760431b804cdf18afd50aa82f57b50579cfe1465f9f6d6e9ee7b1aa96cc`다.

고정 CLI SHA는 `8ab2f8b45681550f6890b1b329ca35119a2f8b0800d8e2c3764d475dc80db154`, GUI SHA는 `b583ba8823266b59132198dfc27fe325db6b2d2903acfc9c0c0dde05d0b8794c`다. GUI는 실행하지 않았다. Cargo test 로그의 실제 jobs-46b3a58152122637.exe를 고정하고 --list에서 exact case를 확인했다. harness SHA는 `d9520fe87eb1535d6553fc62cd170600cf621343edf1223f53e9c5a39e5cce2e`다.

## 실제 인수

`resolve_cli::actual_cli_resolve_preserves_completed_player_document`는 추가 compile 없이1pass/24.66초/exit0이다. 같은 불변 episodes seed의 첫 회차 한 개를 사용했다. explicit provider host에 실제 CLI jobs.submit을 수행하고 typed client의 동일 idempotency key가 기존 job을 반환하는 것을 확인했다. 마지막에 stdin EOF→browser_worker_exited=true를 확인한 뒤 고정 CLI catalog.read_raw로 초기 DOM·player DOM·event trace 전체 바이트를 보존 파일과 대조했다. 이전 실제 관측은 덮어쓰지 않았다.

초기 source.example.test /e 문서는136370bytes/SHA `1928650cddd20eb371af51b53eb6b88a123524e06cc3202592231b2131092982`, metadata ID `829f1d19c3bc95e6d550f405730bfa5860c2af6cfb14a556e8047b7f897afb73`다. generation1/native navigation ID2/HTTP200/IsSuccess=true/WebErrorStatus0, 요청33/차단2다. 고정 watch actuator의 Enqueued와 InvocationObserved 기록은 이 generation1/navID2로 남았다. 새 문서 상태에 이전 ID를 혼합하지 않았다.

같은 소유 browser에서 실제 이동한 frame.example.test 문서는28014bytes/SHA `04d80603ab09117849b7d65cc9fbef5fbb92acde652675410c5fe9bf9b993e65`, metadata ID `5476449d3a93a78133fec082a462ded509117c99187c06faf67a43d96123a46b`다. generation2/native navigation ID5/HTTP200/IsSuccess=true/WebErrorStatus0, engine155.0.4283.62, 요청50/차단3이다. 같은 세대/native ID의 completion과 같은 세대/최종 URL의 DOM을 함께 확보했다. 문서 title은 선택한 회차를 표시하며 iframe 태그가 존재한다. iframe 내용·상태는 이 top-level DOM으로 입증하지 않는다. 정확 URL·opaque query는 private metadata/DOM에만 보존한다.

event trace503bytes/SHA `9315df40c2d0f42136ff97c9a8eda2e95c80a944b512a3f94d3c158d0e75771c`, metadata ID `8e523e13b8bae69faed8da8ea2e42ed3d1052f105d55cae567dedd1c7ed1ecd4`다. 각 capture.status는null이며 별도 native navigation 진단과 구별한다. activation_history는 유지됐고 frame candidate0/media request0이다.

## 의미와 남은 작업

job은 NeedsInput/media_verification_required/player_document_observed=true, output 없음/publication_started=false/media_verified=false/native_playback=false다. 문서 완료·원문 보존·소유 종료·전체 CLI 재읽기 인수이며 영상 추출·영상 재생 성공이 아니다. airing 결과에서 선택한 첫 회차 한 개의 근거다. 다른 작품·세 카테고리의 회차·페이지네이션 전체 지원을 입증하지 않는다.

다음 단계는 보존 player DOM의 실제 iframe·공개 초기화 구조를 분석하고, 허용된 실제 요청/response/prefix의 소유 수명을 연결하는 것이다. 현재 MediaRequested는 요청 후보뿐이며 response 검증 결과를 게시하지 않는다. 기존 미등록 media_observer의 current URL-only 귀속을 허용하지 않는다. 최종 verified media 이후 공통 SDK/CLI의 native host/mpv lifecycle·control·회차전환을 검증한다. 모델 추론·다운로드 보류는 유지한다.
