---
id: sot:kinekino/pts-cadence
title: 원 PTS 보존형 양자화 cadence 검증
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 원 PTS 보존형 양자화 cadence 검증

2026-10-28 중앙 PM. 원R75의 1/1000 timebase·duration41·PTS간격41/42를 기존 정확 연속성 규칙으로 승인하지 않는다. 별도 live Matroska 관측 프로필을 사용하고 원 PTS/duration을 그대로 보존한다. 기존 MP4와 archive는 cadence=None인 정확 연속성 경로를 유지한다.

공식 배경: [FFmpeg AVStream](https://ffmpeg.org/doxygen/7.0/structAVStream.html)은 avg_frame_rate를 평균값으로, r_frame_rate를 추정 가능 값으로 설명한다. 따라서 두 값은 권위가 아닌 검증 후보 입력이다. [Matroska 규격](https://www.rfc-editor.org/rfc/rfc9559.html)의 tick 시간 표현도 raw duration을 임의 교체할 근거로 사용하지 않는다.

## 입력과 수학적 조건

live Matroska의 고정 probe argv는 기존 frame 관측과 함께 stream의 avg_frame_rate/r_frame_rate를 요청한다. 같은 원 source/tool/process/guard가 이 원JSON을 생성해야 한다. 두 양의 유리수 후보가 정규화 후 같아야 하며 누락·0/0·불일치는 거절한다. 이전 raw에 외부 후보를 덧붙여 과거 성공으로 재계상하지 않는다.

domain ObservedCadence는 rate numerator/denominator만 보유하며 Deserialize/권위 constructor는 없다. 후보rate r/s와 timebase b/c에서 프레임 간 이상적 tick 간격은 P/Q=(s·c)/(r·b)다. checked 정수 연산만 사용한다. 현재 경계는 최대50000관측·후보≤240fps·프레임간격≥1tick·양자화 프로필최소2관측이다.

각 n에 대해 d[n]=PTS[n]-PTS[0]으로 놓고, **하나의 공통 위상** phi가 아래 모든 구간에 동시에 존재해야 한다.

`0 ≤ phi < Q` 및 `d[n]·Q - n·P ≤ phi < (d[n]+1)·Q - n·P`

위상 구간의 교집합이 비면 실패한다. 매 프레임마다 허용 오차를 새로 주지 않는다. PTS는 엄격히 증가해야 하고 각 원 reported duration은 P/Q의 floor 또는 ceil 중 하나여야 한다. 프레임을 삽입/삭제/정렬하거나 duration을 인접 PTS 차이로 덮지 않는다. 이는 한정된 관측 창에서의 cadence 일관성이고 전 영상 CFR·관측 밖 프레임·의미적 품질을 증명하지 않는다. 후보 frame rate만 같아서는 통과하지 않는다.

cadence=None이면 기존 `next_pts == previous_pts + reported_duration` 규칙이 그대로 적용된다. 기존 성공을 느슨한 분기로 이동하거나 archive에 rate 힌트를 사후 주입하지 않는다.

## 생산자·발행 reader 연결

RetainedVideoFrames의 live 생성은 원 raw 관측과 계산한 cadence를 함께 보유하고 frames의 PTS/duration은 변경하지 않는다. TimelineContent/TimelineObservations가 같은 선택 프로필을 검증한다. bundle의 video.cadence와 constructor-private publication snapshot에 후보를 보존한다. 발행 reader는 같은 retained raw를 다시 파싱해 frames/timebase/stream/cadence 모두 원 snapshot과 대조하고 동일 domain 검증을 수행한다. trace의 실제 probe argv도 새 필드 요청을 포함해 재구성한다. 이전 live bundle에 cadence 필드가 없으면 null/None인 기존 정확 프로필로만 해석한다.

rate/phase는 live source의 물리적·provenance 권한을 대신하지 않는다. 원 source hash·case/media/guard·실제 tool/process receipt·origin/PCM/capture·opaque producer 입장·association 정책은 모두 유지한다. runtime/재생·실제9 historical clock·원모델결과 품질은 별도다.

## 필수 검증

원41/42 관측의 성공과 원 값 불변, 기존정확모드의동일입력거절·정확MP4회귀를 구분한다. 누락·중복·누적drift·부적절duration·다른rate·계산overflow, 후보누락/불일치·legacy힌트주입거절을 확인한다. 이어 같은 고정 source로 새 probe/PCM/capture/bundle/publication/reader·SDK association을 실제 실행한다. 시험 통과 전 실제 producer/Verified/GUI/품질 완료를 주장하지 않는다. 원R74/R75실패와 보류를 보존한다.
