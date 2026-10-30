---
id: evidence:kinekino/20261029113739
title: 제안 편집 UI 실제 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제안 편집 UI 실제 검토

## r162 추가 일반 미지원 실행

같은 고정 SHA052849 실행본을 --proposal-root 없이 --record-validation --validation-position=3500,100으로 새로 실행했다(PID64216). 1440 제작 CARD에서 제안 영역 기본 접힘을 확인하고 펼쳤다. 저장소 연결 필요 문구와 비활성 ID/제안 열기 표시를 확인했다. 다른 두 작업과760 미지원 입력 거부 조작은 이 실행에서 확인하지 않았다. 펼침 직후 request_discard4frames 경고가 한 번 보였고 다음 별도 캡처에서 사라졌다. 기존 resize5frames와 함께 구조 안정화 후속 근거로 유지한다. 캡션 닫기 후 fresh list_windows에서 해당 경로 창0개를 확인했다. 이 고정 r162 관측을 변경 중인 S2 source 검증으로 소급하지 않는다.

하이라이트 단독 재시작도 같은 고정 r162를 별도 PID20476으로 실행해 확인했다. 같은 소유 store를 연결하고1440 제작→CLIP AI→ui-highlights 조회 시 저장 제목 `외부 CLI가 저장한 새 버전 · 최종 사용자 검토 / 手動確認`, 초안4·출력4 receipt와 비활성 채택 버튼을 확인했다. AI 탭 첫 클릭 직후 화면이 유지돼 새 캡처 근거로 다시 클릭한 뒤 실제 전환을 확인했다. 캡션 닫기 후 fresh 경로 창0개다. 아래의 highlight 단독 GUI 재시작 미검증 기록은 이 추가 관측으로 보완한다. 다른 미검증/경고는 유지한다.

## r162 최종 receipt 복원·760 하이라이트·실제 충돌

SDK r2 fmt/test150passed9ignored/clippy all-targets/build 양bin 통과. 별도 고정 검토본 `runtime/review-runs/ui-proposals-20261019-r162/player.exe`, SHA256 `05284974561addb3fbd973e9da8b284c2fe45607e2d09af08a288d62a31b670e`, PID48692. CLI SHA `e9adca1f41268fd188cc99593590152a3aba03cfe146509b47d5728775b504a4`. 일반 실행과 같은 r161 소유 격리 store를 사용했다. 아래는 이 SHA에서 실제 확인한 범위다.

- 1440 재시작 후 CARD `ui-cards`와 SUB 교정 `ui-correction-repair`를 각각 조회했다. 수정문·원문과 출력버전3 receipt가 복원되고 같은 저장본의 채택 버튼이 비활성화됐다. 교정의 원생성 parse 실패는 현재 저장 상태와 별개라는 보조 label을 확인했다. 교정 조회와 CLIP 탭 전환에서 각각 사용자 입력 감지 후 fresh capture로 재관측하고 입력을 재시도했다. 사용자 Stop은 없었다.
- CLIP AI를760×760으로 전환했을 때 `request_discard has been called 5 frames in a row` 경고가 한 번 보였다. 다음 별도 capture에서 사라졌다. 이 변경의 레이아웃 안정화를 완료했다고 주장하지 않는다.
- `ui-highlights` 조회 초안1에서 제목을 `예시 하이라이트 · 충돌에도 남을 편집 / 入力保持`로 수정했다. 별도 최종 CLI로 expected_revision1 편집을 저장해 서버 초안2를 만들었다. GUI 저장은 RevisionConflict로 거부됐고 위 입력이 그대로 남았으며 채택은 dirty 상태로 비활성화됐다. 자동 재시도/덮어쓰기는 없었다.
- 검증자가 명시적으로 수정 버리기를 누른 뒤 재조회해 CLI 저장 초안2를 확인했다. 제목을 `외부 CLI가 저장한 새 버전 · 최종 사용자 검토 / 手動確認`로 수정했다. Tab으로 c1 원문근거 checkbox focus 이동, 설정 drawer 열기→Escape 후 수정문 보존을 확인했다. 본문 스크롤로 저장 버튼에 접근해 초안3을 저장하고 별도 채택 클릭으로 출력4 receipt·렌더링/파일 내보내기와 구분 안내를 확인했다. 저장하지 않은 수정 상태에서는 조회/채택이 잠겼다.
- 별도 최종 CLI의 세 작업 inspect 결과를 r162 `readback-correction.json`, `readback-cards.json`, `readback-highlights.json`에 보존했다. 원문/원생성 문자열은 최초 fixture와 같은 내용이며 null 시각을 유지했다. correction/card receipt는 output3, highlight는 draft3/output4이고 automatic_application_enabled=false다. 이는 합성 자료의 실제 Rust 저장/GUI/CLI 교차 조회이며 모델 품질/사용자 실자료 인수/미디어 렌더 증거가 아니다.
- 캡션 닫기 뒤 fresh list_windows에서 r162 경로 창0개를 확인했다. UI cargo0이며 SDK에 타입 freeze 해제 안전 체크포인트를 통보했다. SDK의 이후 source V2 변경과 이 실행 근거를 섞지 않는다.

