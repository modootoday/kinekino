---
id: evidence:kinekino/20261029114018
title: 공용 안내·경고와 검사기 footer 후속 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공용 안내·경고와 검사기 footer 후속 검토

기준: [CARD-08/09](../../player/presentation/studio/cards-visual.md), [PROJ-14/15](../../player/presentation/studio/projection-visual.md), [OUT 계약](../../player/presentation/system/export-visual.md), [전체 구성 관측](all-pages-composition-review-2026-10-18.md). 2026-10-18, 전체 시안 인수는 미완료다.

## 기준 요소와 변경 책임

투영 원본 `projection-package.png`의 하단 경고·동작 영역(1128,730,440,162)을2배 확대한 파생물 `runtime/visual-review/projection-footer-20261018-r6-2x.png`에서 아이콘/제목·본문 계층·얇은 border·radius·배경 대비를 재검토했다. 원본은 변경하지 않았다. 생성 이미지의 글로우 같은 비일관 장식을 제품 규칙으로 승격하지 않는다.

공용 `notice::show(Kind,title,detail)`는 Warning/Info 수작업 SVG, 타이틀 amber/본문 secondary, notice surface/border, radius5·border1·padding12·icon18·gap8·본문 전4를 사용한다. 이는 논리 토큰 결정이며 원본 raster 치수의 실측값이 아니다. CARD 자동맞춤, PROJ 투영 한계, OUT 충돌 미확인 안내가 공유한다. 실제 검증 상태·문구 의미는 각 화면이 소유한다.

기존 `InspectorPanel`의 고정 footer112를 SEG/SUB/OBJ/SAVE에도 적용했다. 화면별 명령을 공용 컴포넌트에 하드코딩하지 않았다. 실제 SDK 자막 변환의 별도 폼/손실 확인 경로는 유지한다.

## 실제 r8 관측

소유 검토 player0개 확인 후 `runtime/review-runs/ui-all-pages-20261018-r8/player.exe`(PID113144) 하나 실행.1440×900, 가상 모니터 위치3500,100. VM·DPI 미변경.

- SEG 검사기의 수정/현재회차저장 버튼이 하단 고정 영역으로 이동했음을 직접 확인했다.
- CARD info SVG·텍스트 자동맞춤 title/detail·얇은 외곽선·둥근 모서리·어두운 amber surface가 표시되고 오른쪽 경계를 넘지 않음을 확인했다. 저장footer도 유지됐다.
- PROJ warning SVG·상대 깊이/복원 한계 문구가 wrap되고, 검증/패키지저장 버튼이 하단에 남는 것을 확인했다. 단, 시안 dropdown/group 구성은 미완료다.
- OUT 충돌 검증 필요 title와 “현재 충돌 여부 미확인” detail이 별도 경고 박스로 표시됨을 확인했다. 실제 파일 충돌·비교 결과가 있는 것으로 표시하지 않았다.
- SUB ASR의 파형과 검토/시작 고정 footer를 확인했다. 파형은 명시적 DesignSample이며 실제 오디오 분석 증거가 아니다.
- OUT 최초 전환 snapshot에 debug `request_discard` 연속 frame 경고가 나타났다. 전환 직후 snapshot을 최종 안정 배치의 증거로 일반화하지 않으며 반복 측정/settling은 후속 검토한다.

## 발견한 추가 차이와 후속

PROJ package 탭의 scene/tree가 같은 폭이라 scene이 지나치게 좁고, 검증 결과가 큰 read-only 필드로 나열돼 하단을 밀었다. 공용 Taffy `detail_sidebar`와 `PACKAGE_TREE240`으로 유동 scene/고정 tree를 분리하고 검증영역을 항목/상태/비고3열6행 표로 수정했다. 실제값은 전부검증전이다. 이 수정의 별도 빌드/실기 확인 전에는 해결로 판정하지 않는다.

후속 필수: 새 package layout 실기, OBJ/SAVE·SUB 각 하위탭의 footer/좁은설정창, caller별 scroll ID 격리, select/checkbox/swatch/slider 역할을 보존한 typed inspector, 실제 stable row/subview interface. generic “연결 필요” 필드의 외형 개선만으로 시안 완료를 선언하지 않는다.

공유 검사 중 SDK 소유 새CLI테스트의 sha2 unresolved를 관측해 소유세션에 전달했다. 담당자가import를제거한후 최신 fmt/test/clippy/build exit0을 확인했다(library54pass, 새CLI identity1pass 포함, 외부5ignored). 이후 package layout 코드 수정의 별도 검사 실행은 별도 결과로 기록한다.

## r9 관측

r8소유PID113144/경로를확인해종료하고player0개확인후r9(PID91372)하나로교체했다. package탭에서유동scene/고정tree240배치와3열검증표를직접확인했다. 폭비율은개선됐지만표의기본행높이로하단행이밀리는것을발견해min_row_height(body13+tight4)와동일3열최소폭을명시했다. 이마지막수정의실기재확인전전체6행표시완료로판정하지않는다. r9원근plane은계속명시DesignSample이다.

## r10 재검증

마지막 표 수정 후 fmt/test/clippy/build exit0. r9는 소유 PID/실행 경로를 대조해 종료하고 player0개를 확인한 뒤 r10(PID102376, `runtime/review-runs/ui-all-pages-20261018-r10/player.exe`)하나로 재검증했다.

1440×900 package 탭에서 넓은 scene·우측 tree240·전체폭3열 검증표를 확인했다. 좌표계/scale/UV/alpha/material/sourcePTS6행이 모두 queue 위에 표시되고 상태는 검증 전을 유지했다. warning SVG/callout과 고정 footer도 표시됐다. 이는 실제 깊이 추론·파일 존재·대상 소프트웨어 import 증거가 아니다. 좁은 package 배치와 세부 dropdown/파일 glyph는 미검증/미완료다. 검토 후 캡션 닫기 입력과 후속 process inventory로 player0개를 확인했다.
