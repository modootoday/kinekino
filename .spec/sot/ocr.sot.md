---
id: sot:kinekino/ocr
title: 자막 음성 인식 화자 분리 화면 OCR 교정 변환
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 자막 음성 인식 화자 분리 화면 OCR 교정 변환

하위 화면별 배치·형태·데이터 연결은 [요소별 시각 계약](subtitles-visual.md)을 적용한다. 공통 표와 미리보기만으로 모든 하위 화면을 대체하지 않는다.

## 시각화

- [음성 인식](../../../../artifacts/design/v2/studio/subtitles.png)
- [화면 OCR](../../../../artifacts/design/v2/studio/ocr-refined.png)
- [교정](../../../../artifacts/design/v2/studio/correction.png)
- [변환](../../../../artifacts/design/v2/studio/convert.png)

이미지는 배치 참고이며 아래 요소 계약과 공통 필드·상태 규칙이 구현 기준이다.

상태: UI/UX 설계 승인(2026-10-18). 생성 이미지 디테일은 실제 Rust 공통 컴포넌트로 검토한다. 구현 완료와 구분한다. [공통 계약](../sot-020-ui-contract.md)의 프레임·입력 검증·상태·접근성·저장·Rust 계층 규칙을 상속한다. 기본/hover/pressed/selected/disabled/focus는 모든 조작 요소에 적용한다.

## 요소 계약

| 요소 ID | 요소 | 표시와 동작 |
|---|---|---|
| SUB-01 | 하위 탭 | 음성 인식/화자 분리/화면 OCR/교정/변환, cue와 화자 revision 연결 |
| SUB-02 | 입력 종류 | 외부 텍스트·내장 텍스트·이미지 트랙·burn-in·ASR 구별 |
| SUB-03 | 오디오 | 트랙·채널·구간, 언어 자동/명시, 번역과 분리 |
| SUB-04 | ASR 모델 | 준비 상태 표시, VAD/청크·정렬 지원 여부 |
| SUB-05 | OCR ROI | 원본 좌표에 사각형, 포함 범위·회전·미리보기 |
| SUB-06 | OCR 언어/표본 | 음성 언어와 별도, 간격과 경계 오차 표시 |
| SUB-07 | 시작/취소 | 모델/트랙 확인 후 실행, 수동 cue 편집은 모델 불필요 |
| SUB-08 | cue 표 | 번호·시작·끝·화자·원문·교정안, 행 재생과 다중 선택 |
| SUB-09 | 편집 | 추가·삭제·분할·병합·줄바꿈·시각 이동, undo/redo |
| SUB-10 | 교정 제안 | 원문 diff·고유명사 사전, 개별/선택 수락과 되돌리기 |
| SUB-11 | 검토 필터 | 겹침·빈 cue·음악·무음·확인 필요, 점수 없는 경우 확률 미표시 |
| SUB-12 | 오프셋 | ms 정수 입력, 미리보기와 파일 적용을 별도로 선택 |
| SUB-13 | 범위 | 전체/선택, [in,out) 잘라 0 기준 재배치 여부 |
| SUB-14 | 출력 | 원본/SRT/VTT/ASS, UTF-8 기본, 원본 보존 |
| SUB-15 | 변환 손실 | 위치·스타일·노래방·화자 정보 손실 목록 확인 |
| SUB-16 | 저장 | 원본/교정본 구별, OUT 계약, 내보내기 전 타이밍 검사 |
| SUB-17 | 화자 분석 선택 | ASR와 결합/기존 음성만 분석, 별도 의존 작업 표시 |
| SUB-18 | 화자 모델·인원 | segmentation/embedding 상태, auto/fixed/min-max와 지원 범위 |
| SUB-19 | 화자 목록 | 로컬 ID·임시 라벨·발화 구간, 이름 변경·병합·분할·undo |
| SUB-20 | 화자 타임라인 | 파형과 행별 발화, 겹침·unknown 구별, 원본/단일 화자 표시 전환 |
| SUB-21 | 배정 편집 | cue/구간/다중 선택 화자 배정, 미지정 되돌리기, 수정 revision |
| SUB-22 | 검토 필터 | 겹친 발화·화자 변경 cue·미지정·청크 경계, 모델 점수 없으면 확률 없음 |
| SUB-23 | 출력 라벨 | SRT 접두어·VTT voice·ASS Name·sidecar, 손실과 호환 범위 설명 |
| SUB-24 | 분석 실행·취소 | Preflight/큐 공유, 실패 재시도·이전 revision 결과·수동 편집 유지 |

## 화자 분리 화면 배치

화자 분리 하위 탭은 공통 제작 프레임 안에서 왼쪽 화자 목록, 중앙 영상/파형·화자별 타임라인·cue 표, 오른쪽 모델·인원 수·분석 범위·표시/출력 설정을 사용한다. 좁은 창에서는 공통 설정 서랍 규칙을 상속한다. 화자별 색과 함께 라벨·겹침 표시·키보드 포커스를 제공한다. 이름 변경 F2·목록 선택·구간 재생·병합/분할은 명명된 버튼/메뉴로도 접근한다. 화자 라벨과 캐릭터 이름을 자동 동일시하지 않는다.

분석 전 모델 준비/입력, 실행 중 처리 단계/취소, 완료 후 unknown/overlap 검토, 수정 충돌/모델 오류를 각각 표시한다. 빈 오디오 결과와 실행 실패는 별개다. 기존 이미지는 화자 하위 탭 추가 전의 배치 참고이며 새 탭의 시각화 검증 완료 근거가 아니다. Pages는 사용자 지시로 더 갱신하지 않는다.

화자 값·변환의 정본은 [화자 도메인](../../domain/subtitles/speaker-contract.md), 작업 계약은 [화자 분석](../../application/analysis/speaker-pipeline.md)이다. CueDocument에 diarization_revision과 cue speaker link를 연결하며 화자 원본 span을 텍스트 문자열에만 저장하지 않는다.

## 상태와 오류

무음은 성공한 빈 결과와 오류를 구별한다. burn-in OCR은 글자 제거가 아니다. 번역·교정이 원문/시간을 몰래 덮어쓰지 않는다. 트랙 없음, 모델 없음, 실패 구간 재시도, cue 수정 충돌을 구별한다. SRT/VTT/ASS는 지원 범위 후보이며 변환 우선순위는 검토로 확정한다.

## 저장 계약

CueDocument(id,source_revision,language,cues[{id,in_us,out_us,text,style_ref,origin,evidence}],corrections,offset_us,export_options). 원본 시간은 보존하고 변환 결과를 별도 출력한다.

## 인수 기준

한글 IME·일본어·VFR·음수/겹침·구간 끝·무음 환각·원문 diff·스타일 손실·형식 왕복.

각 요소 ID의 유스케이스는 application에, 검증 정책/값 객체는 domain에, OS·추론·파일·프로세스는 infrastructure에 분리한다. presentation은 입력과 렌더링을 담당한다. 모델 미선정/encoder·플랫폼 지원 여부 미확정 항목은 완료로 표시하지 않고 해당 기능을 비활성 또는 NeedsInput으로 제공한다.

