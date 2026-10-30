---
id: sot:kinekino/visual-elements
title: 시청 화면 요소별 시각 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 시청 화면 요소별 시각 계약

[r163 시작 세부 감사](../../../verification/presentation/viewing-start-detail-audit-2026-10-19.md): 기준 원형 표식4배와 r162 실제 관측을 대조했다. 전용 stroke/비례,수동 중앙 여백·크기 상수의 토큰/공용 StartScreen 이관을 S2 인수 뒤 별도 후속으로 기록한다. 수정/시각 완료 근거는 아니다.

[r128 손잡이 토큰 검토](../../../verification/presentation/media-rail-thumb-review-2026-10-19.md):1180 일반 시청의 inline/popup amber thumb,Home0/End100 끝점·glyph·별도 focus와 비활성 seek의 중성색을 확인했다. 다른 모든 rail 상태 인수는 별도다.

VIEW-07a 키보드 후속: 볼륨 팝업을 클릭 또는 키보드로 열면 팝업 내부 rail에 한 번 포커스를 준다. 열린 동안 매 프레임 포커스를 다시 가져오지 않는다. Home/End/좌우로 조정하고 focus 외곽선을 표시한다. Escape는 팝업→회차 패널 순서로 닫으며 회차 패널을 동시에 닫지 않는다. 초기 포커스와 실제 값/zero glyph를 새 실행에서 재검토한다.

[r126/r127 실제 검토](../../../verification/presentation/viewing-panel-volume-focus-review-2026-10-19.md)에서 일반 무소스1180의 팝업→패널 Escape 순서와 클릭 열기 후 Home0/End100/Left99,Tab으로 inline rail 이동을 확인했다. 키보드 열기·좁은/실제 재생은 별도다.

[r97 캡션 cycle 검토](../../../verification/presentation/caption-cycle-review-2026-10-18.md)에서 가상 모니터의 실제 최대화→복원 glyph 클릭, 마우스/Space 최소화와 복귀, 회차 패널 보존, focus 외곽선·Enter 닫기를 확인했다. 이전 복원/최소화 미검증을 이 범위에서 보완하며 재생/전체화면 조합과 pressed 상태는 남아 있다.

VIEW-08의 실제 빈 시청 r81/r83와 동일 컴포넌트의 채워진/긴문구/Empty 데모 r82, 방향키와 Enter/Space·Escape의 구분 결과는 [회차 패널 검토](../../../verification/presentation/episode-panel-review-2026-10-18.md)에 있다. 실제 공급자 회차 전환이나 전체 시청 인수로 확대하지 않는다.

VIEW-08 후속: 회차 패널의 header/닫기·비어 있지 않은 작품명·본문 스크롤을 공용 episode_list로 분리한다. 행은 stable episode URL/ID, 최소40·왼쪽 play slot32·glyph20·현재 amber선2/문자/선택면과 별도 focus를 사용한다. 번호를 제목에서 추측하거나 분리하지 않고 제공된 제목을 보존·truncate+tooltip한다. Up/Down/Home/End는 focus만 옮기고 Enter/Space/클릭이 활성화한다. 방향키만으로 네트워크 탐색/재생을 시작하지 않는다. 실제 빈 패널과 명시적 회차 디자인 샘플을 구분해 실제 renderer에서 검토하며 샘플 활성화는 UI current만 바꾸고 URL을 열지 않는다. 작품명 빈 문자열은 행을 예약하지 않는다.

2026-10-18 VIEW-09a 후속: fullscreen에서 일반 제목줄·제작·캡션을 숨기는 경로를 정지/재생/idle 및 Escape 복귀로 [실제 검토](../../../verification/presentation/viewing-fullscreen-chrome-review-2026-10-18.md)했다. 작품명 배지·Esc 안내·영상 위 하단 합성은 여전히 미구현이며 이 확인을 전체화면 시안 일치 완료로 확대하지 않는다.

2026-10-18 VIEW-07a 후속: playback-v2의 음량 영역(973,910,210,48)을4배 확대하고 실제 mpv 재생에서 볼륨0 수치/상태를 비교한다. 공용 VolumeControl이 현재 수치와 zero-volume SVG를 소유한다. 볼륨0과 별도 mute 정책을 동일시하지 않으며 이번 범위는 후자를 추가하지 않는다. inline rail72·popup180·popup용 확보 높이64는 sizing 토큰,980 미만은 popup만 표시한다. 현재 child HWND 영상이 egui popup을 가리는 관측에 대응해 popup이 열린 동안 transport 상단 공간을 확보하고 영상 영역을 줄인다. 이는 libmpv 단일 합성 완료를 뜻하지 않으며 resize/닫기 후 실제 재검토한다.

[시청 기능 계약](viewing.md), [화면 구성](../shared/screen-composition.md), [요소 기록 형식](../shared/element-review-contract.md)을 적용한다. 참조는 v2 start/playback/fullscreen/skip/web이며 경로·crop은 [확대 관측](../../../verification/presentation/viewing-elements-2026-10-18.md)에 기록한다. 아래 수치는 최종 논리 단위의 설계값이다. 원본 raster의 정밀 측정값으로 해석하지 않는다.

## 기본·재생 화면

