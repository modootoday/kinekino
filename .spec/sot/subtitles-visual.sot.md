---
id: sot:kinekino/subtitles-visual
title: 자막 요소별 시각 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 자막 요소별 시각 계약

[r139 라벨 면 후속](../../../verification/presentation/comparison-label-surface-review-2026-10-19.md)에서1440/760 LongLabels의 고정 열 폭·전체 행 높이 배경·구분선·drawer scroll/Escape를 확인했다. 아래 r138의 label stretch 미완료를 이 범위에서 보완한다.

[r138 교정 비교 실행 검토](../../../verification/presentation/correction-text-comparison-review-2026-10-19.md)에서1440 행 클릭/Down·다국어 LongLabels,760 drawer scroll/Escape/Empty를 확인했다. label 배경의 전체 열/행 stretch와 실제 제안 명령은 남는다.

## SUB-10b 비교 검사기 후속 설계 — 2026-10-19

2026-10-20 선언형 제품 연결 준비: 신규 `web_ui/studio/subtitles/correction-panel.tsx`는 기존 TextComparison·ProposalSource·ProposalActions와 토큰 CSS를 재사용한다. verification/proposal-review-intent.tsx 기술 fixture는 유지한다. 아래 표는 구현 선행 기록이며 실제 시각 판정은 모두 미검증이다.

| ID/상태 | 참조·형태·구조 | 데이터·검증 한계 |
|---|---|---|
| SUB-10b/product-compare | v2/studio/correction.png, crop(1158,139,410,115), 기존 correction-compare-r138-4x.png를 다시 열어 관측. 둥근 한 프레임·label 면·수평 구분선, 원문/교정안 두 행. 기존 TextComparison의 grid/field-label-width·space2/3·thin border·control radius·body Pretendard 사용 | Rust가 전달한 selectedSourceText/workingText만 표시, 변경 강조/diff·모델 이유를 TSX에서 계산하지 않음. 1440/760·긴 한일문구·focus/disabled 실제 비교 미검증 |
| SUB-10b/product-evidence | 같은 inspector의 근거 영역을 ProposalSource 공통 책임으로 연결. 제목→비교→선택 reference·raw/error→수정 입력→공통 하단 action 순서, 내부 scroll/줄바꿈·축소는 기존 inspector CSS 상속 | canonical cue ID 또는 전체 DocumentTextRef의 Rust 표시값, 원문 SHA/revision/generated raw/import_error. package binding을 품질 승인으로 표현하지 않음. source/proposal/selection 미연결 표시·실제 렌더 미검증 |
| SUB-10b/product-edit | aw-text-input/aw-draft-input·useId label/textarea, ProposalActions의 shared save/accept/refresh 버튼·focus·disabled. 새 색/간격/border/radius 선언 없음 | 사용자 수정 working field, busy/confirmed mutation/dirty/refresh를 Rust projection으로 받음. 명시 저장/채택은 같은 SDK usecase이며 자막 파일/영상 적용을 자동 실행하지 않음. 실제 OS 키보드/IME 및 최소화·최대화·닫기 포함 전체 frame 검증은 별도 |

correction.png의(1158,139,410,115)4배 확대(`runtime/review-runs/correction-compare-r138-4x.png`)에서 하나의 둥근 프레임 안에 원문/교정안 두 행, 좌측 label 면, 우측 본문, 수평 구분선과 변경 문구 amber를 확인했다. 현 `subtitle_inspector::fields(tab3)`는 두 값을 연결하지 않은 단일 Readout이며 시안 구조를 대신하지 못한다. 이 항목은 소스·시안 조사이며 수정본 실제 실행 완료가 아니다.

후속 공용 TextComparison은 stable ID·optional source/proposed 문자열·명시 상태를 받고, 두 행 label 폭FIELD_LABEL_MIN72·내용 유동폭·padding8/12·THIN border/CONTROL radius·body13을 공유한다. 양쪽 원문을 보존하고 줄바꿈하며 label과 본문이 서로 침범하지 않게 한다. 미상 값은 대시/연결 안내로 구분하고 proposal 부재를 원문 복제로 채우지 않는다. 변경 강조는 실제 전달된 source/proposed의 문자 차이만 표시하며 변경 이유·confidence·수락 상태를 생성하지 않는다. 동일 문자열은 변경 없음으로 표시한다. LongLabels·한국어/일본어·Empty 및760 drawer 내부 스크롤을 실제 확인해야 한다.

