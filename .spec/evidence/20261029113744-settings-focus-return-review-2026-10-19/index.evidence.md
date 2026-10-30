---
id: evidence:kinekino/20261029113744
title: 설정 닫기 포커스 반환 실제 검토 — r169
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 설정 닫기 포커스 반환 실제 검토 — r169

## 실행과 검사

순수 package 추출75c8 기준과 별도인 focus 수정 EXE `runtime/review-runs/ui-focus-20261019-r169/player.exe`, SHA `cc4b1d3208ab73820dd75435b0a446101622028c3e512f28e1c6c18dab952071`를 사용했다. CLI SHA `357a6c399af2aab709b6ca14deba2fa1ce4cf0c4c700fd2a3678b829b6923456`. SDK는 source사본을 고정하고 fmt --all --check/workspace test156 passed·9 ignored/strict all-targets Clippy/bins examples build exit0를 확인했다. 새 회귀는 settings_escape_returns_focus_to_trigger_for_keyboard_reentry다.

기존r168 소유 창 caption닫기 뒤 freshpath[]를 확인하고 별도복사store와 새 고정bin을 실행했다. PID48588/window13897426, 메인100,100에 제품GUI한개를 활성화했다. VM·DPI를 변경하지 않았다.

## 실제 조작

시작→제작1440×900→검토크기760×760→SEG설정 클릭으로 패널을 열었다. Escape 한 번으로 설정을 닫고 trigger의 amber focus 외곽선을 확인했다. Tab 없이 Return 한 번으로 설정 패널이 다시 열렸다. 패널 닫기X 클릭으로 닫은 뒤에도 trigger focus가 표시되고 Return 한 번으로 다시 열렸다. 마지막 Escape로 닫혀있는760제작화면을 유지했다.

이는 [r165 기존 실행](proposal-layout-panel-audit-2026-10-19.md)의 Escape 후 Return 재진입 안됨을 수정한 실제 근거다. actual trigger response에 focus를 반환하고 안정 ID를 유지하며 버튼이 없는 화면에는 반환을 미루지 않는 source변경을 사용했다. 패널 크기·토큰·하단 footer 구조는 유지됐다.

## 미검증과 잔여

후속 검사기 없는 화면 전환: SUB760 미저장문자열이있는상태로설정을열고OUT탐색을클릭했다. OUT에는설정trigger/drawer가없고본문내보내기요약·저장위치가표시됐다. SUB탐색으로복귀했을때drawer는닫힌상태이고탐색SUB에focus외곽선이유지되며미저장문자열·/5선택·문서SHA도남았다. 따라서 noinspector전환의패널닫기·복귀입력보존·지연focus강탈없음은이한경로에서실제로확인했다. wide resize와최초drawer내부focus는여전히미검증이다.

후속 SUB교정760 실제 범위도 추가했다. 같은r169GUI에서 Gemma문서 저장자료를 조회해 /5원문과수정문·초안3이 복원됐다. 수정문끝에 `[미저장 포커스 검증 · 手動]`을 추가하자 제안ID/조회가 비활성화됐다. 설정열기→Escape닫기→Tab없는Return재진입→Escape닫기 후 동일문자열·원문/5·문서SHA가 유지됐다. 새save/accept는 누르지 않았고 이입력은r169복사fixture의메모리buffer에만 남긴 상태다. 따라서 dirtybuffer를이전의전부미검증으로확대하지않으며 실제확인범위는SUB760해당문서한건이다. 다른task/resize/wide/최초내부focus는 계속잔여다.

초기 실제 범위는760 SEG의닫기→trigger반환이며 아래 후속에SUB760dirtybuffer보존을추가했다. 다른task/resize중반환/최초패널내부키보드진입·전체focus순환은 별도검증이다. wide/noinspector에서반환폐기는source에구현됐지만실제인수는아직없다. inspector실제제안상태불일치·본문선행editor의canvas밀림·StartScreen수동배치는 다음별도source단계다. 전체시각일치완료가아니다.
