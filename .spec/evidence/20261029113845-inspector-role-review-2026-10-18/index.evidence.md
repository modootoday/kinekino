---
id: evidence:kinekino/20261029113845
title: 검사기 역할과 프레임 정보 실제 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 검사기 역할과 프레임 정보 실제 관측

2026-10-18. [컨트롤 계약](../../player/presentation/shared/inspector-controls.md)과 [이전 구성 검토](all-pages-composition-review-2026-10-18.md)의 후속이다. 기준은 기존 FRAME/CLIP/PROJ 요소 계약이며, 이 기록은 전 요소 시각 일치 승인이 아니다.

## 실행 조건

소유 가상 모니터에서 1440×900 검토 창을 사용했다. VM·DPI·모니터 토폴로지는 변경하지 않았다. `runtime/review-runs/ui-all-pages-20261018-r11/player.exe`에서 역할과 조작을 관측한 뒤 종료하고, r12 복사본(PID 88708)으로 프레임 메타데이터를 재검증했다. 화면은 직접 관측했으며 저장된 캡처 파일은 없다. r12도 닫기 버튼으로 종료했고 player.exe 프로세스 조회 결과는 0개였다.

## 확인 결과

| 범위 | 실제 관측 | 한계 |
|---|---|---|
| PROJ 깊이 | 선택 필드의 공통 SVG 화살표, 값 미상의 손잡이 없는 깊이 track와 —, 고정 footer | 실제 모델·깊이 값 및 결과 미검증 |
| FRAME 원본 | readout과 출력 select/text가 구별됨 | 실제 출력 명령 미연결 |
| FRAME 썸네일 | 폰트 select·크기 track·외곽선/그림자 indeterminate·고정 footer 확인 | 색 swatch 및 편집 가능 상태 미구현 |
| CLIP 수동/추천/일괄 | select·toggle·scalar·readout 역할과 footer 확인 | 값과 선택 목록을 발명하지 않은 disabled 상태 |
| CLIP 추천↔일괄 | 방향키 이동 확인. 일괄 inspector를 아래로 스크롤한 뒤 추천으로 이동하면 추천의 위쪽 위치가 유지되고, 일괄 복귀 시 기존 아래쪽 위치가 유지됨 | 모든 화면의 스크롤 보존을 증명하지 않음 |
| FRAME r12 | 소스 `디자인 검토용 예시 영상`, sourcePTS `00:00:52.000`, 해상도 `1672 × 941`, 색 공간 — 확인 | 해상도는 등록된 DesignSample 이미지 크기. 실제 미디어 메타데이터 검증 아님 |

## 자동 검사와 남은 범위

위 실행본의 Rust 변경 후 cargo fmt --check, cargo test, cargo clippy --all-targets -- -D warnings와 player 빌드가 통과했다. 테스트는 71개 통과·외부 의존 5개 ignored였으며, 실제 화면 관측과 구별한다.

SEG/SUB/OBJ/SAVE/MODEL/JOB/OUT의 일반 필드는 역할별 이관이 남아 있다. Color/Path/Multiline, 편집 가능한 typed value와 intent, 전체 하위 탭의 Empty/Populated/LongLabels/NeedsInput 및 좁은 창의 실제 비교도 남아 있다. 이번 관측으로 전체 시각 일치나 모든 키보드 상태를 완료 처리하지 않는다.

## r13 — SEG/OBJ/SAVE 후속

`runtime/review-runs/ui-all-pages-20261018-r13/player.exe`(PID 103140)를 가상 모니터 위치 3500,100에서 하나만 실행했다. 변경 후 fmt/test/clippy/build가 통과했다(71 passed, 외부 의존 5 ignored).

- 1440×900 SEG: OP/ED/건너뛰기 동작의 화살표, 자동재생·회차 확인의 indeterminate/알 수 없음, 예고 시간의 손잡이 없는 track와 — 확인. 최근 기록과 footer 유지.
- 1440×900 OBJ: 분석 방식·모델 select, 선택 구간 —, 불투명도·경계 track 확인. 분석 방식의 최종 segmented와 실제 mask contour는 미완료이며 현재 선택 bounds를 mask 완성으로 해석하지 않는다.
- 1440×900 SAVE: 저장 방식은 전체 미선택+연결 필요. 세 트랙은 indeterminate+확인 필요. 형식 select, 경로/이름 field, 예상 크기/이어받기 — 확인. 미상 값을 false/원본 저장 기본 선택으로 대체하던 표시를 수정했다.
- 760×480 SAVE: 설정 창을 열어 본문 스크롤 시 경로·파일명·크기·이어받기 항목에 접근하고 하단 두 버튼이 유지됨을 확인했다. Escape는 설정만 닫고 영상 저장 화면을 유지했다. 창 크기만 변경했으며 DPI/VM 변경은 없다.

r13은 캡션 닫기로 종료했다. 이번 이관은 SUB/MODEL/JOB/OUT, 편집 가능한 값, Color/Path/Multiline, 전 상태·전 폭 실기 비교를 포함하지 않는다. 시안 확대·최종 segmented/트랙 selector·세부 마스크 대응은 계속 남아 있다.

## r14 — SUB 5개 탭

`runtime/review-runs/ui-all-pages-20261018-r14/player.exe`(PID 99036), 가상 모니터 3500,100·1440×900에서 관측했다. ASR의 track/language/model select와 자동 문장부호 indeterminate, 화자 모델/범위/출력 label select와 인원 scalar, OCR의 language/model·interval scalar·검토 toggle·영역/구간 readout, 교정의 비교/근거 readout·교정 toggle·사전 field, 변환의 format/encoding/range select·offset/path field·보존 toggle을 확인했다.

화자 탭 클릭 후 Right로 OCR, Right로 교정, End로 변환 이동을 실제 확인했다. 변환 진입 직후 request_discard 경고가 한 프레임 관측됐지만 후속 안정 프레임에는 없었다. 반복적인 전환 성능 검증으로 확대하지 않는다. 각 inspector footer가 유지됐다. 실제 SDK 파일 변환·IME·좁은 SUB 창은 이번 실기 검증 범위가 아니다. 화자 전용 이미지는 여전히 없다.

OCR은 시안과 달리 파란 8개 사각 handle을 사용하는 문제를 발견했다. 기존 `sub-ocr-roi-2x.png` 확대 원본을 다시 열어 황색 외곽선·흰 원형 네 모서리를 확인했고, 전용 `paint_ocr_region`과 색/반지름 토큰으로 수정했다. r14는 수정 전 관측이며 캡션 닫기로 종료했다. 수정 후 실제 재검증은 아래 후속 기록으로 구분한다.

### r15 OCR 재검증

수정 후 fmt/test/clippy/build 통과(71 passed, 외부 의존 5 ignored). r14 종료 후 player 0개를 확인하고 `runtime/review-runs/ui-all-pages-20261018-r15/player.exe`(PID 25056)를 같은 가상 모니터 위치·1440×900으로 실행했다. OCR 탭에서 황색 테두리, 흰 원형 네 모서리, 중간 사각 handle 제거를 실제 확인했다. 영역은 DesignSample 장식이며 OCR 인식·ROI 편집을 검증한 것이 아니다. 캐릭터/자막 예시의 내용·위치까지 원본 시안과 동일하다는 의미도 아니다. 검토 후 캡션 닫기로 종료했다.