UI 투영은 선택 stable row_id의 명시 original/correction Text cell만 사용한다. DesignSample은 예시 후보임을 표시하고 Live에 교정 계약이 없으면 후보를 만들지 않는다. 같은 source/row의 시작·끝 TimeUs는 읽기 근거로 표시할 수 있으나 sample의 시간이나 문구를 모델의 실제 설명·추론 완료로 승격하지 않는다. 수락/기각/undo는 application command 연결 전 비활성과 이유를 유지한다. 이 비교 구조가 먼저 구현되고 실제 검토된 뒤 기존 placeholder 필드를 대체한다.

변환 첫 전환의 세로 줄바꿈은 [공용 호스트 후속 검토](../../../verification/presentation/layout-first-frame-review-2026-10-18.md)에서 기본2패스/27줄로 재현하고7패스 안정화를 확인했다. 상한8 적용 후 첫 출력 프레임 회귀와 r95 실제 관측 범위가 있으며, 아래 r94 시점의 미완료 기록을 보완한다.

[r93/r94 변환 비교 검토](../../../verification/presentation/subtitle-conversion-comparison-review-2026-10-18.md):1440 문법/번호·양쪽 선택/Down,760 적층·출력 End/스크롤·설정/Escape·긴 문구/Empty를 확인했다. 실제 문서 dry preview와 첫 전환 폭 안정화는 남아 있다.

SUB 변환 비교 후속: convert.png의 비교영역(263,501,899,328)2×와 첫 행(265,546,442,138)4×를 검토했다. 번호 rail32·본문/시각·아래 문법 monospace11, 선택 중성면+amber선·독립 focus2를 공용 비교 목록으로 구현한다. 두 패널은 stable cue ID를 공유하며980미만은 세로 적층한다. 출력에 correction 교정안 열을 사용하지 않는다. DesignSample만 명시적 WebVTT→SRT fixture 쌍을 사용하며 시간·본문이 대응하는 행만 표시한다. 실제 문서의 source/target syntax·loss preview 계약이 없으면 연결 필요를 표시하고 완료 영수증을 본문 미리보기로 사용하지 않는다. 실제 변환/저장 정책은 기존 application SDK를 유지한다.

SUB-08a 후속: ASR/교정의 모든 열 균등 분배를 번호48·시작/끝132·화자80 고정과 원문/교정안 유동 열로 바꾼다. 고정 합은 공통65% 제한을 따른다. 번호는 공용 data_table의 UI projection으로 표시 순서만 부여하고 stable row_id는 유지한다. OCR도 같은 번호 projection을 사용한다. Time kind는 monospace로 표시해 숫자 기준선을 맞추고 원문은 원래 글꼴/전체 hover를 유지한다. 실제 다중선택 계약 없는 checkbox는 추가하지 않는다.

[r63 실제 검토](../../../verification/presentation/subtitle-table-columns-review-2026-10-18.md)에서1440/760 ASR 번호·전체 시간·Down/파형 동기화,760 OCR Home·원문 검사기·Escape를 확인했다. checkbox/다중행/실제 편집과 미검토 상태는 별도다.

SUB-09a 후속: subtitles 시안의 도구 영역(280,674,844,54)을3배 확대했다. 분할/병합/추가/삭제는 icon+label이 같은 테두리 버튼 안에 놓이고 구분선 뒤 undo/redo가 이어진다. 현재 아이콘 버튼과 별도 label 조합, 이미지 glyph를 쓰는 병합, 지우개 삭제, 단순 뒤로 화살표 undo 및 누락 redo를 수정한다. 직접 SVG Merge/Delete/Undo/Redo와 공용 toolbar::unavailable_grouped를 조합하고 높이32·icon20·간격8/그룹16·얇은 border·control radius를 따른다. 실제 history 명령이 없으므로 모두 비활성/이유를 유지한다.

[r62 실제 검토](../../../verification/presentation/subtitle-toolbar-review-2026-10-18.md):1440 ASR·760 ASR/교정에서 여섯 버튼/그룹 구분선과 설정/Escape 복귀를 확인했다. 실제 명령·활성 상태 및 긴 번역 wrap은 별도다.

