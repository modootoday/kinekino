---
id: evidence:kinekino/20261029114148
title: 수동 화자 annotation SDK/CLI 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 수동 화자 annotation SDK/CLI 검증

2026-10-27 중앙 PM. [계약](../../player/application/analysis/manual-speaker-annotations.md). 공통 speaker document/domain event replay·SDK application·ordinary file lease·명시 새 파일 publication을 정상 CLI의 subtitles.speakers/edit_speakers에 연결했다. AI diarization/align/word timing·화자 신원·미디어 바인딩이나 GUI/품질 승인으로 확대하지 않는다. 원본/source/history를 유지하고 사용자 편집만 새 revision에 기록한다.

## 입력 epoch 보존

첫 SOURCE `67C8A8E89EC69F9E1F3A19736C689516444B5D7D7176B9D53254F39B36D7FFAE`/2237,inventoryA6A3015F3A3C5D00A07FF65C83F9DB801F384B1DEAEAC6A2E41BA5FE83880B39를 source375-speakers-input-r1에 보존했다.604변경 strict/605all-targets strict/606snapshot/607no-run/608영향4시험3.32초/609기본97schema 발견1시험14.90초가 통과했다. 원 합성 fixture는 runtime/build-cache/tmp/speakers-cli-34468-{1793066478471395700,1793066478482461000,1793066479922790200}에 유지하고 runtime/verification/speakers-cli-datalab-r1/{cancel,negative,edit-reopen}에 복사했다.14개 원 CLI PROCESS·request/stdout/stderr,원 annotation SHA7E16B2E6DEA6CA3D4D114559334DCD32E0BE92F26F1F40D6E89D2179C8EA3548,CLI SHA C14F17A56A7366E167F2C5857942A06BFD1E30496F57324FA8E1FDC6944B7976를 보존했다. 복사본을 원 FileID로 가장하지 않는다.

동일 길이 변경에도 원 handle content hash를 재검사하고 hash 검사 뒤 최종 cancellation을 보도록 강화했다. r2 SOURCE `9EF84687BDEA336E1BE3F7880547E9ACC8F7C0D3E84F0F781603F56E979B79C6`/2237,inventory59616A4C604D08BAA2268612F71EADCD1AB7F0A67C962B09CB0735A30BAF4C1D의610strict/611snapshot/612no-run을 보존했다. 이 입력의 실제 후속 시험/190/bins는 실행하지 않았다.

정본의 span review_state 지원을 추가했다. 기존 strict DTO는 그 필드가 있는 입력을 거절했으므로,원 상태를 조용히 삭제한 것으로 주장하지 않는다. 현재는 optional 선언 상태를 보존하고 미제공은 undeclared로 두며,이름/현재 span 배정 변경을 needs_review로 돌린다. 원 reviewed 선언은 품질 승인이 아니다.613all-targets strict 후 새 r3를 고정했다.

최종 SOURCE `3BBB37E87F2FD20160BFD4CF1283A83FD2E1A4CA636F74AF69FFC2B5905B6936`,inventory `D4DF55050B3EB15B39AC511E699A25BCD7CF5C782F2105F7775700B64337D21B`,2237파일은 source375-speakers-input-r3다. 앞 입력 성공을 이 입력의 실행으로 승계하지 않는다. 원375 private overlay·기존WIP/소유권·원 실패/증거를 보존하며 canonical 승격이 아니다. 편집 전5파일과 중간 delta preimage는 source375-speakers-r1-preimage에 보존했다.

## 인수 범위와 남은 것

합성 JSON의 겹친 span/복수 화자/unknown·원 시각·원 source·편집 log/새 부모 SHA·undo/redo·과거 ID 재사용 거절·분기 후 redo 거절·기존 출력 충돌/무생성·stale SHA/revision·원 source guard와 publish 직전 cancel이 대상이다. 모델/오디오/미디어 decode·실제 source PTS·단어 정렬·DER/JER·SRT/VTT voice export·actual9·GUI/사용자 품질 인수는 대상이 아니다. File guard 해시 재검사 fault fixture는 의도적으로 permissive handle을 사용하며 production read-share guard가 정상 writer를 허용했다고 주장하지 않는다.

