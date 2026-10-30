---
id: evidence:kinekino/20261029113941
title: 내보내기 순환 검토 r148
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 내보내기 순환 검토 r148

## r156 속성 아이콘·라벨·값

export.png의(1012,578,530,218)3배 확대 `runtime/review-runs/export-properties-r156-3x.png`에서 공통 아이콘 열·라벨·값의 정렬을 확인했다. 기존 일반 폼 readout을 공용 metadata_row(glyph20/label영역144/최소32/유동value/수직중앙)로 바꿨다. 직접 작성 file-output.svg는 기존 공통 stroke1.75와 cap/join을 따른다. OUT의 실제 제공 field_id만 Folder/Warning/FileOutput/Info에 대응하고 없는 시간·해상도·오디오·자막 값은 생성하지 않았다.

session43671은 import 정렬 fmt 차이로exit1, 소유 파일 rustfmt 뒤 session8630은 fmt--check·전체test139pass9ignored(lib94pass1ignored)·clippy all-targets-Dwarnings8.07s·player build0.70s 최종exit0였다. test compile1m17s. 이전 체크포인트를 새 수정 검증으로 쓰지 않는다.

실행본 `runtime/review-runs/ui-export-properties-20261019-r156/player.exe`, PID58044, SHA256 `ab5a935bb4e57603b7e11cd65d5b59a32112fac61632c0dff9bbae0ede63ab72`. fresh sky 대상 없음 뒤 가상 모니터3500,100에 하나 실행했다. VM·DPI 변경 없음.

1440 Populated에서4개 아이콘/라벨/값의 수직 정렬과 공통 값 시작점을 확인했다. 새 파일 glyph가 형식 행에 표시되고 기존 요약 썸네일/저장 카드가 유지됐다.760×480 Populated에서는 하단으로 스크롤해4행과3개 비활성 동작·로컬 안내까지 폭 안에서 접근했다.760 Empty 전환 뒤 확인사항 heading 아래에 이전4개 아이콘/라벨/값이 남지 않았다.

현재 실제 표본 값은 짧으므로 긴 label/value wrap·hover 원문,980/1180과 실제 preflight/저장·키보드 동작은 미검증이다. 원래 시안의5개 내용 항목과 현재4개 read 항목은 같지 않으며 typed 계약 확장이 남아 있다. 이 변경은 시각 구조 개선이며 OUT 전체 완료가 아니다.

캡션 닫기 후 새 sky 목록의 r156 경로 필터는 빈 배열이었다. 창 소멸 확인이며 별도 PID 소멸 증명은 아니다.

## r155 복합 요약 행 — 실행 검증 전

export.png의(300,615,666,116)을3배 확대해 `runtime/review-runs/export-summary-r155-3x.png`로 확인했다. 시안은 둥근 썸네일 옆에 강조된 출력 이름과 원본/설명을 묶지만 기존 코드는 파일명/형식 두 문자열을 나열했다. `output_summary` 공용 컴포넌트에144×81 썸네일 토큰과 GROUP16 간격의 고정/유동 Taffy열, 이름 줄바꿈·전체 hover·형식/소스 이미지 구분 문구를 구현했다. artboard의 새 static thumbnail 경로는 공유 CONTROL radius를 이미지에도 적용한다. 기존 media preview 전체 렌더는 변경하지 않는다.

현재 read 계약에서 실제 출력 썸네일·원본 구간·설명이 없으므로 생성하지 않았다. source 이미지 재사용은 DesignSample에만 적용하고 출력 결과 이미지가 아님을 표시한다. 소스 제목의 최종 위치와 속성 icon-label-value 목록은 별도 차이다. session90524의 fmt/test/clippy/player build가 진행 중이며 아직 수정본 실제 GUI 인수로 주장하지 않는다. 아래 r148/r150은 이전 버전 관측이다.

### r155 수정 실행 결과

