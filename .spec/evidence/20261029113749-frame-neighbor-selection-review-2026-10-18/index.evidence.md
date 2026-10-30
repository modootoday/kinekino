---
id: evidence:kinekino/20261029113749
title: FRAME-02/03 인접 프레임 표시 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# FRAME-02/03 인접 프레임 표시 검토

2026-10-18. 기준 raw-frame.png 전체와417,664,589,187을3배 확대했다. 파생물 `runtime/design-inspection/frame-neighbors-r51-3x.png`. snapshot-refined.png도 전체 검토했다. 원본은 preview→세 thumbnail/caption→동작, 편집은 canvas→동작→filmstrip 순서다.

## 변경

공용 FrameThumbnail이 기존 Artboard를 조합해 centered caption·선택 amber·별도 focus 선을 표시한다. 원본 동작을 thumbnail 아래로 옮기고 편집 동작은 위에 둔다. keyboard_selection을 사용해 좌우/Home/End 선택을 요청한다. Taffy 재계산에서 응답을 중복 추가하지 않고 item index별로 갱신한다. 현재 sample에는 인접 frame의 독립 PTS가 없어 임의 fps/시간을 만들지 않고 샘플·PTS 연결 필요를 표시한다.

## 중간 실제 관측

r51 `runtime/review-runs/ui-frame-neighbors-20261018-r51/player.exe`, PID114592. 기존 가상 모니터3500,100,1440×900, --ui-demo DesignSample이다. 원본에서 세 caption 중앙 정렬·현재 caption amber·하단 동작 순서를 확인했다. 오른쪽 thumbnail 클릭으로 다음 프레임 선택 후 Left로 현재 프레임 선택이 돌아왔다. 큰 이미지는 동일 sample 자산이며 실제 decoder seek의 증거가 아니다.

이 실행에서 focus 선이 image 대신 넓은 slot을 감싸는 시각 오류를 발견했다. 이후 공용 Artboard::fitted_rect에 image bounds 계산을 모으고 FrameThumbnail focus를 같은 bounds 안쪽에 그리도록 수정했다. r51 캡션 닫기 후 player/mpv 조회가 비어 있음을 확인했다.

## r52 수정 후 실제 재검토

`runtime/review-runs/ui-frame-neighbors-20261018-r52/player.exe`, PID96700, 같은 가상 모니터 위치3500,100.1440에서 다음 thumbnail 클릭 후 focus 선이 이미지 안쪽에만 그려졌고 넓은 slot 외곽선은 없어졌다. Home으로 첫 thumbnail을 선택해 선택선·caption amber·focus가 함께 이동했다.760×480은 기존 데모 크기 선택 기능으로 변경했다. 원본 탭에서 본문 스크롤 후 세 thumbnail/caption과 하단 동작이 모두 보였다. 편집 탭에서는 동작이 filmstrip 위에 유지됐다. 이는 native 모서리 resize 성공의 증거가 아니다.

Empty 편집 탭에서 thumbnail/caption/선택이 제거되고 source placeholder가 표시됐다. 전환 순간 egui request_discard 4연속 frame 경고를 한 번 관측했고 다음 안정 관찰에서는 사라졌다. 지속 루프가 없다는 정량 증거는 아니므로 재현·계측이 남는다. 기존 grid guide가 소스 없는 canvas에도 보이는 문제도 남아 있다. 반환 이미지를 직접 관측했으며 별도 저장 screenshot을 주장하지 않는다. 캡션 닫기 후 player/mpv 조회가 비어 있음을 확인했다.

최종 소스에서 cargo fmt --check, cargo test, cargo clippy --all-targets -- -D warnings 및 player 빌드가 통과했다. SDK 동시 변경으로 테스트 개수가 변하므로 이전 실행의80개를 최종 개수로 재사용하지 않는다. SDK 담당에 UI 파일 소유 범위와 인접 PTS 미연결 표시를 공유했다.

## 남은 차이

현재 strip 전체 폭과 세 thumbnail 간격은 기준보다 넓다. 편집 canvas의 crop/safearea/textlayer, 실제 인접 PTS·프레임 공급자 연결·저장 동작도 미완료다. 편집 시안의5개 filmstrip을 현재3개의 인접 sample로 대체한 상태도 미완료다. 일부 선택 표시를 전체 화면 일치 완료로 확대하지 않는다. VM·DPI·모니터 구성은 변경하지 않았다.
