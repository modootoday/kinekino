---
id: evidence:kinekino/20261029113827
title: S2 문서 원문 실제 UI 검토 — r164
status: active
created: 2026-10-29
updated: 2026-10-29
---
# S2 문서 원문 실제 UI 검토 — r164

## 메인 재실행 복원 후속 — r167

아래 초기 미검증 목록 중 Gemma 문서 교정의 r164 재실행만 후속 확인했다. 기존 window54072618을 caption 닫기 후 fresh list에서 같은 경로[]임을 확인했다. 동일 고정 EXE를 메인 위치100,100/동일 복사store로 재실행했다. 새 PID103288, 제품 GUI window64361366이다. Windows 목록에는 같은 프로세스 경로의 콘솔 제목 window113580940도 나타났으며 이를 두 번째 제품 GUI로 집계하거나 다른 앱으로 조작하지 않았다. 후속 배포 실행에서는 debug console 노출을 별도 검토해야 한다.

실제 시작→제작1440×900→SUB 교정→actual-gemma270-correction 재조회에서 source revision1/draft3, 문서 SHA, outer/extraction 위치, /5 선택·원문 `So, what brings you to the land or the gatekeepers?`, 수정문 `So what brings you to the land of the gatekeepers?`가 복원됐다. 본문 스크롤 후 `채택됨 · 출력 버전 3`과 저장/재채택 비활성화를 확인했다. 새 저장/채택 클릭은 하지 않았다. 카드·하이라이트 재실행이나 9건 전체 검증으로 확대하지 않는다.

시각·상태 불일치: 연결된 본문 실제 제안 편집기 옆의 기존 우측 교정 inspector는 원문/교정안 `연결 필요`, `교정 제안 연결 필요`, 근거 `—`를 그대로 표시한다. actual source와 draft가 존재해도 이 영역은 별도 미연결 read model을 사용한다. 연결 성공 상태를 전체 화면에 일관되게 투영하도록 기존 inspector와 비교 편집 패널을 통합해야 한다. 단순히 경고 문구를 숨기거나 실제 미디어/PTS 연결을 생성하지 않는다. r165의 canvas 밀림과 함께 수정해야 하는 명확한 미완료 항목이다.

## 고정 실행본과 자료

`runtime/review-runs/ui-document-source-20261019-r164/player.exe` SHA256 `8ad628c969c7aa9f580740d734e10655f97a55834161bac96b81fa4e05fd1779`, CLI `bc6483acf78e4a973b5716387e6c0ae59659ea8d365d7986f72941dec5594b00`. SDK 최종 gate는 fmt/test 155 passed·9 ignored/clippy/build 성공이다. r162 합성 cue 검토와 구별한다.

actual-nine-r1/input-store를 r164/store에 복사하여 사용했다. 원본 input-store·technical-store를 수정하지 않았다. 새 추론·모델 품질 승인은 없다.

## 실제 문서 교정과 readback

PID110916/window54072618에서 SUB 교정의 actual-gemma270-correction을 조회했다. 원생성 `thir` 반복과 parse 오류를 읽기 전용 펼침에서 확인했다. 문서 SHA·추출 위치가 표시되며 문장 선택 공용 dropdown에 /0…/5가 나타난다. /5 선택 시 원문은 `So, what brings you to the land or the gatekeepers?`이다.

별도 수정문 `So what brings you to the land of the gatekeepers?`을 입력했다. 이는 쉼표 삭제와 or→of 변경을 포함하는 기술 검증용 수동 편집이다. 문장부호만 교정했다거나 품질이 승인되었다고 판정하지 않는다. 1440×900→760×760 전환 후 원문·수정문·SHA가 유지되고 스크롤로 저장에 접근했다. 수정본 저장 후 별도 채택 클릭으로 `채택됨 · 출력 버전 3` 및 미디어 렌더링/파일 내보내기와 다르다는 안내를 확인했다.

