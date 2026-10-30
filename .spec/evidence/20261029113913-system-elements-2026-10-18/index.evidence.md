---
id: evidence:kinekino/20261029113913
title: 모델·대기열·내보내기 확대 관측 — 2026-10-18
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 모델·대기열·내보내기 확대 관측 — 2026-10-18

selected_studio 3장을 Rust visual-review로 crop하고 직접 확인했다. 이미지 경로는 `.spec/artifacts/design/v2/studio/`, 출력은 `runtime/verification/design-crops/` 기준이다.

| 원본 | 영역 x,y,w,h | 배율 | 출력 |
|---|---|---|---|
| models-refined.png | 1098,50,488,840 | 2 | model-details-2x.png |
| queue-refined.png | 1120,65,450,760 | 2 | job-stage-2x.png |
| export.png | 1000,155,560,750 | 2 | export-conflict-2x.png |

모델은 미선정/미설치 pill·repo 외부 링크·revision 설명·라이선스 경고·정밀도·장치·크기·disabled 다운로드가 별도 계층이다. 대기열은 단계 원/연결선·현재 단계 banner·미상 진행값·입력 summary·행동/진단이 별도다. 내보내기는 유동 경로+찾아보기·파일명·amber 충돌 banner·radio와 새 이름 설명·속성·하단3버튼·로컬 저장 안내다. 내부 요소 정밀 bounds/색 sampling은 미수행이다.

공통화 결정: MODEL 시안의 main/tag placeholder는 검색 입력으로만 허용하고 실제 준비 immutable revision 계약을 유지한다. Q4·자동장치는 지원 보장이 아니다. JOB 시안의 입력필요+적색 진단 혼합은 상태 계약에 따라 분리한다. OUT 시안의 숏폼 선택은 진입 경로이며 OUT 목적을 감추는 규격으로 승격하지 않는다.

현재 editor.rs는 MODEL/JOB/OUT에 일반 검색·filter·table을 사용하며, preview_inspector.rs는 문자열 field 묶음을 제공한다. 시안의 accordion/stepper/저장 확인·충돌 영역을 구현했다고 볼 수 없다. 새 상세 read 계약과 컴포넌트 조합이 필요하다. 구현 기준은 [MODEL](../../player/presentation/system/models-visual.md), [JOB](../../player/presentation/system/jobs-visual.md), [OUT](../../player/presentation/system/export-visual.md)에 분리했다.

이번 작업은 문서/시안 확대/소스 대조다. 실제 설치·다운로드·queue retry/cancel·파일 충돌·저장 실행을 새로 관측하지 않았다. 기존 Rust 검사 성공을 해당 시각·동작의 증거로 사용하지 않는다. VM·DPI를 변경하지 않았고 새 GUI를 실행하지 않았다.
