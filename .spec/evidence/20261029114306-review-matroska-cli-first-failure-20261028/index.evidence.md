---
id: evidence:kinekino/20261029114306
title: R78 실제 MKV 정상 CLI 첫 실패
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R78 실제 MKV 정상 CLI 첫 실패

2026-10-28 중앙 PM. R77의 실제 producer/SDK association 성공 다음에 정상 CLI host 경유 검증을 추가했다. 제품 구현은 변경하지 않고 기존 기술 표본 시험과 별도 opt-in 실제 표본 시험 두 개를 등록했다. 실제 선택 패키지 B1E2BE10, 원 미디어 F12C070E, gemma270-highlights, 기술 구간107.25~133.8초를 명시한다. 역사 모델 시간축·사용자 품질 권한은 부여하지 않는다.

변경은 mirror의 tests/review_startup_actual.rs 및 하위 controls.rs, 새 matroska.rs다. 기존2파일 preimage는 mirror/runtime/verification/preimage-r77-mkv-cli에 보존했다. 기존 기술 표본의17초 시작은 유지하고 실제 표본에107.25초 시작·134초 외부 seek를 전달한다. 원본9개·WIP·소유권·보류·private overlay를 유지한다.

R78 SOURCE SHA256 `2613B68F77F9F3C5CFB38469A025288375A700225BEFFD40132974063E3EC8E6`, inventory `F51E5B54E290F67D8995E6890205994C99396763262CBE549562D3EA2B02DA9E`, 2323파일. 스냅샷은 mirror/runtime/verification/input-r78이다. 시험 고정본 fixed-r78/review_startup_actual-b31556eec3471e2a.exe SHA256 `ABFE7ADDF5012F346B23BF04C743E840849DEBEA7471E84BFC0261BA17B3C431`. 정상 CLI는 원R77 D4E7BE8B 실행파일이며 guard와 JOB_CLI_SHA256로 확인했다.

로그는 runtime/verification/source375-datalab-b0-r1-logs 아래다.

|로그|관측|
|---|---|
|1684|fmt 적용 exit0|
|1685|PM이 없는 native-ui feature를 지정해 실행 전 실패. 원stderr 보존|
|1686|올바른 native-proposal-window startup 시험 no-run exit0|
|1687/1690|Native/default all-targets strict 각각 exit0|
|1688|현재 입력 snapshot exit0|
|1689|실제 MKV CLI controls 시험 FAILED, verify 응답 대기 timeout|
|1691|fmt --check exit0|
|1692|최종 custody exit0·2323입력 content/identity 동일|

실제 시험 raw는 mirror/runtime/verification/actual9-mkv-cli-controls-r78-r1. host READY는 새 package AF03E7A8 및 evidence20F3404D를 보고했고 association.create 응답은 성공했다. association.revise 전송 뒤 client_deadline/read_response에서5019ms·CLI exit10, delivery=execution_unknown으로 실패했다. 동일요청 replay·mpv open/제어·EOF·restart는 미실행이다. 저장 mutation 성공/실패를 추측하지 않는다.

정상 CLI help와 EndpointClient를 확인한 결과 --endpoint 전송은 고정5초 기한이며 request.context.timeout_ms와 별개다. 공통 시험 supervisor도15초다. 대용량 미디어/증거 검증을 포함하는 실제 mutation에 대한 명시 전송 기한 계약이 다음 구현 공백이다. 기본 기한을 몰래 늘리거나 해시 검증을 생략하지 않는다. bounded opt-in 전송 기한, 요청 기한과의 구분, 상한·잘못된 입력 거절·execution_unknown 보존 및 시험 supervisor의 일치가 필요하다.

실패시 기존 시험 Owner Drop은 host kill/wait 경로로 나갔으며 정상 EOF/host PROCESS 기록은 생성되지 않았다. 이후 player-cli/mpv/ffmpeg/ffprobe 프로세스 조회는 대상0개였지만 원 host의 정상 정리 receipt로 계상하지 않는다. 재실행은 새 출력 디렉터리를 사용한다. restart 시험은 아직 실행하지 않았다. 190/정상bins는 원R77 근거이며 R78 전체 gate를 주장하지 않는다.

적용 스킬: modootoday/software-qa/test-fixture-and-mock-build SHA256 A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및 code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B. 실제 표본/기술 표본과 전송 불확정/실행 성공을 구분했다. C-only·heavy1·CPU2·BelowNormal·offline/locked/-j1 유지. GUI·추론·다운로드·WARP·push 실행0, 사용자 승인0/9, goal active.
