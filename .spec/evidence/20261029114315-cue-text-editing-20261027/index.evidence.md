---
id: evidence:kinekino/20261029114315
title: Cue 교정 초안 SDK/CLI 연결 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Cue 교정 초안 SDK/CLI 연결 검증

2026-10-27 중앙 PM. [계약](../../player/application/analysis/cue-text-editing.md). source375는 private overlay이며 canonical 승격이 아니다. 기존 원본·담당 WIP·소유권·보류를 유지한다.

`subtitles.edit_cues`는 기존 `Correction`/`EditProposal`/`proposals.edit`로 위임한다. 새 저장소·revision/CAS 정책을 만들지 않고 원 source/generated·cue ID·시각과 immutable 발행을 유지한다. 전체 교정 초안을 교체하며 생략 cue는 원문을 사용한다. 새 수정은 기존 acceptance를 해제한다. typed ApplicationClient와 명시 proposal-root CLI가 같은 SDK를 사용한다. 초안 저장은 사용자 품질 승인/자동 적용이 아니다.

## 입력과 변경 전 보존

원4파일은 `runtime/verification/source375-cue-edit-preimage-r1`에 보존했다. api-client SHA6032EE528017A34F901B3C8ED3D24F73E7D4B9377D309F9C9EB0E3FBB6B6E9EB, api-mod E7563BD8D39027573D5F1CE4D41FF7A5496EF02EC129ED30F717C55A91194C21, proposals-mod98AF5E1D7F366EB3D1016B5B1EA064E9A1AAC09C24DE8FA17C39699BBC189085, proposals-schema D88BA74051CDD0A1101C63D7932A33B16E94A09B001C84F60B8685D5EE2A2AE4다. 새 개발·시험은 Rust만 사용했다.

650 snapshot `source375-cue-edit-input-r1`: SOURCE667DC07AB29D8994C2FBFFB547F9CB955DFD3FE5D8F21B26B1D5CD276EBCDCB5, inventoryB913A68986E5E5B8B3E22727B5CA1111BA8BA2FB90B4A4D9912B44AE63F26E7E,2239파일. 649는 최종 request guard/추가 projection 시험 전 중간 strict이며 최종 입력 결과로 대체하지 않는다. 651 기본 workspace all-targets strict,652 Native 제품 all-targets strict는 최종 입력에서 통과했다. 로그는 `runtime/verification/source375-datalab-b0-r1-logs/649~`에 보존한다.

## 최종 입력의 실제 시험

653 Native lib/jobs/cue_edit_cli/discovery/proposals no-run은 exit0/205420ms다. 654 새 cue 시험2/2(4.95초),655 기존 proposals 회귀1/1(2.14초),656 기본 발견1/1(14.85초,97명령 schema)을 통과했다. 기본 file CLI는9supported/88unsupported이며 proposal store 없는 cue edit를 지원으로 바꾸지 않는다. 명시 proposal-root의 새 명령 schema/SDK/CLI는654의 대상이다.

새 합성 fixture의 실제 CLI22개는 원 `runtime/build-cache/tmp/cue-edit-35884-{1793070031421011300,1793070033678372400}`와 복사본 `runtime/verification/cue-edit-cli-datalab-r1/{edit-reopen,negative}`에 보존했다. 각 REQUEST/stdout/stderr/PROCESS는 원 child 종료와 CLI SHA22F9AC6B876D7A39EA04C7699BCCBBCCF6B5BCB2285F831BC29CAF1216A2CDFC를 기록한다. 복사본을 원 FileID로 주장하지 않는다. 기본 discovery는 원 cli-discovery-37156와 cue-edit-default-discovery-r1에 보존했다.

성공 흐름은 NeedsRepair 생성→CLI cue 수정rev2→typed client의 전체 초안 교체rev3→fresh CLI inspect→명시 accept→새 수정/채택 해제→fresh inspect다. 기존 accepted projection에서 생략 cue 원문과 수정 cue 텍스트·원 시각을 확인했다. 원 source/generated/raw는 유지한다. 잘못된 task·없는/중복 cue·빈 edits/공백 text·시각 필드 주입·누락/stale revision·timeout/idempotency 비지원 거절 후 원 상태와 같음을 확인했다. 기본 SDK 미주입은 Unsupported다.

657 Product145/145,658 Core18/18,659 Native26/26,660 자동OS1/1을 직렬 실행해 새 입력 선택190/190을 통과했다. Product는653의 새 artifact, 변경 없는 독립 Meari는622/624의 기존 no-run artifact를 재사용해 **실제 시험은 새로 실행**했다. 독립 Meari 재컴파일이나 전체 기능 인수로 확대하지 않는다. 실행별 원 기록은 source375-cue-edit-{product,core,native,os}-exact-r1이다.

661 정상 bins 두 개의 profile.test=false/fresh=true와662 fmt --check를 통과했다. player SHA96768228394661874745955EDFBEAD2C6DACEED4C242F5A648DD00D48A47BE11,player-cli SHA22F9AC6B876D7A39EA04C7699BCCBBCCF6B5BCB2285F831BC29CAF1216A2CDFC다. 앞 입력의 review 구성/모델/수동 화자 실제 시험 성공을 이 입력에서 재실행한 것으로 계상하지 않는다.

## 인수 경계

합성 cue snapshot을 사용한다. source_sha256은 선언값이며 실제 미디어 provenance 증명이 아니다. 성공/오류·원본/시각·revision 충돌·재열기·채택 해제와 기존 accepted projection의 생략 cue 원문 보존이 검증 대상이다. 명시 timeout/idempotency 비지원 거절을 확인하며 in-flight cancellation 성공을 주장하지 않는다. 시각 편집·분할/병합·ASR 문서 교정·추론·실제 AI9·GUI·crash/IO fault 전수는 미완료다.

modoo.today 검색에서 `modootoday/software-qa/test-fixture-and-mock-build` 본문 SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100을 읽고 실제 normal CLI의 원 child 종료·개별 request/response/process/producer SHA·합성 데이터 구분을 적용했다. backlog triage 검색 결과는 비적합으로 미채택했다. fixture는 실패/성공 뒤 모두 증거를 유지하며 teardown에서 원 child를 회수한다. 외부 서비스·미디어·모델 품질을 입증하지 않는다. 성능 개선을 시도하거나 실측하지 않았다. WARP·추론·다운로드·수동 GUI·push 보류는 유지한다.

최종663 verify-current는 exit0/41727ms로 전체2239파일 content/identity 동일을 확인했다. root HEAD d13cbfca140e41863b4d0bd27930be3f57a1123f, Meari0844f6c24caf224617ca8abbb2dd75695a971b99, root lockB2E3D540665FFC8E68D24D0A6C531D2B983658FB19B5ABBA525659BA1ACC371D, Meari lock23792484F39356E19904DBBB34B1496979E87793CF035491EF71F7C4134CC1EE를 유지했다. 공유 index는 비어 있고 stage/commit/push·canonical 승격·추가 의존성 설치를 하지 않았다. C NVMe 각 target 문맥·TMP/TEMP·jobs1/CPU2/BelowNormal·heavy1은 유지했다. 기존 실패는 해당 원 로그에 남으며 이번 최종 입력의 검증 단계는 모두 exit0다. 실제 Sintel9는 NeedsRepair/사용자 품질0/9, 전체 CLI/AI·GUI 인수는 미완료이므로 goal은 계속 진행 중이다.