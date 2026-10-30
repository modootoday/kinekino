---
id: evidence:kinekino/20261029113940
title: OBJ 실제 track 목록 행 이관
status: active
created: 2026-10-29
updated: 2026-10-29
---
# OBJ 실제 track 목록 행 이관

2026-10-19. 기준 subject.png 전체와(1198,213,374,77)4배 확대를 비교했다. 파생물 `runtime/review-runs/subject-row-r136-4x.png`에 thumbnail/amber 둥근 외곽선/label/세 점 menu가 확인된다. 기존 r134의 generic table3행과 실제 mask1track은 대응하지 않았다. SDK 소유 세션도 FK가 없음을 확인했다.

## 변경

공용 SubjectRow에44×40 thumbnail slot·label/detail·32 menu slot·PANEL 면/CONTROL radius/INLINE padding/FIELD gap을 조합했다. 세 점은 직접 작성한 more-vertical.svg다. OBJ 전용 subject_list 투영이 mask.subjects만 표시하고 실제 track ID/검토 상태를 공급한다. 사용자 이름은 없으므로 ‘이름 미지정’으로 표시한다. 유일한 현재 mask track에 amber 외곽선을 표시하며 행 선택·메뉴·이름 편집 성공을 꾸미지 않는다.

thumbnail은 allowlist 원본의 polygon bounds 영역 표시이며 alpha에서 분리한 새 자산이 아니다. 원본 크기·polygon 점 수/범위/양수 bounds를 검사하고 실패 시 비운다. SDK/domain에는 표시 기하를 넣지 않았다. crop 검사에는 원본 밖 좌표/빈 polygon/0크기 거부와 일반3행-마스크1track 불일치 표본을 포함했다.

## 실제 실행

r136 `runtime/review-runs/ui-subject-row-20261019-r136/player.exe`, 소유 PID108336, 기존 가상 모니터3500,100에서 확인했다. VM·DPI·모니터 구성 변경 없음.

- 1440×900: 목록이 한 행으로 바뀌며 사람 원본 영역 thumbnail, 이름 미지정, ‘미검토 · sample-subject-001’, amber 외곽선과 세 점 menu가 표시됐다. 마스크 한 track과 목록 수가 맞는다. 본문 합성과 도구는 유지됐다.
- 760×480 drawer: 동일 thumbnail/label/검토 상태/track ID/menu가 폭 안에 들어갔다. footer와 겹치지 않았다. Escape 후 drawer가 닫히고 기존 합성 보기가 유지됐다.
- Empty 전환→drawer: 이전 행/thumbnail/amber가 사라지고 ‘피사체 목록 · 확인 필요’ 안내가 표시됐다. 소스/마스크 없는 화면은 보기3개가 미선택·비활성이었다. 이 표본을 확인된 빈 목록0으로 과장하지 않는다.
- caption 닫기 후 CIM에 player.exe/mpv.exe가 없었다.

## 검사와 남은 차이

최초 좌표 타입 불일치를 i64 비교/음수 거부로 수정했다. 이어 main의 병행 LocalHost 오류 변환 E0277 때문에 전체 gate가 실패했으며 main은 수정하지 않고 소유 세션에 전달했다. 소유 수정 후 최종 fmt--check·전체 cargo test(lib85pass/1ignore+기본 통합/doc)·clippy all-targets-Dwarnings·player 빌드가 통과했다. UI target은 runtime/parallel-build/ui,-j1이다.

추가 피사체 header 동작, 사용자 label, 다중 track 선택/focus와 canvas 동기, menu 동작, 실제 분리 thumbnail, Live 미연결 실제 실행, LongLabels/NeedsInput 및980/1180은 남는다. 현재 행은 읽기 문맥 표시이며 시안 전체 완성이나 실제 추론 결과로 판정하지 않는다.