SUB-03b 후속(2026-10-18): r60 실제 ASR에서 검정 직사각 파형과 정규화된 재생선만 표시되고 시간 눈금·선택 구간·양끝 grip이 누락됨을 확인했다. 공용 waveform을 duration/position/optional selection의 정수 µs 입력으로 확장하고, 같은 timeline_ruler::x 변환으로 파형·아래쪽 눈금·선택면·grip·독립 playhead를 정렬한다. 파형 면은 FIELD·얇은 border·control radius, 선택은 amber 반투명·폭4 세로 grip, body56/ruler32 토큰이다. 선택은 클릭/키보드로 선택된 stable cue의 start/end TimeUs에서만 투영하며 임의 구간을 만들지 않는다. amplitude는 명시적 DesignSample을 유지한다. 교정의 선택 구간 확대 timebase는 별도 후속이며 이번 전체 파형과 동일하다고 표시하지 않는다.

[r61 실제 검토](../../../verification/presentation/subtitle-waveform-review-2026-10-18.md)에서1440 클릭/Down·760 Home의 cue/선택면 동기화와 설정/Escape,760 교정 Empty를 확인했다. 확대·실제 오디오·전용 cue 표·transport는 남은 범위다.

2026-10-18 SUB-05a/06a 후속: ocr-refined.png 결과영역(272,654,865,245)을2배 확대했다. OCR에서는 일반 구간 timeline과 cue 분할/병합 toolbar를 제거하고 감지된 텍스트/낮은 신뢰도 탭 → 번호·시작·끝·감지문·후보·검토 표를 배치한다. typed TableSnapshot의 row_id/start/end/original/correction을 투영하고 화자열은 OCR에서 제외한다. 번호는 표시 순서이고 별도 cue identity가 아니다. 검토/신뢰도 데이터가 없으면 미상·비활성 사유를 표시한다. 선택 row_id를 검사기 원문/후보 read-only 필드와 공유하고 편집 명령 미연결을 명시한다. 읽기 기능이며 영상 글자를 제거하지 않는다는 안내는 공용 notice로 표시한다. 실제 UI 검토 후 범위별 판정을 남긴다.

[자막 기능 계약](subtitles.md)의 ID와 [공통 구성](../shared/screen-composition.md)을 적용한다. 이미지 근거는 subtitles/ocr-refined/correction/convert이고 [확대 관측](../../../verification/presentation/subtitle-elements-2026-10-18.md)에 추출 영역을 기록한다. 치수는 공통 논리 단위이며 원본 raster 측정값이 아니다. 화자 분리는 승인된 기능 계약을 따르지만 전용 이미지가 없어 이미지 일치 판정을 유보한다.

## 공통 요소

