---
id: evidence:kinekino/20261029114034
title: 검사기 첫 그룹 여백·구분선 정리
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 검사기 첫 그룹 여백·구분선 정리

2026-10-19. r141 CLIP 검사기의 제목 앞 불필요한 빈 구분선을 확인했다. 원인은 첫 그룹도 후속 그룹용 section(GROUP여백→구분선→FIELD여백)을 사용한 것이다. short-recommend-refined.png와 raw-frame.png 전체를 확인하고, raw-frame의(1207,62,366,60)4배 확대(`runtime/review-runs/frame-inspector-heading-r142-4x.png`)에서 첫 제목 위에 별도 구분선이 없음을 재확인했다.

공용 inspector::heading은 strong 제목/아래FIELD 간격만 소유한다. section은 기존 선행 간격/구분선 뒤 heading을 조합한다. CLIP3개/FRAME2개 inspector의 그룹 반복에서 첫 그룹만 heading을 사용한다. 필드·데이터·footer 의미는 바꾸지 않았다.

## 실제 실행

r142 `runtime/review-runs/ui-inspector-heading-20261019-r142/player.exe`, PID106352. 기존 가상 모니터3500,100 사용, VM·DPI·모니터 구성 변경 없음.

- 1440×900 CLIP 수동/AI추천/일괄: 첫 제목이 검사기 본문 상단에서 시작하고 앞의 빈 구분선이 사라졌다. 다음 그룹 구분선과 고정 footer는 유지됐다. 수동 첫 캡처의 이미지 loading spinner는 이번 제목 검토 완료와 별개다.
- 1440 FRAME 원본/편집: 프레임 정보/썸네일 설정이 같은 시작 기준선을 사용했다. 프레임 추출/텍스트/출력 등 후속 그룹의 구분선은 유지됐다.
- 760×480 FRAME 편집: 설정 drawer 첫 본문 제목 ‘썸네일 설정’ 위의 불필요한 여백/구분선이 없고 세 설정 필드가 표시됐다. Escape 후 같은 편집 탭/canvas로 돌아왔다. 다른4개 inspector의 좁은 창 검증으로 확대하지 않는다.
- caption 종료 뒤 CIM에 player.exe/mpv.exe가 없었다.

fmt--check·전체 cargo test(lib86pass/1ignore+기본 통합/doc)·clippy all-targets-Dwarnings·player 빌드가 통과했다. UI target runtime/parallel-build/ui,-j1 사용. LongLabels/Empty/NeedsInput/980/1180,전체 옵션·수동 crop/일괄 표 폭 등 별도 차이는 남는다. 첫 그룹 정렬 보완이며 전체5화면의 시각 인수가 아니다.