| 요소 ID | 구성·치수·형태 | 상태·데이터·입력 | 컴포넌트 책임 |
|---|---|---|---|
| VIEW-01a | 검정 영상면에 원형 play 표식 100×100, muted 선; 제품명 23 semibold, 동작/안내와 세로 묶음 | 소스 없음. 표식은 로딩 spinner나 재생 가능 버튼으로 오인시키지 않음 | 시작 화면 그룹·StartMark SVG |
| VIEW-01b | 파일 열기/작품 탐색 outlined 동작 40 이상, radius5, icon20, 간격12; amber 폴더/외곽선 | hover/focus 별도. Ctrl+O 안내는 보조11, 제품명은 식별 설정에서 주입 | 공통 시작 동작·Folder/Search |
| VIEW-02a | 제목줄32, brand hit32, glyph20, 좌우 여백6~12; 제품명13과 구분선·작품명13 | 미디어 없음은 제품명만. 긴 작품명 우선 truncate, 전체 tooltip | 공통 title bar, 제품 식별/미디어 제목 분리 |
| VIEW-02b | 우측126 고정, 각42×32; glyph16 canvas 안의 9~10 선형 도형, stroke1 | 최소화, 최대화/복원, 닫기. 기본 무테두리, hover/pressed 면, focus 내부선 | CaptionControls/창 SVG. 제목 drag 영역과 비중첩 |
| VIEW-03a | 재생/일시정지 glyph20, hit40, 이전/다음 hit32; 왼쪽 그룹 | 실제 재생 준비 전 disabled, pause glyph는 실제 의도/상태 반영 | transport + application 재생 상태 |
| VIEW-04a | 이전/다음은 막대+삼각형, 회차 경계 disabled, 동일 baseline | 대상 제목 tooltip, 안정 회차 ID와 현재 세션 연결 | episode control + 공통 SVG |
| VIEW-05a | 영상 아래 전폭 seek, track3 radius2, thumb 반경5(hover/drag6), 시각 영역18·hit32 | 미상 duration은 disabled. 드래그 예상값과 완료 seek 구별, focus2 | MediaRail. 좌우6 inset, 시간 단위 adapter |
| VIEW-06a | 현재 / 전체 고정폭 숫자13, 기본 MM:SS·1시간 이상 HH:MM:SS | 미상 각 값 `--:--`, 현재 미상과 duration 미상 독립 표현 | 시간 표시 formatter. 알려지지 않은 현재값을 0으로 확정하지 않음 |
| VIEW-07a | 우측 볼륨 glyph20/hit32 + track3/원형 thumb, 뒤 구분선 | 0~100과 mute 별도; 좁으면 같은 rail을 팝오버로 이동, 클릭 중 유지 | 공통 volume + MediaRail, Home/End/좌우 |
| VIEW-08a | 우측 패널260, heading/작품명/스크롤 목록, 행 최소40; 닫기 glyph16/hit32 | P로 열고 닫기, Esc는 열린 패널만 닫기; 데이터 없음 안내 | EpisodePanel. 제목과 닫기 정렬, 본문만 스크롤 |
| VIEW-08b | 행에 번호·제목, 현재 행은 중성 선택면+amber 좌측선/재생 glyph·문자 | current/connecting/focus/hover 구별, 긴 제목 truncate+tooltip | 공통 선택 행 + 안정 episode ID |
| VIEW-09a | 전체화면 glyph20/hit32; 전체화면에서는 제목줄 숨김·overlay | 더블클릭은 버튼 입력과 분리. Esc는 popup→패널→전체화면 | fullscreen action + 공통 shell |
| VIEW-10a | 하단 overlay는 검정/graphite 면 위 읽을 수 있는 transport | 재생 중 idle2.5초 제안, pause/error/drag/menu/focus는 유지 | application visibility 정책 + presentation pin 상태 |
| VIEW-11a | 돌아가기/분류 탭/광고차단 상태 → 검색 → 작품 목록 → 안내 | 웹 보조 화면. 외부 페이지 상태와 native control 분리 | 브라우저 adapter와 presentation 탐색 shell |
| VIEW-12a | 재생 방식은 보조 메뉴, 활성 매체일 때 접근 가능 | 전환 준비/실패 표시, 이중 소리 금지. 무소스에서 빈 모드 메뉴 없음 | 공통 application route 정책 |
| VIEW-13a | 영상 우하단 skip outlined 버튼 hit40+, 카운트다운/이번만 보기 설명; seek OP/ED overlay | 확인된 annotation만 표시, 후보는 실행 불가. 자동 진행은 SEG 정책 | skip overlay + 구간 read model |
| VIEW-14a | 제작 진입은 제목줄 보조 동작, 캡션 앞 고정 slot, 제목이 양보 | 기능 준비 상태 표시, 미디어/PTS 전달, 시청 복귀 시 위치/일시정지 의도 유지 | 공통 studio 진입 action |

## 공통 상태와 좁은 배치

모든 조작 요소는 기본/hover/pressed/focus/disabled를 갖는다. 선택은 현재 회차/모드처럼 지속 상태에만 사용한다. focus는 선택색에 묻히지 않아야 하며 disabled도 이름·이유를 제공한다. 시안은 대부분 기본/선택 상태만 제공하므로 나머지는 공통 컴포넌트 규격의 의도적 보완이다.

1440/1180은 왼쪽 재생 그룹과 오른쪽 음량/회차/전체화면을 분리한다. 980/760에서 음량 rail은 팝오버로 옮기고 시간 문자는 잘라내지 않는다. 캡션126과 창 최소760을 유지한다. 열린 회차 패널은 영상 작업영역을 줄이며 컨트롤을 덮지 않는다. 전체화면 transport는 같은 glyph·키보드·팝오버 규칙을 재사용한다. DPI 변경 검증은 사용자 보류 상태다.

## 미결정·미검증

실제 화면별 padding·영상/transport 경계·회차 패널 상세 상태의 전 요소 비교는 계속 필요하다. 현재 코드를 계약에 맞췄다고 선언하지 않는다. 시청 read model의 unknown position/duration, 회차 번호/현재/연결 상태 표현은 application 계약과 조율하며 임의 가짜 데이터를 만들지 않는다. 상세 상태 시안이 없는 경우 이 문서의 공통화 선택을 관측된 이미지 사실과 구분해 기록한다.