session90524 최종exit0: fmt--check·전체test139pass/9ignored(lib94pass/1ignored)·clippy all-targets-Dwarnings6.71s·player build0.58s. test compile1m36s. 실행본 `runtime/review-runs/ui-export-summary-20261019-r155/player.exe`, PID36504, SHA256 `b76d546809bf9ffcb06c8d3c927e2fcb8c298bb2cdcffd60933929f58289f008`. fresh sky 대상 없음 확인 후3500,100 가상 모니터에서 하나 실행했다. VM·DPI 변경 없음.

-1440 Populated: 영상 아래 요약에 세 개의144×81 소스 썸네일과 sample-001/002/003.png·PNG가 같은 행 구조로 표시됐다. 둥근 이미지 모서리와 고정 썸네일/유동 텍스트 간격을 확인했다. 각 행은 소스 미리보기·출력 결과 아님을 표시했다. 우측 저장/충돌/확인사항 카드는 유지됐다.
-1440 LongLabels: 한·일·영 혼합의 긴 파일명과 형식이 텍스트 열 안에서 줄바꿈됐고 저장 카드 폭을 밀지 않았다. 소스 제목은 현재 요약 행 위에 별도 표시돼 시안과 위치 차이가 남는다.
-760×480 LongLabels: 요약→저장 카드의 한 열 배치, 세 개의 긴 출력 행과 세 번째 썸네일 전체·다음 저장 위치 입력까지 스크롤 접근을 확인했다. 썸네일과 글자가 겹치지 않았으며 행 높이는 텍스트에 맞게 늘어났다.1440/760에서 실제 hover 원문 popup은 별도 관측하지 않았다.
-760 Empty: 이전 소스 이미지가 검정 프리뷰/소스 이미지 연결 필요로 바뀌고, 아래 내보내기 요약에는 이전 제목·세 파일명·세 썸네일이 남지 않았다. 미상 시간/비활성 transport와 저장 위치 카드는 유지됐다.

남은 차이는 시안의 원본 구간/설명 및 실제 출력 썸네일 계약, 소스 제목 위치, 속성 icon-label-value 목록·요약 제목 광학 크기,980/1180·실제 export preflight/저장/hover·키보드다. 이번 복합 행 추가를 OUT 전체 시각 일치 완료로 표시하지 않는다.

캡션 닫기 뒤 새 sky 목록에서 r155 경로 필터 결과는 빈 배열이었다. 창 소멸 확인이며 별도 PID 소멸 증명으로 확대하지 않는다.

2026-10-19. export.png 전체를 다시 대조했다. 실행본 `runtime/review-runs/ui-export-cycle-20261019-r148/player.exe`, PID114936은 통과한 r147 실행 파일 복사본이며 진행 중 JOB 프레임 수정의 검증본이 아니다. 시작 전 소유 검토 창 없음 확인, 기존 가상 모니터3500,100 사용. VM·DPI 변경 없음.

## 실제 관측

- 1440×900: 요약/저장 두 카드가 중앙 전체 폭을 공유하고 별도 inspector가 없었다. 원본 예시 이미지/3개 PNG 이름·형식, 비활성 경로/이름/충돌 선택, 확인 사항과3개 동작이 표시됐다.
- 760×480: 요약→저장 한 열 적층, 설정 버튼/drawer 없음. 본문 스크롤로 경로·파일 이름→충돌 처리→확인 사항·취소/뒤로/저장 시작·로컬 저장 안내까지 접근했다.
- 760 Empty: 이전 예시 미디어/출력 목록 대신 검정 소스 이미지 연결 필요 안내가 표시됐다. 확인 사항은 제목만 남고 값 목록이 제거됐다. 이 상태를 preflight 성공으로 해석하지 않는다.
- caption 닫기 후 새 sky 목록에서 r148 창이 반환되지 않았다. 창 소멸 확인이며 별도 PID 소멸 증명으로 확대하지 않는다.

## 발견 차이

시안은 검정 프리뷰 면/하단 transport와 별도 썸네일 요약 행·icon-label-value 속성 목록을 가진다. 현재는 카드 제목 아래 큰 이미지와 출력 파일/형식 단순 나열이며 해당 전용 구조가 빠져 있다. 기존 MediaPreview와 요약 행을 통해 전용 구성을 조합할 후속 설계가 필요하다. 시안의 실제 clip time/형식/대사를 예시값으로 복제하지 않는다.

