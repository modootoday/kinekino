---
id: sot:kinekino/save
title: SAVE 요소별 시각 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# SAVE 요소별 시각 계약

[r152 SourceMediaCard 실행 검토](../../../verification/presentation/save-cycle-review-2026-10-19.md#r152-수정-실행-검토)에서1440/760소스헤더·검정면/transport·긴제목·760Empty·트랙/설정Escape유지를확인했다. 시안의2줄transport/볼륨rail·header치수·격자/호환성안내차이는남는다.

SAVE-01/02 소스 카드 후속: r151에서 source 제목에 Folder SVG가 없고 프리뷰가 카드 바탕 위의 단독 이미지/문자형 transport인 차이를 확인했다. download-refined.png의 소스 헤더(284,143,816,75)3배와 하단(284,568,815,75)3배를 기준으로 공용 SourceMediaCard를 조합한다. 기존 Card의 PANEL/THIN/BORDER/CONTROL radius·padding12, Folder SVG20/제목 strong 한줄·말줄임/원문 hover·gap8을 공유한다. 아래는 MediaPreview::unavailable의 전체 폭 VIDEO_SURFACE·CHROME transport/미상시간/비활성seek·Play/Volume/Fullscreen32다. 소스 제목/asset은 snapshot과DesignSample origin을 따르고 시안의 회차명/23:45를 생성하지 않는다. 기존 viewport 기반 미디어 높이를 유지하며 OBJ/SUB 등 선택좌표가 필요한 preview는 이번 변경에서 교체하지 않는다. 실제1440/760·긴제목·Empty·track scroll/설정/Escape를 재검토한다.

[r151 순환 검토](../../../verification/presentation/save-cycle-review-2026-10-19.md)에서1440 소스/트랙/저장설명과760 트랙반응형행·설정/Escape스크롤유지를확인했다. 전체폭프리뷰/transport·소스헤더·호환성안내카드의시각차이는남는다.

SAVE-05/08/10 후속 설계: 저장 방식은 32 높이 radio 제목과 그 아래 들여쓴 보조 설명을 한 그룹으로 조합한다. 공용 described_choice가 제목·설명·비활성 사유의 외형을 소유하며 저장 의미는 SAVE 전용 inspector가 소유한다. 원본 유지/재인코딩 없는 컨테이너 변경/재인코딩 변환의 차이를 설명한다. 계약 미연결 시 세 항목 모두 비활성·미선택이며 시안의 권장 선택을 실제 정책으로 복사하지 않는다. 출력 필드와 예상 정보는 별도 그룹으로 분리한다. 300 검사기와 760 설정 drawer의 본문 스크롤·고정 footer·Escape 복귀를 실제 검토한다.

[r59 실제 검토](../../../verification/presentation/save-mode-description-review-2026-10-18.md):1440/760 저장 방식 설명·예상 정보, 좁은 설정 내부 scroll/고정 footer/Escape와 Empty 트랙 미확인 표시를 확인했다. 전체 SAVE 및 실제 저장 기능 완료는 아니다.

SAVE-03/04 후속: 공용 TrackSelectionTable의 미확인 상태를 구현한다. 유형 SVG·indeterminate checkbox·비활성 selector·트랙 정보·품질/언어 열을 유지하고 unknown을 없음으로 표시하지 않는다.980 미만에는 정보/언어를 다음 행으로 옮긴다. 선택·새로고침 명령은 SDK 연결 전 비활성과 이유를 유지한다. 기존 inspector Select와 같은 공용 unavailable selector를 조합한다. 실제1440/760·Empty 검토 전 완료로 판정하지 않는다.

[기능 계약](downloads.md), [공통 구성](../shared/screen-composition.md), [확대 근거](../../../verification/presentation/frame-card-projection-save-elements-2026-10-18.md). 기준 download-refined. 논리설계값이다.

| ID | 배치·형태 | 데이터·상태·컴포넌트 |
|---|---|---|
| SAVE-01/02 | heading/설명 → sourcecard(folder/title·영상/transport) → trackcard | 명시적회차선택·sessionhandle, source확인상태/실패, 실제URL비밀표시금지 |
| SAVE-03/04 | trackcard heading/설명·새로고침 → checkbox/유형glyph·종류/선택selector/track정보/품질언어 | 영상/오디오/자막row, unknown/none/available별도. track_id/codec/language/variant안정ID |
| SAVE-05 | inspector 원본저장/컨테이너변경/호환변환 radio+설명 | 라디오hit32, 원본유지·리먹스·재인코딩차이, 묵시열화전환금지 |
| SAVE-06/07 | format·codec·변환크기/품질 | 원본모드에서변환옵션숨김, capability없는selector이유표시 |
| SAVE-08/10 | 예상bytes·이어받기지원 readout | unknown확인필요·추정label, Range/ETag/HLS상태실제근거 |
| SAVE-09 | 저장위치field32+folder/파일명field32 | OUT경로검증/충돌·원본보존, 사용자picker취소기존값유지 |
| SAVE-11 | 만료/재확인안내·소스확인동작 | 정상세션재resolve, 같은미디어/track여부확인, 임의프로그램우회금지 |
| SAVE-12 | 호환성info/warningcard → 하단소스확인secondary/큐추가primary40 | 확인필요항목구체표시·preflight통과필수, 미상트랙0개로처리금지 |
| SAVE-13/14 | queue취소/part처리·완료파일/트랙/길이검증summary | 실제journal/receipt, UI샘플을실제다운로드상태로표시금지 |

검사기300/input32/body13/small11/group16/padding12. 980미만trackrow는유형/선택위에정보/언어상세를다음행, inspectordrawer. 긴회차·경로는truncate/tooltip, 주동작/queue는scroll밖. Empty/미확인/트랙없음/지원안함/만료/진행/실패/완료구분, 선택checkbox와disabled값을색만으로구분하지않는다.

필요컴포넌트 SourceMediaCard/TrackSelectionTable/SaveModeGroup/CompatibilitySummary/DestinationFields. typed sourceverification/variant/tracks/capability/estimate/resume/preflightread계약이필요하다. 기존genericpreview/table은trackselector/호환성/저장방식을대체하지못한다. 정책·미디어다운로드는SDK공유, UI는포트호출/상태렌더만담당한다.