| ID | 형태·배치 | 데이터·상태·컴포넌트 |
|---|---|---|
| SUB-01a | 제목/설명 아래 음성 인식·화자 분리·화면 OCR·교정·변환 탭, 높이32, amber 밑줄·선택 문자 | 안정 subview ID. 숫자 탭 index로 서로 다른 데이터 문서를 공유하지 않음. 공통 TabBar/Flex wrap |
| SUB-02a | 검사기 첫 그룹 소스 종류·경로/트랙과 선택 버튼 | 텍스트/이미지 트랙/burn-in/ASR 구별, 입력 미상과 없음 구별. SourceSelector |
| SUB-03a | 음성 검사기에 트랙·채널·인식 언어·번역·범위, label와 입력 정렬 | 번역은 별도 선택, 전체/선택 radio, 선택 구간 in/out ms 표시. typed source/audio fields |
| SUB-04a | 모델 선택과 준비 설명, 검사기 그룹 구분선 | 미선정·미설치·준비·실행·실패 구별. 지원 없는 VAD/정렬 옵션 숨김 또는 이유 표시 |
| SUB-07a | 주요 인식/OCR/제안 생성 hit40; running은 단계·취소 접근 경로 | 수동 cue 편집은 모델 불필요. unsupported command 가짜 실행 금지, 이전 결과·새 초안 revision 구별 |
| SUB-08a | cue 표 header28/row 최소32, 선택 checkbox, 번호·시작·끝·화자·원문·교정안/후보·검토 | cue_id 선택, 시간열 고정폭 monospace, 본문 유동 폭·다국어 wrap, 원문/수정 별도. 선택 행 중성면+amber선, focus 별도 |
| SUB-09a | 파형 아래 분할/병합/추가/삭제, 구분선, undo/redo icon+label hit32 | 가용성은 실제 선택/문서 history 기준, disabled 이유. 좁으면 toolbar wrap, 드래그 대안 명명 동작 |
| SUB-11a | 감지 결과/낮은 신뢰도·겹침/빈 cue 등의 검토 filter | 모델 점수 없으면 확률 숫자 금지. 예시/실측 표시 분리, 선택 cue를 미리보기·검사기에 동기화 |
| SUB-12a | 변환 검사기에 offset numeric·단위·파일 적용 checkbox | 정수ms. 보기 offset과 output 적용 별도, 음수/범위초과 검토 표시 |
| SUB-13a | 전체/선택 범위 radio, 선택 시 in/out 입력 | [in,out), µs 저장/ms 표시. 0기준 재배치 여부 명시 |
| SUB-14a | 형식 selector·UTF-8·원본 보존 안내·저장 위치/이름 | 실제 포맷 능력과 선택값 분리. 원본 불변, 지원하지 않는 옵션을 샘플처럼 활성화 금지 |
| SUB-15a | 중앙 비교 아래 amber 정보 손실 banner·구체적 항목, 검사기에 수락 | loss code→설명, 실제 분석 결과에 한함. 옵션/입력 변경 시 수락 초기화, 비밀 경로/원문 진단 유출 금지 |
| SUB-16a | 하단 내보내기 primary와 검토 상태 | OUT preflight/충돌 정책 공유, 변환 전 타이밍 검사. 완료 영수증은 내용 미리보기 대체물이 아님 |

## 하위 화면 전용 구조

| 화면/요소 | 중앙·검사기 구성 | 재현해야 할 디테일 |
|---|---|---|
| ASR / SUB-03b | 영상+자막 overlay/transport → 전체 파형·시간 ruler → cue 도구 → cue 표 | 파형은 실제 amplitude 또는 명시된 DesignSample 데이터, 구간 양끝 handle·반투명 선택면·독립 playhead. 일반 블록 타임라인으로 대체 금지 |
| OCR / SUB-05a | 원본 영상 위 ROI → transport → 감지/낮은 신뢰도 탭·표 | amber ROI선·흰 원형 corner handle, 원본 좌표 역변환, 회전/contain 반영. A2D blue bounds와 의미 구분 |
| OCR / SUB-06a | 언어·ROI 지정·시간 범위·표본 간격·검토·모델·실행 → 텍스트 편집 | 선택 감지문 원본/수정 필드, focus선, '읽기 기능이며 글자 제거 아님' 안내. 시안의 임의 문자수 상한을 정책으로 복사 금지 |
| 교정 / SUB-10a | 영상/transport → 선택 구간 확대 파형 → cue 표 | 선택 cue의 원문·제안 diff, playhead와 시간 ruler. 전체 길이 파형과 확대 파형 구별 |
| 교정 / SUB-10b | 검사기 원문/교정안 두 행 비교 → source/PTS 근거 → 수락/기각/undo → 옵션·사전·모델·적용범위 | 원문 보존, 변경 부분 amber+문자표현, 근거 설명 wrap, 고유명사 파일 picker. 제안 없음·생성 중·오래된 제안 구분 |
| 변환 / SUB-14b | 영상/transport → 원본 형식 패널 `|` 출력 형식 패널 → 손실 banner | 패널 heading에 형식·cue 수, 각 cue는 번호·시간·읽는 텍스트·원래 문법 monospace. 양쪽 동일 cue 선택, 시간 표기 형식 차이 보존 |
| 변환 / SUB-16b | 검사기 소스/언어 → 출력 형식/인코딩 → 범위/offset → 원본 보존 → 저장 위치/실행 | 폴더는 직접 SVG, 경로 필드 minwidth0/tooltip, 실행 hit40. 변환 결과 내용을 읽기 전에 '저장 완료' 경로만으로 비교를 채우지 않음 |

## 화자 분리 — 이미지 미제공