잔여: highlight 단독 GUI 재시작, backend 없는 실제 GUI, 세 작업 모두의760/1440 긴 문구·전체 키보드, 원생성 펼침/복사 실제 인수, 일반 오류·모든 비동기 경쟁은 미검증이다. conflict 입력을 보존한 채 최신본과 나란히 재검토하는 UX는 아직 없고 명시 discard/reload 경로만 있다. 원본 시안의 전용 inspector와 실제 제안 편집 영역 통합·밀도/여백, 공용 dropdown glyph 등 세부 시각 일치는 계속 필요하다. resize5frame 경고와 전체 화면군의 기존 잔여도 유지한다. 다음 source V2 문서 참조 연결은 별도 SDK DTO checkpoint로 진행한다.

## r161 중간 실행 — 최종 빌드와 구분

2026-10-19. UI는 source-owned `proposal_editor`/`input::multiline`과 `studio/proposals` state/worker/view로 구성한다. SUB 교정·CARD·CLIP AI의 본문 스크롤 안에 typed SDK inspect/edit/accept를 연결했다. 정책/파일 저장/application/API/main은 UI가 수정하지 않았다.

중간 검토본 `runtime/review-runs/ui-proposals-20261019-r161/player.exe`, SHA256 `d14669b37bffa7493e473781fc8b55c069ebb15e0b753addb7a54ebcc16e8828`, PID49692. 일반 실행 --record-validation --validation-position=3500,100 --proposal-root=<동일 검토 디렉터리/store>. 기존 가상 모니터에서 실제 입력했다. ui-demo/모델 추론/실제 사용자 미디어를 사용하지 않았다. 합성 fixture JSON은 검토 디렉터리에 보존했다. 기존 연구/SDK 저장소를 수정하지 않았다.

- 일반 시청 시작→제작→SUB 교정에서 공용 제안 영역을 확인했다. `ui-correction-repair` 조회는 초안1/수동수정필요/import_error와 원생성결과 읽기 전용 펼침을 표시했다. 저장/채택은 비활성이었다.
- 교정문 추가→c1 원문 선택에서 한국어·일본어 원문이 표시됐다. 수정문에 `사용자가 고친 문장입니다. 原文を保ち、修正文だけ保存します。`를 입력했다. dirty 동안 ID조회/채택이 잠겼고 저장은 가능했다. 저장 후 초안2와 수정문 보존·채택 활성화를 확인했다. 별도 클릭으로 채택 후 초안3/출력버전3 receipt와 ‘미디어 렌더링/파일 내보내기 완료가 아님’ 안내를 실제 스크롤로 확인했다. 원생성 parse 오류 문구는 저장 뒤에도 남아 원생성 오류인지 설명 보완이 필요하다.
- CARD에서 별도 `ui-cards` 조회로 앞·뒷면과 c1 원문근거 체크/원문 표시를 확인했다. 앞면에 ` · 사용자 수정 / 手動編集`를 추가했다. dirty→저장 초안2→별도 채택 초안3/출력버전3과 렌더 구분 안내를 확인했다. 교정 편집이 카드에 섞이지 않았다.
- CLIP AI 진입과760×760 검토 preset 전환만 확인했다. 이 중간 실행에서는 하이라이트 조회/수정/저장/채택을 아직 하지 않았다.
- 캡션 닫기 후 별도 fresh list_windows에서 해당 경로 창0개를 확인했다. VM/DPI/모니터 설정과 사용자 앱을 변경/종료하지 않았다.

## 검사와 남은 결함

SDK 소유 공유 native target에서 중간 전체148pass/9ignored와 UI 신규2회귀 통과를 보고받았다. UI view의 nested if clippy1건을 let-chain으로 최소 수정했다. SDK 최종 fmt/test149pass9ignored/clippy all-targets/build 양bin 통보를 받았고 고정 final player SHA는 `7bf52ac96f3c7e9d46ba1cb2c13edc36b343e0a96a63b8d977640f0e11afc5cc`다. 위 실제 관측을 이 final SHA의 검증으로 소급하지 않는다.

재시작 인수를 준비하면서 typed inspect가 ProposalDocument만 반환하고 StoredProposal.acceptance를 노출하지 않는 계약 누락을 발견했다. 새 GUI가 기존 receipt를 복원할 수 없고 UI가 revision에서 이미 채택됨을 추정해서는 안 된다. SDK에 명시적 typed receipt 조회 계약 확장을 요청했으며 이 항목은 미완료다. 최종 실행·3기능 전체·충돌 입력 보존 실제 동작·재시작/unsupported·760 편집/패널/키보드·원본 bytes 대조는 계속 검증해야 한다. 전체 시안 일치 완료가 아니다.
