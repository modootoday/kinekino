---
id: evidence:kinekino/20261029113852
title: 전체 제작 화면 구성 1차 구현·실기 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 전체 제작 화면 구성 1차 구현·실기 관측

관측 날짜: 2026-10-18. 기준은 [화면 구성 정본](../../player/presentation/shared/screen-composition.md)과 각 화면의 visual 계약이다. **시안 일치 완료 또는 전 요소 인수 기록이 아니다.**

## 구현 범위

`presentation/studio/screens`에 SEG/SUB/OBJ/CLIP/FRAME/CARD/PROJ/SAVE/MODEL/JOB/OUT 전용 조합을 추가했다. A2D는 기존 전용 편집기를 유지한다. 공용 Artwork/Card/Toolbar/Filmstrip/Waveform/Stepper/Input/Inspector와 Flexbox/Grid 조합을 사용한다. 값·색상·선·모서리는 design_tokens에서 공급한다. Inspector의 미연결 항목은 작업 성공이나 유효한 설정으로 위장하지 않는다. 실제 자막 변환의 SDK 폼·파일 선택·손실 검토 경로를 보존한다.

파형은 DesignSample의 명시적 그래픽 fixture이며 실제 오디오 분석 결과가 아니다. Filmstrip은 같은 등록 이미지의 예시이며 실제 연속 추출 프레임이 아니다. 투영 장면은 예시 평면과 축이며 추론 깊이·GLB·파일 생성 결과가 아니다. Live에는 샘플을 주입하지 않는다.

## 실제 관측

검토 복사본: `runtime/review-runs/ui-all-pages-20261018-r3/player.exe`, PID110476. 실행 옵션 `--ui-demo --record-validation --validation-position=3500,100`. Computer Use snapshot의 origin(3500,100), 크기1440×900을 확인했다. 기존 가상 모니터에서 실행했으며 VM·DPI를 변경하지 않았다. 사용자가 직접 종료할 수 있도록 모든 review player를 유지했다.

| 화면 | 직접 확인한 구성 | 발견·후속 |
|---|---|---|
| SEG | 소스 미리보기, transport, filmstrip, 감지 구간 표, 우측 검사기 | 긴 전체 영상에서 짧은 구간 라벨 겹침 발견. 폭이 충분할 때만 라벨을 그리고 hover로 전체 라벨 제공하도록 수정 |
| CARD | 세로 번호 rail, 정사각형 canvas, 제목/부제, 검사기 | 최초 Taffy 측정 frame과 안정 frame을 구분해야 함. 게시 대상·비율·템플릿·swatch·alt 전용 컨트롤 미완료 |
| FRAME | 원본 탭, 원본비율 이미지, 이전/고정/다음 도구, 3개 썸네일 | 하단 밀림 발견 후 preview 예약 높이 조정. 기본 가운데 프레임 선택 수정 |
| CLIP | 수동 탭의 세로 crop/grid와 원본 비교, 시간 구간, 검사기 | 3개 실제 track 조합 미완료. 후속 코드에 데모 파형 추가 |
| PROJ | 깊이 탭의 원근 평면, 축/grid, inspector, 도구 | 예시 장면임을 명시. package 탭 전체 실기 인수 미완료 |
| SAVE | 소스 카드, preview/transport, 3개 트랙 선택 행, 저장 모드 검사기 | 트랙 메타데이터 UI projection 미완료 |
| MODEL | 검색, 후보 카드3, 기능별 접기, 정보 검사기 | 전용 category/filter/badge 및 실제 모델 메타데이터 projection 미완료 |
| JOB | 6개 상태 탭, 동시 실행, 작업 표, 우측 stage/입력/진단 | 전체 시작 버튼의 과도한 전체폭을 compact variant로 수정. 필터별 데이터 projection 미완료 |
| OUT | 썸네일·파일 요약, 경로/이름, 충돌 옵션, 확인/버튼, 검사기 | 실제 충돌 파일 preview·preflight 연결과 전용 컨트롤 미완료 |
| SUB | ASR/OCR/교정/변환 탭 각각 preview·표·설정과 변환 비교 패널 | OCR에 부적절한 중앙 pivot 발견. 후속 코드에서 실제 image rect 기준 하단 ROI/피벗 없는 bounds로 수정. 화자 탭 전용 시안 없음 |
| OBJ | 도구, preview, 선택 장식, strip, 피사체 목록 | 임의 중심 pivot 제거와 source image rect 범위로 후속 수정. cyan mask contour 전용 컴포넌트 미완료 |