| ID | 계약으로 정한 구조 | 미검증 범위 |
|---|---|---|
| SUB-17/18 | 화자 목록 `|` 영상·파형·화자 track·cue 표 `|` 모델/인원/범위 | 별도 전용 시안 없음. 모델 auto/fixed/min-max 능력별 입력 차이 실제 검증 전 |
| SUB-19/20 | 목록 row 라벨·발화 구간, track 색+문자·overlap/unknown 표식 | 임시 라벨과 인물명 자동 동일시 금지, 화자 색/목록 너비 상세 시각 인수 전 |
| SUB-21/22 | cue/구간/다중 선택 배정, 미지정·겹침·경계 검토 filter | split/merge/undo 실제 command 연결 전 편집 성공 표시 금지 |
| SUB-23/24 | 출력 라벨 정책·손실·분석 단계/취소/재시도 | ASR와 별도 의존 작업 표시, sample fixture를 실추론 완료로 표시 금지 |

## 크기·키보드·상태

OCR 결과의 번호48·시간132×2·상태88는 sizing 토큰, 감지문/후보는 유동2열로 둔다. 고정열 합은 가용 폭의65%를 넘으면 비례 축소하며 창 자체를 확장하지 않는다. 기존 모든 열 균등 분배로 번호가 과도하게 넓고 원문이 잘리는 차이를 줄인다.

OCR 결과 행은 공용 data_table을 사용한다. 클릭 시 해당 stable row의 첫 cell로 focus를 옮기고 위/아래/Home/End 선택을 검사기의 읽기전용 원문/후보와 공유한다. row 최소32·선택 중성면/amber선·focus 외곽선을 구분한다. confidence 필터는 데이터 미연결로 비활성이며 다중선택 checkbox/실제 편집·OCR 실행은 후속이다.

검사기300, 내부 padding12/그룹16/필드8, 일반 입력32·주요 실행40, 본문13·보조11·Pretendard다. 1280 미만 검사기는 서랍, 1180 미만 탐색48, 980 미만 변환 비교는 위아래로 전환한다. 두 패널은 각각 세로 스크롤/동일 cue 선택을 유지한다. 원문 문법은 수평 스크롤을 허용하되 window 폭을 늘리지 않는다. 긴 한국어/일본어·다중행·emoji/조합은 텍스트 삭제 없이 처리한다.

Tab 순서는 탭→소스→미리보기/범위→도구→cue 표→검사기→실행→큐다. 표 Up/Down/Home/End 선택, Enter 편집/재생의 의미는 명명 동작과 일치해야 한다. IME 조합 중 Space/화살표/단축키를 가로채지 않는다. Ctrl+Z는 필드 편집 우선, Esc는 popup→설정 drawer→제작 이탈이다.

Empty는 입력 안내와 화면 전용 골격, Populated는 cue·파형·문법 비교, LongLabels는 경로/다중행/다국어, NeedsInput은 트랙/모델/손실 수락 원인을 보여준다. Running/Failed/Cancelled/Completed/이전 revision은 application 상태를 따른다. cue 없음(무음 결과)과 실행 실패를 구분한다.

## SDK 조율 항목

OCR의 수동 영역 preview는 `selection_frame::paint_ocr_region`으로 그린다. `color::OCR_REGION` 황색 테두리와 `color::REGION_HANDLE` 흰색 네 원형 corner, `size::REGION_HANDLE_RADIUS` 반지름4를 사용한다. 이 치수는 논리 설계값이며 raster 측정값이 아니다. A2D의 파란 8개 사각 handle과 pivot은 공유하지 않는다. 실제 ROI read model/편집 명령이 없을 때는 DesignSample에 한정한 수동 장식이며 resize cursor나 편집 성공을 제공하지 않는다.

현재 workspace 단위 일반 snapshot만으로 하위 화면을 구분하지 않는다. 안정 subview ID, cue 원문/수정/문법 표현·format·evidence·speaker link, 파형 min/max 샘플·timebase, ROI 원본좌표·source transform, 교정 diff/근거/상태, 변환 dry preview/loss/출력 능력이 필요하다. application이 정책·형식 변환을 소유하고 presentation은 표시·선택·확대만 소유한다. 기존 실제 변환 command/영수증은 보존하며 read preview 계약과 분리한다. SDK 동결 중에는 application 타입을 presentation에 복제하거나 별도 파서를 만들지 않는다.