고정 CLI `--input inspect-correction.json --proposal-root ABS` readback은 ok=true. draft_revision2→output_revision3, automatic_application_enabled=false, source ref asset SHA `b282db497133af492690d5d8726549cbafa7dd203517476e64e915c8101b2e27`·outer `/messages/0/content`·extraction `asr_request_after_no_think_lf_v1`·inner `/5`와 수정문을 확인했다. 원본 asset_utf8·원생성 반복 raw·원문6개가 함께 보존되어 있다. 최초 CLI 호출은 잘못된 옵션 형식으로 invalid_argument였으며 성공 검증에 포함하지 않는다. 정확한 성공 응답은 r164/readback-correction.json이다.

## 메인 모니터 전환

최신 사용자 지시에 따라 가상모니터 전용 검증을 종료했다. 기존 Rust validation 도구의 inventory는 DISPLAY1 primary [0,0,3440,1440]/96 DPI와 DISPLAY5 [3440,0,7280,2160]/96 DPI였다. 소유 constant MONITOR_ID9와 기존 설치/제거 기록을 확인하고 `display-driver remove`로 ID9만 제거했다. 후속 inventory는 물리 DISPLAY1 한 개·96 DPI였다. 드라이버/인증서 삭제·물리모니터 설정·DPI·VM 변경은 없다.

동일 창을 다시 활성화한 실제 screenshot origin은 (55,68), 크기760×760이며 /5 원문·수정문·채택3이 그대로 남았다. 이어 CARD 탐색은 메인 모니터에서 실제 클릭으로 수행했다. 이전 가상모니터 관측과 이후 메인모니터 관측을 구별한다.

## 미검증

추가 메인모니터 CLIP AI 추천 관측: actual-qwen06-highlights를 조회하여 ranges/cue_ids1…7/reason 원생성 raw를 별도 읽기 전용에서 확인했다. 하이라이트 추가→제목 `기술 검증용 하이라이트 · 手動確認` 입력→원문 cue1 선택→저장 응답→별도 채택 클릭을 수행했다. GUI 출력3·재채택 비활성화와 고정 CLI readback-highlights.json의 ok=true/draft2→output3/source_refs[1]/automaticfalse를 확인했다. 원문7개·원생성 raw와 null 시간은 보존된다. 따라서 이번 실행에서 교정·카드·하이라이트 각 한 건(서로 다른 세 모델)의 실제 수동 저장·채택 연결을 확인했으며 9건 전부 검증과 구별한다. 사용자 확인용 소유 창 한 개는 메인모니터에서 유지한다.

추가 메인모니터 CARD 관측: actual-llama1b-cards의 원생성 cue_ids6/7·headline·summary를 읽기 전용에서 확인했다. 카드 추가 후 앞면 `기술 검증용 카드 · 原文比較`, 뒷면 `저장·채택 연결 검증용 수동 문구. 모델 품질 판정 아님.`을 별도 입력하고 실제 cue1을 선택했다. 원문 cue1…7은 그대로 노출되며 시간 값은 만들지 않았다. 저장 버튼 클릭·응답 후 별도 채택 클릭·응답에서 출력3과 재채택 비활성화를 확인했다. 고정 CLI readback-cards.json은 ok=true, draft2→output3, source_refs[1], 원문7개와 원생성 raw 보존, 모든 cue start/end null, automaticfalse다. 좁은 화면에서 근거 목록이 길어 저장까지 여러 스크롤이 필요한 배치 문제는 잔여 개선이다. 공유 소스 lease 동안 presentation 변경 없이 기록한다.

이 기록은 Gemma 문서 교정 한 건의 실제 편집·저장·채택과 메인모니터 이동 근거다. r164 재실행 복원, 나머지 실제9 GUI 인수, 전체 시안 시각 일치, 모든 키보드·저장·오류 경로는 아직 완료가 아니다. 프로젝트 영속 저장은 SDK의 별도 projects 계약과 후속 통합한다.
