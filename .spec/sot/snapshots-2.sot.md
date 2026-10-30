---
id: sot:kinekino/snapshots-2
title: 스냅샷과 썸네일
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 스냅샷과 썸네일

추가 계약(snapshot 검증 완료·root 통합 대기): 파일 기반 `CapturedFrame` 성공 응답의 `output_sha256`는 선택적 canonical SHA256다. 신규 producer는 실제 publication bytes의 SHA를 Some으로 반환하고 legacy receipt의 필드 부재는 None으로 유지한다. 이 값으로 기존 `production.export_snapshot.expected_source_sha256`를 채우며 capture의 원 actual PTS receipt를 그대로 연결했다. 요청 schema·명령·native의 frame_time_exact=false는 바꾸지 않았다. [실제 미디어 연속 검증](../../../verification/application/frame-snapshot-shell-checkpoint-2026-10-20.md)의 관측·한계를 따른다.

2026-10-20 구현 인수 범위: 기존 PNG의 고정 SHA·명시 pixel crop→새 PNG 저장은 `production.export_snapshot` typed SDK와 실제 CLI/endpoint로 검증했다. [실행 근거](../../../verification/application/snapshot-export-sdk-cli-2026-10-20.md)를 따른다. 원 PNG와 numeric sample/alpha를 보존하되 ICC/부가 metadata는 복사하지 않는다. 아래 전체 편집·캡처 설계 중 GUI·리사이즈·텍스트·마스크·모델 및 원 미디어 PTS/색공간·영상 연동은 이 인수에 포함하지 않는다.

현재 디코딩 프레임을 고정하고 실제 PTS·해상도·색공간·회전·자막 포함 상태를 기록한다. 원본 PNG와 편집본 JPG/PNG를 분리한다. 저장 시 UI/OSD·광고·브라우저 문서가 포함되지 않아야 한다. [mpv screenshot 명령](https://mpv.io/manual/stable/#screenshot-commands)의 video/subtitles 선택을 명시하며 출력 드라이버·HDR 톤 매핑은 실기 검증한다. 이미 영상에 입혀진 자막은 ‘자막 제외’로 제거할 수 없다.

편집은 16:9/9:16/사용자 비율, 크롭/맞춤, 텍스트·배경 박스·색·여백·정렬을 제공한다. 한글 줄바꿈과 미지원 글꼴 대체를 미리 보여주고 스냅샷 원본을 다시 인코딩하며 손상시키지 않는다. 인식한 피사체 마스크는 배경 분리/전경 강조에 재사용한다. 추천 문구와 자동 레이아웃은 수정 가능한 초안이다.
