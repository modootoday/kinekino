---
id: evidence:kinekino/20261029113755
title: 전체 화면의 Web UI 소비 범위 대조
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 전체 화면의 Web UI 소비 범위 대조

2026-10-20 현재 파일 직접 확인. [화면 구성](../../player/presentation/shared/screen-composition.md)과 `src/presentation/web_ui/routing/routes.json`을 대조했다. 이 기록은 source inventory이며 parser/runtime/시각 통과표가 아니다. 기존 egui 시각 관측을 새 Meari consumer 완료로 전이하지 않는다.

| 화면군 | 현재 선언 source | 전용 화면의 남은 구성/연결 |
|---|---|---|
| VIEW | viewing/player·start-screen, 공용 transport/chrome | browse/watch route source 없음. 영상·회차·전체화면·OP/ED·실제 transport/캡션 host 연결 필요 |
| SEG | segments/workspace·inspector·candidate-table | 실제 SDK read model/실행 intent·영상·OP/ED handle·되돌리기 연결, 전체 원graph 인수 필요 |
| SUB | speech/correction/conversion workspace, cue/toolbar/syntax/loss 공용 구성 | speakers/OCR source 없음. 파형/ROI·실제 cue 선택·Tabs/라우터·SDK 인터페이스 연결 필요 |
| OBJ | 후속 objects/workspace·tool-rail·view-modes·subject-row·subject-list·inspector 초안 | source-only7파일/CSS 포함. route 미등록, 실제 mask/좌표·영상·추적 시간축·SDK/입력·parser/runtime/시각 검증 필요. [작성 근거](web-subject-source-2026-10-20.md) |
| CLIP | recommendation-item, highlight-proposal-panel | 수동/추천/일괄 route source 없음. 제안 문구 패널은 crop·다중 track·후보/회차 편집 화면을 대신하지 않음 |
| FRAME | 전용 workspace 없음 | 원본/썸네일 route와 PTS filmstrip·crop/text canvas·레이어·저장 필요 |
| CARD | cards/proposal-panel | 카드 문구 SDK edit/save/accept는 P1a 검증. 카드 rail/canvas·페이지/alt/프리셋·선택/전체 저장은 별개 미완료 |
| A2D | 전용 workspace 없음 | checker canvas·mask tools·bounds/pivot·레이어·패키지 검증 필요 |
| PROJ | 전용 workspace 없음 | 깊이/패키지 route, scene/XYZ·camera·file tree·validation 필요 |
| SAVE | 전용 workspace 없음 | 영상/범위·파일 정보·codec/옵션·큐 추가 전용 구성 필요 |
| MODEL | 전용 workspace 없음 | category/list·선택 모델 정보·라이선스/revision·설치 상태/자원 설정 필요 |
| JOB | jobs/workspace·job-row·inspector | typed 목록/선택·상태filter·단계·취소/재시도/진단 SDK 연결 및 전체graph 인수 필요 |
| OUT | 전용 workspace 없음 | 출력 preview/요약·경로/이름·충돌 해결·저장 확인 전용 구성 필요 |

route catalog는 VIEW3 + studio20이다. JSON 직접 집계상 studio20 중 source 지정5, null15이며, source 지정은 실행 등록을 뜻하지 않는다. navigation은 VIEW+제작12개를 모두 선언하지만 stable scalar callback capture/P2 router가 미인수다. SUB의 TabsTrigger5개 선언 역시 본문5개 구현 또는 실제 Tabs 전환을 증명하지 않는다. JOB 상태filter를 별도 route로 부풀리지 않는다.

공통 StudioApplication은 frame/navigation/InspectorDock/queue를 원본으로 조합한다. P1a는 frame+chrome와 실제 SDK editor1개를 같은 document에 연결한 결과이며 navigation/inspector/queue는 Empty다. 실제 production application7의 Boolean callback 진단은 유지된다. P1b controlled Boolean/Dialog·P2 scalar capture/router를 인수한 뒤 동일 원본 graph로 재검증한다.

후속 순서는 공통 Inspector 계약 인수 → 공통 탐색/하위탭 전환 → 화면별 전용 컴포넌트·SDK 계약을 병렬 책임으로 나누되 단일 writer 유지 → native media parent/geometry 연결 → 전체 screen/state의 실제 검토다. 모든 화면에 Empty/Populated/LongLabels/NeedsInput을 유지하고 실행/실패/취소/완료/이전 revision은 실제 application 응답과 구별한다. 한 범용 표/제안 패널의 성공으로 나머지 전용 편집 구조를 대체하지 않는다.

새 실행·빌드·소스 변환은 이번 대조에 포함하지 않았다. P1a 실행 근거는 [별도 검증](studio-frame-sdk-2026-10-20.md), Inspector 인수 요구는 [후속 계약](../../../plans/player/presentation/inspector-dock-consumer-acceptance.md)에 둔다.