위 관측 이후 수정한 파형·ROI·filmstrip 라벨·FRAME 높이·compact 버튼·A2D980미만 detail 적층은 별도 새 실행에서 재확인해야 한다. r3 이미지는 수정 후 빌드의 시각 증거가 아니다.

## 검증 범위

Rust headless 검사에 모든 하위 탭 × Empty/Populated/LongLabels/NeedsInput ×760/1280/1440 폭을 추가했다. Taffy 측정/안정 두 pass를 실행한다. 이는 panic/렌더 경로 검사이며 픽셀 유사도·넘침·접근성·입력 인수를 입증하지 않는다. 실제 확대 비교와 전용 컨트롤 구현을 계속한다.

남은 전체 범위는 [후속 계획](../../../plans/player/presentation/visual-fidelity-follow-up.md)에 유지한다. 디자인 완료 승인 또는 `.spec` 전체 재배치 승인은 요청/획득하지 않았다.

## r4 후속 관측

사용자 정정 후 소유 검토 PID33516/56484/54492/110476을 정리하고 player0개를 확인한 뒤, r4 하나만 실행했다. PID58216, 경로 `runtime/review-runs/ui-all-pages-20261018-r4/player.exe`. 이후에도 소유 PID/경로 확인 후 교체하며 검토 창을 누적하지 않는다. 앞의 “사용자가 직접 종료” 기록은 당시 방침의 이력이며 현재 규칙이 아니다.

- SEG 라벨 겹침이 제거된 것을 직접 확인했다. 전체1420초에 앞65초 구간이 압축되는 가독성은 남아 표시 시간창을 조정하는 후속 코드를 작성했다.
- CARD 게시 대상/비율/템플릿 선택 그룹, 제목/본문·자동맞춤 안내·Pretendard·요약/alt 영역을 직접 확인했다. 실제 명령 연결은 비활성이다. rail이 측정 후에도 첫 thumbnail 일부만 보여 Taffy leaf 안 ScrollArea의 최소 스크롤 높이를 확보하는 후속 수정을 했다. 후속 빌드 재검토 전 해결로 처리하지 않는다.
- FRAME 원본/썸네일 탭 모두 하단3개 reference와 라벨이 큐 위에 배치되고 가운데 frame 선택이 강조됨을 직접 확인했다. 썸네일 탭의 grid와 제목도 확인했다. 아직 레이어·텍스트 전용 입력 UI는 미완료다.
- CLIP 수동 탭의 portrait crop/원본 비교·명시적 데모 파형·자막 시간 블록, 추천 탭의 portrait preview/후보3개/근거/도구를 직접 확인했다. 추천/일괄 inspector가 수동과 동일한 문제를 확인해 탭별 field/action schema를 분리했다. 일괄 회차 카드의 선택/thumbnail/범위/후보/상태 전용 구조도 후속 코드로 추가했다. 이 후속 코드는 아직 실제 화면 인수 전이다.

지속적인 소스 수정과 실제 렌더 비교가 필요하며 전체13화면/모든하위탭 시각 일치는 여전히 미완료다. 개발 도구 연구는 별도 사용자 요청 세션으로 이관했으며 제품 시각 인수의 대체 근거가 아니다.

## r5 후속 관측

