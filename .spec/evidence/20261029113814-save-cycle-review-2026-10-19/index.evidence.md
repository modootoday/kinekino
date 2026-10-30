---
id: evidence:kinekino/20261029113814
title: 영상 저장 순환 검토 r151
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 영상 저장 순환 검토 r151

2026-10-19. download-refined.png 전체와 SAVE 요소 계약을 재대조했다. `runtime/review-runs/ui-save-cycle-20261019-r151/player.exe`, PID103516은 통과한r149 복사본이며 진행 중 OUT 수정 검증본이 아니다. 새 sky 목록에서 기존 검토 창 없음 확인 후 기존 가상 모니터3500,100에 한 개 실행했다. VM·DPI 변경 없음.

## 실제 관측

1440×900: 소스 제목 카드의 이미지/문자형 재생 영역, 트랙 표의 영상/오디오/자막 유형 SVG·미확인 선택/정보, 우측 저장 방식3개 설명·출력 설정·예상 정보·호환성 확인 문구가 표시됐다. 선택 미연결은 비활성/미선택이며 예상 파일 크기/이어받기는 대시였다.

760×480: 이미지 카드 뒤 트랙 표를 본문 스크롤로 접근했다. 영상/오디오의 트랙 정보/품질·언어가 selector 다음 줄에 표시됐다. 세 번째 자막 행은 화면 아래로 이어져 이 실행에서 좁은 폭 전체3행 인수로 확대하지 않는다. 설정 drawer에서3개 저장 방식과 설명, 고정 소스 확인/대기열 추가가 표시됐다. Escape 후 같은 트랙 본문/스크롤 위치로 돌아왔다.

첫 탐색 입력 직후/후속 캡처가 이전 SEG를 유지해, 새 관측 후 screenshotId를 지정해 다시 클릭한 뒤 SAVE 진입을 확인했다. 최초 입력이 성공한 것으로 기록하지 않는다.

## 시안과 남은 차이

실제 프리뷰는 카드 가운데 이미지와 왼쪽 회색 문자형 재생/볼륨/전체화면으로 표시된다. 시안의 전체 폭 영상면·하단 seek/시간 transport와 다르다. 소스 제목의 Folder SVG와 전용 헤더, 트랙 카드 외곽 격자·refresh 구조, 호환성 Info/Warning 카드도 추가 대응이 필요하다. 미확인 트랙을 없음으로 바꾸거나 시안의 권장 radio를 실제 선택으로 복사하지 않는다.

LongLabels/Empty·980/1180·실제 트랙 선택/저장·전체 트랙 스크롤·키보드 Tab 순서는 이번 실행에서 미검증이다. caption 닫기를 요청했고 새 창 목록으로 소멸을 별도 확인한다. OUT 검사 session51501은 그대로 유지 중이다.

닫기 뒤 새 sky 목록에서 r151 창이 반환되지 않았다. 창 소멸 확인이며 별도 PID 소멸 증명으로 확대하지 않는다. 이후 OUT session51501은exit0로 끝났으며 새 OUT 실행 검토는 별도 기록한다.

## SourceMediaCard 후속 소스 — 검증 진행 중

download-refined.png의 소스 헤더(284,143,816,75)3배와 하단(284,568,815,75)3배를 `save-source-header-r152-3x.png`/`save-source-transport-r152-3x.png`로 검토했다. 공용 SourceMediaCard에 기존Card·Folder SVG20/긴제목말줄임·MediaPreview::unavailable를 조합하고SAVE만연결했다.기존viewport기반높이와snapshot/DesignSample asset경계를유지하며시안의23:45/선택트랙을생성하지않는다. fmt--check통과후session91246의test/clippy/build진행중이며새실행관측은아직없다.위r151기준관측을수정본검증으로사용하지않는다.

session91246은exit101로 끝났다. SDK의 ReadReviewTextAsset/trait 메서드 추가 전 컴파일한 lib와 추가 후 tests/model_review.rs가 섞여 E0432/E0599가 발생했다. 현재 소스에 타입/메서드가 존재함을 확인했고 SDK가 소스 안정 및 실제 FS/CLI 통합8개·clippy 통과를 알렸다. 기존 프로세스 종료를 확인한 뒤 같은 UI target-j1의 fmt/test/clippy/build를 session25184로 다시 실행했으며 SDK 소스는 수정하지 않았다. 검사 실패 기록이며 SAVE 실행 검증이 아니다.

## r152 수정 실행 검토

caption 닫기 뒤 새 sky 목록에서 r152 창이 반환되지 않았다. 창 소멸 확인이며 별도 PID 소멸 증명으로 확대하지 않는다.

session25184 최종exit0: fmt--check·전체test(lib87pass/1ignored와 기본 통합/doc)·clippy all-targets-Dwarnings·player build 통과. test compile4m11s,clippy1m05s,마지막build0.61s였다. 이 결과는 아직 모듈 등록 전인 model_review/worker.rs/state.rs의 검증을 포함하지 않는다.

새 실행본 `runtime/review-runs/ui-save-source-20261019-r152/player.exe`, PID36708. 기존 검토 창 없음 확인 후 가상 모니터3500,100에 한 개 실행했다. VM·DPI·모니터 구성 변경 없음.

- 1440×900 채움: Folder SVG/소스 제목·전체 폭 검정 프리뷰 면·비활성 Play/미상시간/seek/Volume/표시비례/Fullscreen을 확인했다. 트랙 표와 저장 방식 검사기는 유지됐다. 첫 탐색 입력 후 이전 SEG가 남아 재관측/활성화 후 입력 결과가 실제 SAVE로 전환된 것을 확인했다. 최초 캡처를 SAVE 성공으로 해석하지 않는다.
- 1440/760 LongLabels: 한·일·영 혼합 소스 제목이 한 줄 안에 표시되고 카드 폭을 밀어내지 않았다. 이 표본은760에서도 제목이 들어가므로 실제 말줄임 발생/hover 원문 관측으로 확대하지 않는다.
- 760 LongLabels: transport 전체가 폭 안에 표시됐다. 본문 스크롤로 영상/오디오 트랙의 selector와 다음 줄 정보/품질·언어에 접근했다. 설정 drawer의3개 저장 방식 설명·고정2동작,Escape 후 같은 트랙/스크롤 위치 유지 확인.
- 760 Empty: 이전 이미지/긴 제목이 제거되고 Folder/영상 선택·소스 이미지 연결 필요·검정 면/비활성 transport가 표시됐다. 미확인 트랙을 실제 없음으로 바꾸지 않았다.

남은 차이: 시안의2줄 transport/볼륨 rail/별도 영상 동작, source header 치수와 전체폭 이미지 비례·트랙 카드 격자·호환성 Info/Warning 카드가 남아 있다. 미연결 재생/실제 트랙 선택·저장·980/1180·전체3행 스크롤·키보드 Tab/hover는 미검증이다. caption 닫기를 요청했으며 새 창 목록으로 소멸을 별도 확인한다.
