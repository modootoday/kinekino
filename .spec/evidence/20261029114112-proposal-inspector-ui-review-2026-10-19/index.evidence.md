---
id: evidence:kinekino/20261029114112
title: 공통 제안 검사기 실제 UI/UX 검토 — r170
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공통 제안 검사기 실제 UI/UX 검토 — r170

## 실행·자동검사 범위

고정 `runtime/review-runs/ui-proposal-inspector-20261019-r170/player.exe` SHA `17b2e93b1f893dfe418eebd2d6ecc0dec50dd76879f231df2f5a1ab61590714e`, CLI `d583d9399baa609e9b5b9f57eb1656cc5a07f30e383bc10106e1418edc8c20a7`. SDK소유묶음은 workspace166 passed·9 ignored/fmt/strict all-targets Clippy/build가모두exit0. 24개긴카드의300×600고정footer회귀도통과했다. 기능정확성은SDK/CLI정본으로판정하며GUI관측은배치·입력·상태·연결UX에한정한다.

시작전r169경로fresh창목록[]를확인했다. 새고정bin과r169저장소복사본을r170에두고PID32284/window67048342를메인100,100에활성화했다. 제품GUI한개,VM·DPI변경없음. r169미저장검증문자열은기록으로남았으며새자료에는저장된결과만복사됐다.

## 실제 CARD 1440×900

시작→제작→CARD에서도구strip이본문y122에표시되고canvas는대략(418,184)…(1046,812)에배치됐다. 이전r168의선행제안편집기로도구가y291/canvas가y353부터시작한것과달리작업영역이본문상단에유지됐다. 첫제안검토클릭은전환되지않아freshcapture후한번재클릭했으며두번째실제전환을근거로한다.

우측작업설정/제안검토모드전환후에도본문도구/canvas위치는동일했다. actual-llama1b-cards의고정결과를조회하자표시중인제안ID·원문revision1/초안3·앞면한글/일본어·뒷면줄바꿈·실제cue1선택이표시됐다. 이전미연결문구의별도검사기는검토모드에서동시에나타나지않았다. 하단에는비활성저장/재채택버튼과채택출력3·문서결과/미디어내보내기구별이표시됐다. 새저장/채택기능을반복시험하지않았다.

우측본문만스크롤해cue1…7까지확인했을때canvas와하단버튼y751…791·상태약y803/823은유지됐다. 기존처럼긴근거목록뒤에서저장동작을찾도록전체본문을스크롤하지않는다. 선택/포커스/모드의amber외곽선과고정footer구분선을직접확인했다. 전체시각일치가아니라본문밀림/저장접근배치개선의관측이다.

## 사용자 중단·잔여

760크기메뉴를열려고한시점에Computer Use가물리Escape중단을보고했다. 그뒤추가입력을하지않고즉시종료보고했다. 해당클릭은결과미확인이며760전환/새drawer검토로판정하지않는다. 자동goal후속만으로GUI조작을재개하지않고현재창/사용자상태를보존한다.

새배치의760drawer·SUB/CLIP각task·모드키보드/dirty보존·오류표시·실제캡션회귀는미검증이다. SDK고정자료를사용해UI/UX만후속확인한다. 작업설정에서의media/canvas미연결표시명확화와기존시안의모든옵션·시각세부,StartScreen이관·projects저장UI도잔여다. 참조확대는[별도기록](proposal-inspector-reference-review-2026-10-19.md)이며실제관측을대신하지않는다.
