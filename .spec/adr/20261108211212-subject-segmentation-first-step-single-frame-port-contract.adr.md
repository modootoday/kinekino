---
id: adr:kinekino/20261108211212
title: "Subject segmentation first step: single-frame port contract"
status: active
created: 2026-11-08T21:12:12Z
updated: 2026-11-08T21:13:02Z
rel:
  governs: [crates/kinekino-inference/src/executor, crates/kinekino-inference/src/port]
  depends_on: [adr:kinekino/20261029113535]
---

# Subject segmentation first step: single-frame port contract

## Context

로드맵 plan:kinekino/20261031153240의 M4-10(SB-5, 피사체 분할·투영 패키지)이다. M4 완료 기준은 피사체 분할을 **첫 설계까지만** 완료 범위로 두며, 실제 추론 호출·과금·모델 다운로드·실모델 실측은 별도 승인 전 금지한다.

코드 현황(2026-11-09 관측):

- 추론 포트(`crates/kinekino-inference/src/port`)에 `Task::SegmentSubject`(`segment-subject`), `InferenceRequest::SegmentSubject { model, image }`, `Output::SegmentSubject { mask_png }`가 있다. 이 작업을 부르는 executor는 없다.
- 목 백엔드는 `SegmentSubject`에 PNG 서명 4바이트만 돌려준다. 디코드 가능한 마스크가 아니다.
- HF 어댑터는 `SegmentSubject`를 요청 단계에서 만들지 않고 응답 단계에서 `Unsupported`로 거절한다.
- 명령 목록의 `production.segment_subject`는 지원 판정 분기가 없어 항상 unsupported다.
- 플레이어 읽기 모델 `workspace_read::MaskSnapshot`은 원본 자산·원본 크기·`source_pts_us`와 `SubjectMask { track_id, polygon_px, alpha_asset, review_state }`를 갖는다. `alpha_asset`은 원본 캔버스 전체 크기의 알파이며 잘라낸 피사체 이미지가 아니다(sot:kinekino/obj).
- 프레임 입력은 기존 `production.capture_frame`(M4-4·M4-6, 원격 입력 거절, 회전은 보고만)이 만드는 로컬 PNG를 쓸 수 있다.

권고안(루틴 진행, 2026-11-09): 첫 단계는 **단일 프레임·단일 마스크** 계약만 정한다. 추적(track 연결), 구간 분할, 포함/제외 점·브러시 보정, 투영 패키지는 후속으로 둔다. 이 문서는 설계이며 구현·실측·품질 승인이 아니다.

skill: modoo skills에서 `subject-cutout-alpha-matte-check`를 찾아 설명을 참고했다. 실제 알파 채널 여부, 마스크 극성, 프레임 열의 개수·순서 검사를 아래 응답 검증 항목에 반영했다. 합성 배경별 가장자리 검사는 사용자 GUI 검토 범위로 남긴다.

## Decision

### 1. 유스케이스 경계

`kinekino-inference`의 executor에 단일 프레임 분할 유스케이스 하나를 둔다(가칭 `segment_frame`). OCR executor(`ocr_frames`)와 같은 단계 도우미(`step::consent`, `step::call`)와 `CostGuard`·`Consent`를 재사용한다. 호스트 코드에는 사이트·작품 용어를 쓰지 않는다.

입력:

| 필드 | 규칙 |
|---|---|
| `model` | 비어 있지 않은 모델 id. 레지스트리가 `SegmentSubject` 능력을 보고하지 않으면 호출 전 `Unsupported` |
| `frame.png` | 로컬 PNG 바이트 하나. 디코드 가능한 PNG가 아니거나 폭·높이가 0이면 호출 전 `InvalidInput` |
| `frame.width_px`, `frame.height_px` | PNG 헤더에서 읽은 값. 호출자가 따로 넘긴 값과 다르면 `InvalidInput` |
| `frame.source_pts_us` | 원본 미디어 시각(마이크로초, 0 이상). 결과의 출처로만 쓰고 백엔드에 보내지 않는다 |
| `cancel` | 호출 전에 확인. 취소면 호출 없이 `Cancelled` |

백엔드로 나가는 것은 프레임 PNG 바이트뿐이다. 영상 전체·경로·URL·원본 파일명은 보내지 않는다(포트 원칙 "cut segment/frame, never a whole video").

### 2. 응답 검증

`Output::SegmentSubject { mask_png }`를 받으면 결과로 인정하기 전에 다음을 확인한다. 하나라도 어기면 `Protocol`로 실패시키고 결과를 게시하지 않는다. 실패한 응답은 과금 기록에 넣지 않는다(M4-8과 같은 규칙).