SDK/API의 file branch revision은 global latest CAS가 아니며 parent chain 전체 증거도 아니다. 동일 output 재시도는 충돌,재열기로 명시 조회한다. crash/power loss·OS IO fault 전수·hard preemption은 미검증이다. 원본 span 시각과 unknown/overlap을 편의상 단일 화자로 축약하거나 새 confidence를 만들지 않는다.

modoo.today 검색의 agent graph/PR triage는 도메인 비적합으로 채택하지 않았다. 기존 QA test-fixture-and-mock-build와 code-change-review-verification 본문 SHA9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B·verdict-table SHA9D9619A829AE24D73645EF5F7C8126C8257CAEDDAC80800632866257008AD2A9를 읽고 stale '전체 미구현' 문장과 실제 partial caller 경로를 대조했다. 새 정책·시험·도구 코드는 Rust이며 스킬의 다른 언어 예제를 도입하지 않았다. Codex runtime 인증을 주장하지 않는다. 다운로드/추론/수동GUI/WARP/push 보류는 유지한다.

## R3 실제 검증 종료

로그 정본은 runtime/verification/source375-datalab-b0-r1-logs/613~632다. 613 all-targets strict, 614 snapshot, 615 no-run 뒤 616 원 handle 동일 길이 hash fault 1/1, 617 수동 화자 SDK/CLI 4/4, 618 기본 discovery 1/1(97 schema,9 supported/88 unsupported), 619 모델 회귀4/4(실제 CLI115,명시 model-root12/85)를 통과했다. R3 수동 JSON의 실제 CLI14개 request/process/응답은 runtime/verification/speakers-cli-datalab-r3/{cancel,negative,edit-reopen}, discovery는 speakers-default-discovery-r3, 모델 회귀는 speakers-model-regression-r3에 보존했다. 원 tmp 증거도 유지하며 복사본을 원 FileID로 계상하지 않는다.

620 Product145/145, 623 Meari Core18/18, 627 Native26/26, 628 자동 OS1/1로 같은 입력의 최종 선택190/190을 확인했다. 625 Native의 terminal 확인 전에 626 OS를 배정한 순서 오류가 있었다. 실제 동시 실행을 단정하지 않으며 원 로그/첫 실행을 보존했다. 두 원 실행 종료 확인 후 627→628을 직렬 재실행했고 최종 근거에는 source375-speakers-native-exact-r3-serial-r2와 source375-speakers-os-exact-r3-serial-r2를 사용한다. producer 독점·무거운 작업1 배정 규칙을 유지한다.

629 정상 player/player-cli 빌드,630 기존 저장 CLI3/3,631 fmt --check,632 전체2237 입력 content/identity 동일(exit0,40223ms)까지 통과했다. 621 정상 CLI의 이전 R1 JSON 읽기도 exit0이며 review_state 미선언은 미선언으로 남았다. 실제 모델/미디어 승인이나 이전 입력의 성공 승계가 아니다.

정상 player SHA256: 54A945DFE92E58CCB508E552EBF36D1C0700774871E31FDA8D271E1820C25B18. 정상 player-cli SHA256: CB65AD8BF81DF35AC4B3DE1D11D1D7C091F3466468DCEE6919D7A002DA01BD0D. root HEAD d13cbfca140e41863b4d0bd27930be3f57a1123f, stage 비어 있음. 새 source는 private overlay이며 canonical 적용/commit/push를 수행하지 않았다.

실제9 표본은 Sintel이며 모두 NeedsRepair, 사용자 AI 품질 승인0/9다. 이번 합성 annotation 검증이나 자동 OS1을 실제 품질 승인·수동 GUI 시각 인수로 계상하지 않는다. GUI 원본/후보 비교·재생·수정·판정/이력 연결과 실행 시안 비교를 포함한 구현·검증 goal은 계속 진행 중이다. 배포는 대상이 아니다.