r4 소유 PID/경로 확인 후 종료, player0개 확인, r5 PID57232 하나만 실행했다. SEG 표시 범위00:00–00:01:18.000에서 A/B/C bounds·라벨이 분리되고52초 playhead가 구간과 독립적으로 표시되는 것을 확인했다. source duration은 바꾸지 않고 UI 표시 범위만 fit했다.

CARD 최소 스크롤 높이 수정 후3개 썸네일 전체를 직접 확인했다. 두 번째 클릭 후 amber 선택·큰 canvas 제목/부제·우측 제목/본문·2/3 페이지 표시가 함께 변경되는 것을 관측했다. 최초 클릭이 반영되지 않은 경우 입력 대상 창을 재활성화하고 새 snapshot으로 재시도했으며, 이를 제품 선택 실패로 단정하지 않았다.

CLIP 일괄 탭의 전용 검사기는 확인했다. 회차 표의 nested push_id가 Grid 행 경계를 감춰 항목을 수평으로 연결하는 오류를 실기에서 발견해 row scope를 제거했다. 클릭 후 방향키 입력은 실제 전환을 입증하지 못해 tab clicked 경로에 명시적 request_focus를 추가했다. 두 수정은 다음 실행 재검증 대상이며 해결 완료로 보고하지 않는다.

이번 검토 종료 시 r5 캡션 닫기 버튼(1418,16)을 입력했고 이후 process inventory에서 player0개를 확인했다. 입력 직후 window enumeration은 아직 이전 항목을 반환했으므로, 즉시 목록만으로 종료 성공을 판정하지 않았다. 다른 앱은 조작하거나 종료하지 않았다.

## r6/r7 후속 관측과 공통 InspectorPanel

r6(PID8240)에서 일괄 회차 표3행·6열 정렬을 확인했다. 일괄 탭을 클릭한 뒤 Left 입력으로 AI추천 탭과 전용 검사기가 함께 바뀌었다. 이로써 위 두 후속 수정의1440×900 실제 재검증을 완료했다. 전체 탭/상태/키보드 인수로 확대하지 않는다.

투영 시안의 하단 경고/동작을 다시 확인하고 원본 영역(1128,730,440,162)을2배 확대했다. 파생물 `runtime/visual-review/projection-footer-20261018-r6-2x.png`, 원본은 변경하지 않았다. 경고 callout·두 버튼의 고정 하단 구조와 border/radius·disabled 대비를 확인했다. callout 세부 표현과 dropdown·swatch 등은 여전히 후속이다.

공용 `components::inspector::panel`을 추가해 스크롤 본문과 고정footer112를 분리하고 CARD/FRAME/CLIP/PROJ에 적용했다. 각 화면이 명령 의미를 소유하며 공용 패널에 workspace/application 타입을 넣지 않았다.

r6 소유 PID/경로 확인 후 종료하고 r7(PID86068) 하나로 교체했다. CARD1440×900에서 본문 scroll 전/후에도 선택카드저장/전체저장 버튼과 로컬 저장 안내가 동일한 하단 위치에 남고 alt 항목에 접근함을 관측했다.760×480에서 thumbnail rail이 가로로 전환되고 설정 창의 본문은 scroll, 저장footer는 창 내부에 유지됐다. Escape는 설정만 닫고 카드 화면을 유지했다. 작은 창에서 큰 canvas는 본문 아래로 scroll이 필요하며 이는 전체 canvas fit 완료를 뜻하지 않는다. FRAME/CLIP/PROJ의 새 footer 실제 상태별 검토는 미완료다.

변경 후 fmt/test/clippy/build exit0. 제품 test는52 library+2CLI+3CLI계약+2frame+9subtitle=68pass, 외부 미디어5ignored. 전체하위탭×4fixture상태×3폭 headless 검사는 통과했지만 실제 시각 인수를 대신하지 않는다. 별도 spec check는 checked202/canonical_broken_links0/preserved_upstream1/immutable105 확인(해당 실행 이후 문서 append가 있으므로 최신 모든 문서의 재검사 완료로 확대하지 않음).