1. 디코드 가능한 PNG다.
2. 폭·높이가 입력 프레임과 같다. 크기를 바꾸거나 잘라낸 마스크는 받지 않는다(원본 캔버스 전체 알파 원칙).
3. 단일 채널 회색조(8비트) 또는 알파 채널이 있는 형식이다. 알파 없는 RGB는 극성을 알 수 없으므로 거절한다.
4. 극성은 "값이 클수록 피사체"로 고정한다. 반전 여부를 추측해 뒤집지 않는다.
5. 모든 화소가 0이면 실패가 아니라 "피사체 없음" 결과로 돌려준다. 전부 최대값인 마스크도 그대로 돌려주되 자동으로 승인하지 않는다.

출력 task 불일치는 기존 `step::call`이 `Protocol`로 처리한다.

### 3. 결과 형태

결과는 플레이어 읽기 모델로 바로 쓰지 않고 executor 전용 값으로 돌려준다.

- `mask_png`(검증된 원본 크기 알파), `width_px`, `height_px`, `source_pts_us`
- `coverage`: 0보다 큰 화소 비율(0.0~1.0). 품질 점수가 아니며 신뢰도로 표시하지 않는다
- `provenance`(백엔드·모델·비용 추정)와 `Spend`

플레이어로 넘길 때는 `SubjectMask`로 옮기되 `review_state`는 항상 `Unreviewed`, `track_id`는 새 후보 id, `polygon_px`는 `None`으로 둔다. 다각형 추출은 후속 조각이다. 자동 검사 통과를 `Confirmed`로 바꾸지 않는다. 품질 승인은 사용자가 실제 GUI에서 한다.

### 4. 오류 대응

| 상황 | 오류 | 호출 |
|---|---|---|
| 능력 없음·모델 미등록 | `Unsupported` | 0회 |
| 입력 PNG 손상·크기 0·크기 불일치·음수 시각 | `InvalidInput` | 0회 |
| 업로드 동의 없음(원격 백엔드) | `ConsentRequired` | 0회 |
| 비용 한도 확인 필요·거절 | 기존 `CostGuard` 흐름(`NeedsConfirm`/`Budget`) | 0회 |
| 호출 전 취소 | `Cancelled` | 0회 |
| 백엔드 실패(`Unavailable`/`RateLimited`/`Timeout`/`Auth`/`Budget`) | 그대로 전달, 미과금 | 1회 |
| 응답 PNG 손상·크기 불일치·극성 불명 | `Protocol`, 미과금·미게시 | 1회 |

### 5. 명령·CLI 범위

`production.segment_subject`는 이 단계에서 계속 unsupported다. 실제 백엔드(로컬 모델 또는 원격) 연결과 CLI 연결은 추론 승인 뒤 별도 조각에서 한다. 그 조각도 원격 URL 입력을 받지 않고 `production.capture_frame`과 같은 로컬 경로 규칙을 따른다.

### 6. 목 백엔드

다음 구현 조각에서 목 응답을 디코드 가능한 PNG(입력과 같은 크기, 회색조, 일부 화소만 255)로 바꾼다. 현재의 4바이트 응답은 위 검증에서 `Protocol`이 되어야 한다. 목 계약 테스트는 M4-8 OCR 목 계약과 같은 구성(성공, 백엔드 실패, 취소, 동의, 비용 확인, 응답 검증)으로 쓴다.

## Consequences

- 피사체 분할은 단일 프레임 계약까지 설계로 닫혔다. 실제 모델 선택, 모델 다운로드, 실측, 마스크 품질은 승인 전 미정이다. 이 문서는 SB-5를 구현으로 바꾸지 않는다.
- 결과 마스크는 원본 캔버스 전체 크기만 받으므로 읽기 모델의 `alpha_asset` 원칙과 맞는다. 모델이 작은 해상도만 내면 어댑터가 확대할지는 어댑터 조각에서 정한다. executor는 크기 불일치를 받지 않는다.
- `coverage`는 표시용 사실값일 뿐이다. UI에 신뢰도나 품질로 보여주지 않는다(sot:kinekino/seg의 임의 confidence 금지와 같은 원칙).

남은 일(후속 조각, 승인·순서는 큐에서 정한다):

1. executor `segment_frame`과 목 계약 테스트(이 문서의 1~4, 6절).
2. 마스크에서 다각형 추출과 `SubjectMask` 변환.
3. 여러 프레임의 track 연결, 포함/제외 점·브러시 보정, 구간 분할.
4. 실제 백엔드 어댑터와 `production.segment_subject` CLI 연결(추론 승인 뒤).
5. 투영 패키지(SB-5의 투영 부분).