1440과760 데이터 채움에서 충돌 radio는 미선택인데 확인 사항의 파일 충돌은 ‘새 이름으로 저장’이었다. `application/workspace_read/preview_inspector.rs:80`의 디자인 readout이 이 문구를 공급함을 확인했다. UI의 미선택 상태와 같은 계약으로 정정해야 하며 이번 실행에서는 소스를 변경하지 않았다. 실제 파일 충돌 검사 결과가 아니다.

LongLabels/NeedsInput·980/1180·키보드 Tab 순서·실제 경로 선택/쓰기/출력 검증은 이번 실행에서 미검증이다. JOB 프레임 검사 session75245는 동일 핸들로 계속 진행 중이다.

## 프리뷰·충돌 문구 수정 준비

이후 JOB 검사 session75245는exit0로 끝났고r149 실제 검토를 별도 기록했다. OUT 후속은 export.png(299,492,667,65)4배 확대(`runtime/review-runs/export-preview-transport-r150-4x.png`)에서 검정 면/하단 transport를 확인한 뒤 공용 MediaPreview::unavailable를 연결했다. ‘내보내기 요약’ 제목은 프리뷰 아래로 이동했다. 미상 시간·비활성 seek/재생 동작을 유지하고 예시 표시16:9를 실제 출력 화면비로 주장하지 않는다. SDK와 preview_inspector.rs 소유를 조율한 뒤 collision 샘플 확인값을 ‘선택 필요’로 맞췄다.

fmt--check 통과 후 session51501의 전체test/clippy/build가 진행 중이다. 새 실행본의 시각 결과는 아직 없으며 위r148 관측을 이 수정본에 적용하지 않는다. 썸네일 복합 요약·속성 아이콘/값 목록은 여전히 별도 미완료다.

## r150 수정 실행 검토

caption 닫기 후 새 sky 목록에서 r150 창이 반환되지 않았다. 창 소멸 확인이며 별도 PID 소멸 증명으로 확대하지 않는다.

session51501 최종exit0. fmt--check·전체test(lib86pass/1ignored와 기본 통합/doc)·clippy all-targets-Dwarnings·player build 통과. test compile6m50s,clippy11.83s,마지막build0.63s였다. 새 검토 실행본 `runtime/review-runs/ui-export-preview-20261019-r150/player.exe`, PID65556. r151 창 소멸 확인 뒤 기존 가상 모니터3500,100에 한 개 실행했다. VM·DPI 변경 없음.

- 1440×900: 검정 전체 폭 프리뷰 면 안에 원본 예시를contain 표시하고 하단 Play/미상시간/seek/Volume/16:9/Fullscreen을 확인했다. 요약 제목은 영상 아래였다. 경로·충돌 선택이 미연결일 때 확인 사항도 ‘선택 필요’로 표시됐다.
- 760×480: 요약→저장 적층을 유지했다. 프리뷰 하단으로 스크롤했을 때 모든 transport 요소가 폭 안에 표시됐다. 저장 하단까지 스크롤해 충돌 확인 문구·3개 동작·로컬 저장 안내 접근을 확인했다.
- 760 Empty: 이전 이미지와3개 출력 파일명이 제거됐다. 검정 연결 필요 면과 비활성 transport/미상 시간이 남고 요약은 제목만 남았다. 확인 사항의 값 목록도 제거됐다.16:9는 이 프리뷰의 배치 비례이며 미디어/출력 metadata 확인값이 아니다.

프리뷰 조작은 실제 playback capability가 없는 비활성 상태다. 실제clip PTS/duration/출력 프레임 검증으로 주장하지 않는다. LongLabels/NeedsInput·980/1180·키보드 Tab 순서·실제 경로/쓰기 기능은 수정본에서 미검증이다. 썸네일 복합 요약·icon-label-value 속성 목록·시안과 transport 색/모서리의 세밀한 차이는 남는다. caption 닫기를 요청했으며 새 창 목록으로 소멸을 별도 확인한다.
