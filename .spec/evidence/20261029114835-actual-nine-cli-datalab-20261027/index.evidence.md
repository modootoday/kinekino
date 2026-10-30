---
id: evidence:kinekino/20261029114835
title: DATALAB 실제 텍스트 결과9의 제품 CLI 인수
status: active
created: 2026-10-29
updated: 2026-10-29
---
# DATALAB 실제 텍스트 결과9의 제품 CLI 인수

2026-10-27 중앙 PM. source375 private overlay SOURCE667DC07AB29D8994C2FBFFB547F9CB955DFD3FE5D8F21B26B1D5CD276EBCDCB5/2239에서 기존 `historical_provenance_cli::actual_nine_binding_fresh_cli_accept_retry_and_project_capture`를 명시 opt-in으로 실행했다. 이전 호스트의 성공을 승계한 것이 아니라 이 C-only 호스트의 새 실행이다. 원본·기존 WIP/소유권·첫 실패 이력과 보류를 유지했다.

## 고정 입력과 실행

원 패키지는 `.spec/artifacts/quality-review/sdk-text-results-r3`이며 package SHA659B3CB6757259D8E770F6D3C96C4A77DAFBAB43C6EA540BCC7E0C58E4EB0197, historical companion SHA7A7CD05EAFA6294E846C8080C976600EA135AA91E9DA315AC9F770E45F624B10/16566bytes다. Sintel의 기존 Gemma270/Llama1B/Qwen0.6 × correction/cards/highlights9개다. 모델·원문·candidate raw를 새로 생성하지 않았다.

verifier는 C:/workspaces/com.kinekino.player/runtime/quality-review/historical-import-2026-10-19-r1/player-review-sdk-export.exe, SHA E00C1726603EF7D0167C6D58DEB79E694071044C20A2723DE8ABE8A504F45E41다. 기존 고정 Rust 도구를 이 호스트에서 실행했으며 새로 빌드했다고 주장하지 않는다. 역사 D 경로/FileID는 원 pin에 유지하고 실제 C 경로를 명시했다. CLI는 정상22F9AC6B876D7A39EA04C7699BCCBBCCF6B5BCB2285F831BC29CAF1216A2CDFC이며 evidence용 사본과 실제 target의 SHA가 같다.

원 AGENTS·package objects·세 모델 결과 manifest는 시험이 시작 시 pin하고 끝에서 다시 해시 대조한다. 새 evidence는 `runtime/verification/actual9-historical-cli-datalab-r1`이다. 이 경로의 input-pins.json SHA E5DFCCED193122E79F4C74503A9AC1EDD41327949A0014439D3EB373BCDB7C8A를 보존했다. workspace/proposals/projects 사본에서만 기술적 수정·누락/변조·불완전 publication을 주입했으며 원 저장소/판정/실제9는 수정하지 않았다.

664 Native historical test no-run exit0(7.77초),665 정확 opt-in1시험 pass/0ignored/16.74초,runner exit0/17267ms다. 로그는 `runtime/verification/source375-datalab-b0-r1-logs/664~666`이며 73개 request별 receipt/stdout/stderr/PID/exit/reaped를 보존했다. 중간 오류 응답은 기대 거절의 근거이며 정상 기능 성공으로 계상하지 않는다.

## 단계별 실제 상태 확인

| 대상 상태 | 독립 조회/대조 | 결과 |
|---|---|---|
| 고정 verifier 없이는 historical import 불가 | without-verifier 응답·publication 부재 | Unsupported/exit3, 통과 |
| 실제9의 원 source/raw/binding 보존 | 9 import 응답과 각 fresh CLI inspect, 원 object bytes 대조 | 모두 schema3/revision1/NeedsRepair, 통과 |
| 교정의 기술적 수정·채택·재요청 | DocumentTextRef /0의 명시 fixture text→edit→stale 거절→accept/retry→inspect | 같은 receipt, source/generated/historical 불변, 자동 적용false |
| 카드·하이라이트 기술적 수정 | 각 Gemma270 항목의 원 cue ref→foreign/stale 거절→edit/accept/retry→project 재열기 | 두 task 통과; 텍스트 fixture이며 실제 의미 수선 아님 |
| 프로젝트 캡처/저장/재열기 | live capture→saved capture→fresh projects.inspect·stale save 거절 | binding/원 상태 보존, 통과 |
| 증거 누락/동일 크기 변조 거절 | 복사 companion/source/candidate를 각 변조 후 inspect/accept와 publication inventory | 신규 publication 없음, 원 오류 거절 |
| binding 제거/다른 item 교환 거절 | 단일/다중 revision 변조 뒤 fresh CLI | NeedsInput 또는 InvalidArgument, 통과 |
| 불완전 publication 거절·복구 | 사본 r-4 빈 디렉터리→조회 거절→해당 사본 원상 복원→fresh inspect | publication/project inventory 일치, 통과 |
| 원 입력 불변 | Rust harness assert_original_inputs_unchanged | 통과 |

## 범위와 추적

템플릿 이름만으로 실제 전송 내용을 알 수 없다. 이번에는 추론 prompt를 전송하지 않았으며 새 모델/설정/품질 점수를 만들지 않았다. 기존 원 request/candidate bytes의 SHA와 source pin, CLI 요청 JSON·응답·원 process receipt를 사용한다. 실제 시간/PID/임시 경로는 재현 가능한 내용 해시와 구분한다. 동일 source에서 새 독립 scratch를 만들었고 원 파일의 외부 부작용을 반복하지 않았다.

modoo.today 검색의 synthetic-training audit는 비적합으로 미채택했다. `modootoday-free/ai-quality-plus/agent-run-postcondition-and-replay-check` SKILL.md SHA61843099B7D54C3D7AC226218A5EE1B6CCDBE36EF84268762DBC33A68FBB58B5와 worked-example SHA80C9FE8A8A79DD8E08D84B36483D7742F273E55FB2769D2E6156A1111FDD5C11을 읽고 단계별 실제 재조회·원 hash·격리 재현·기대 실패/성공 구분을 적용했다. 스킬 자체가 실행했다거나 Codex 인증을 받았다고 주장하지 않는다.

이 인수는 원 결과9의 저장·수정·기술적 채택 경로다. 사용자 AI 품질 승인0/9, 원 후보 모두 NeedsRepair를 유지한다. fixture text를 모델 결과의 실제 수선이나 품질 승인으로 계상하지 않는다. 실제9 range=null/검증된 영상 PTS 결합·GUI 비교/재생/수정/판정/이력·사용자 시각 인수는 남는다. 새 추론/다운로드·WARP·수동GUI·push는0이다. 기존 같은 source의 strict/fmt/선택190/정상bins는 [667DC07A 검증](cue-text-editing-20261027.md)에 두며 이번 시험에서 다시 실행했다고 주장하지 않는다.

최종666 source verify-current는 exit0/40763ms로 전체2239 content/identity 동일을 확인했다. Rust/테스트/도구 소스를 변경하지 않았다. stage/commit/push0이며 goal은 전체 SDK/CLI·AI 검토/GUI의 미완료를 포함해 진행 중이다.
