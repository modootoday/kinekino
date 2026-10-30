---
id: evidence:kinekino/20261029114546
title: 실제 CLI의 소유 자식 프레임 문서
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 CLI의 소유 자식 프레임 문서

## 고정 검사와 자원 예산

`runtime/quality-review/frame-document-cli-r1`에 source-product/lock/final-bin/harness와 원로그를 보존했다. fmt0, 수리 후 strict workspace-alltargets Clippy31.25초0, 전체245pass/17ignored(root152pass/1ignored), test compile4분01초, test 실행기 전체355.177초/exit0, 두bin build0.84초0이다. 신규 프레임 순수 회귀4개와 반복 native completion COM 회귀1개를 포함한다. 새 AI provenance 모듈은 source archive에 존재할 수 있으나 root에서 미등록이므로 이 시험 수의 검증 대상이 아니다.

첫 Clippy 실패는 windows implement 매크로의 windows-core 직접 의존 누락이었다. 원로그·실행기 receipt와 수리 전 Cargo/lock/frame_completion을 보존했다. 기존 lock의 windows-core0.62.2를 직접 선언했으며 새 다운로드를 하지 않았다. 최종 root lock SHA는 `477b61ddf828c185697fb0b6d5fa35231ee14b3b38670e8690a73331faf4fa14`다.

CLI SHA `5eb7b507338b3f5398a9a369681f5baac7f91b4c2d6b58737ed122a06dbe60df`, GUI SHA `60789f6a88f58969ca9531a37f78df36b8e799d7c9ab01349b9ca4e9ae9b28a4`다. GUI는 실행하지 않았다. 이번 Cargo 로그의 jobs-ca0b21e43d85ab44.exe를 복사하고 --list에서 exact case 존재를 확인했다. harness SHA는 `65fe89b18abe06a053f067a4ef969a6d1e79c2238750e000f291d2e7b8491fd1`다.

[소유 실행기 검증](resource-runner-2026-10-19.md)의 최종 exe SHA는 `692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463`다. 실행기 자체 fmt/test1/strictClippy/build가 통과했다. product Cargo child31428은 실제affinity3/두CPU/priority16384를 기록했다. 고정 actual harness child99648도 같은 제한, terminal=true/exit0/20.203초 receipt를 남겼다. Job Object의 제한을 적용했으며 사용자 앱의 우선순위·affinity는 변경하지 않았다.

## 실제 프레임 관측

고정 `resolve_cli::actual_cli_resolve_preserves_native_child_frame_document`는 추가 compile 없이1pass/20.17초/exit0이다. 불변 episodes seed의 첫 회차 한 개를 사용했다. 실제 CLI 접수와 typed client의 동일 idempotency key 재사용, initial/parent/frame DOM+trace 네 원문 전체 CLI 재읽기, stdin EOF 뒤 browser_worker_exited=true를 확인했다.

직접 Core FrameCreated→Frame5 ID→Frame2 탐색·완료→고정 DOM 직렬화로 확보한 media.example.test 문서는15115bytes/SHA `8b3bd95da09ba94035170ef4687f53aeef4e5519a9cbfa6b2860b8ace3ccab36`, metadata ID `1d23111216c2948dd286675b70e5dee5f1602084fd58067680be3cee518bc085`다. 부모generation2/native navigation ID5, native frame ID2, 프레임 내부generation1/native navigation ID6이다. 자식 native 진단은 HTTP200/IsSuccess=true/WebErrorStatus0이다. 부모 진단과 ID를 혼합하지 않았다. capture_kind=frame_dom_snapshot/status=null이며 frame completion을 별도 provenance에 보존했다.

자식 title은 Anibeast이고 jwplayer 표식이 존재한다. 정상 HTTP와 문서 구조 확보의 근거이며 media decoder나 영상 재생 성공의 근거가 아니다. 정확 URL/opaque 값과 스크립트 원문은 private capture에만 남겼다. 과거 fresh top-level GET의 오류 문서를 수정하거나 그 실패 원인을 이번 성공으로 단정하지 않았다.

초기 source DOM136370bytes/SHA `9b356fba7d23a34b6e5893e897164435a7cb9183be0d2dcf52312697a316d4ea`, metadata ID `2e5a2ff92f58b5e94188100911dfbf2c3faebfab5bd228ebd7e0fe9d48bfdb44`다. Frame-host parent DOM28014bytes/SHA `04d80603ab09117849b7d65cc9fbef5fbb92acde652675410c5fe9bf9b993e65`, metadata ID `727c961c406cb9a548fdafc2b349fe636649b0a825d86f69716ed633fc19b617`다. trace913bytes/SHA `c59c2d960359fd5c543ae8e09d9e1efa0a1c8541a878752884e6ffaf3c8cf936`, metadata ID `951721b4c909405d1dfa3574dacd52749e38a8c52b222fcc69b15143e4081a1c`다.

## 완료되지 않은 범위

job은 NeedsInput/media_verification_required/player_document_observed=true/frame_document_observed=true, frame_document_count1/media_request_count0/output 없음/publication_started=false/media_verified=false/native_playback=false다. 이번 인수는 실제 직접 자식 문서 수명·원문 보존·소유 종료·CLI 재읽기다. 최종 media response/manifest prefix·transferable source·소유 native mpv 재생/제어/회차전환은 미완료다. 부모·자식 DOM과 실제 media 결과를 구별한다.

다음은 보존 자식 문서의 실제 player 초기화 구조에 근거한 요청/응답 관측이다. 미등록 media_observer를 URL-only 현재 문서 귀속으로 연결하지 않는다. 첫 회차 한 개의 인수를 전체 작품·세 카테고리의 회차·페이지네이션 지원으로 확장하지 않는다. 모델 추론/다운로드 보류는 유지한다.
