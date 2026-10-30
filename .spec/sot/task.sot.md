---
id: sot:kinekino/task
title: 제작·분석 task의 통합 입출력과 인수
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 제작·분석 task의 통합 입출력과 인수

2026-10-26 설계 정규화. application/domain/API/CLI 소유 SDK, 원 표본·모델/미디어 계약 Media, 표시/입력 UI. 기능별 기존 정본을 대체하거나 새 executor 지원을 주장하지 않는다. 이 문서는 R15/R17~R31의 공통 의존성과 기능별 인수 조건을 소유한다. 기존 모듈과 typed port를 재사용하고 같은 기능의 두 번째 정책 구현을 만들지 않는다.

## 실행 경로와 공통 데이터

수명이 짧은 inspect/변환은 기존 usecase, 장시간 작업은 기존 JobService→PreparedJobInput→JobExecutor/OwnedAttemptExecutor→검증된 output 경로다. src/infrastructure/jobs.rs의 기본 executor는 SubtitleJobExecutor이며 start_*_with_executor가 명시 주입을 허용한다. 작업 이름을 COMMANDS에 추가하는 것만으로 executor가 등록되지 않는다. download의 직접 execute Unsupported는 원 owned lifecycle을 요구하는 제한이며 기능 전체 부재가 아니다. media-jobs의 ExecutorRegistry·원 attempt scope를 재사용한다.

모든 작업 입력은 operation/schema_version, source identity+revision, 선택 track/시간 범위, 정규화 parameter, 의존 artifact hash/revision, 모델·도구/프리셋 pin(필요 시), 출력의 사용자 선택/권한을 고정한다. prepared descriptor hash는 실제 원미디어 bytes hash와 별개다. raw URL/credential/셸/임의 executable/filtergraph는 persisted task payload가 아니다. trusted adapter가 수명 있는 lease와 검증된 프로그램/argv를 공급한다. 임의 path를 새로운 media registry로 등록하지 않는다.

출력은 artifact ID, bytes/hash/media_type, source/입력·모델·도구·preset pin, 실제 측정 metadata, 검증/경고, publication receipt를 가진다. source timestamp·engine playback position·decoded PCM sample/프레임 인덱스는 타입/단위·mapping evidence 없이 서로 바꾸지 않는다. output status=Completed는 구체 validator 통과와 publication 이후에만 사용한다. 검토가 필요한 모델 출력은 NeedsReview/현 도메인의 대응 상태이며 자동 accept/usable이 아니다. 지원하지 않는 외부 state enum을 기존 wire에 몰래 추가하지 않는다.

## task별 입력·출력·지원 gate

| task / 요구 | 원 입력·출력 | 검증과 실패/취소/복구 인수 |
|---|---|---|
| 자막 convert/export_range · R17 | source SRT/VTT bytes·format·구간·loss 동의 → 새 자막+loss report | strict UTF8/한도·cue 범위·speaker/style 손실/재동의·충돌/원본불변. 기존 파일 CLI 근거와 새 후계 재검증을 구분 |
| VAD · R18 | 원 WAV SHA+PCM16 mono16k·release manifest pin·profile/admission → 원 samples에 묶인 speech spans | 기존 vad.release/1/ProfileLookup/원 AnalysisControl·owned attempt 연결. release bytes≠private profile snapshot, pin/권리/admission 없으면 NeedsInput. 정상/중간취소/중단복구/worker 종료 별도 |
| ASR · R18 | 검증 audio track·sample mapping·언어·chunk/VAD/model pin → immutable transcript/cue draft+오류/근거 | 원본 offset/청크 overlap 복원·무음/노래/겹침·언어·모델없는 confidence 금지. 취소 checkpoint와 완료chunk 부분결과는 명시적으로만 재사용 |
| diarization/alignment · R19 | audio spans+segmentation/embedding refs, optional ASR/word alignment revisions → speaker spans/정렬 revision | 청크 간 speaker ID 연결·동시발화 원본 보존·auto/fixed/bounded capability. 텍스트 교정으로 타임스탬프 생성 금지. 수동 label/merge/split은 모델 없이 별도 draft edit |
| OCR · R20 | image subtitle track 또는 source frame/PTS·ROI·표본간격·언어/model pin → text/cue 후보·표본경계 오차 | ROI 원본 회전/좌표·dedup/장면컷·이미지/burn-in 구분, 읽기≠픽셀제거. 낮은 신뢰·취소/불완전span/reopen을 표시 |
| 교정/요약/문구/highlights · R21 | 원 cue/text/hash·task schema·model/tokenizer pin·허용 context → raw와 typed 후보를 별도 보존 | invalid structure/누락cue/hallucination·truncation·근거 없는 시간/확률 금지. user repair/accept와 모델 raw 불변. 실제9 NeedsRepair를 성공 fixture로 바꾸지 않음 |
| 의미 검색 · R24 | source/text/cue revisions·embedding model pin·index schema → ranked refs+score semantics | index key에 model/precision/source revision 포함. 원문 삭제/변경 후 stale result 거절·필터/빈결과·재색인/cancel. score를 정확도 확률로 표시하지 않음 |
| OP/ED detect/music · R15 | 검증 audio/video/time mapping·task별 model/정책 → 원본 범위 후보+근거 | 자동 추천과 수동 skip/override/ACK/consume 독립. episode/session 재사용·stale/사전/진행취소·원 issuer 거절. generic seek_submission=false 임의 해제 금지 |
| 다운로드 · R25 | verified source/track/lease·mode(original/remux/transcode)·max bytes·출력 → 검증 media+provenance | HTML/blob 저장 거절·Range+ETag 동일성·HLS VOD init/order/discontinuity·만료 후 재해석 동일성. DRM/live/encrypted는 별도 supported gate. part→검증→no-overwrite publication, restart/충돌/disk-full/cancel·owned writer 종료 |
| capture/snapshot/thumbnail · R26 | local/native session source·관측PTS 또는 fixed PNG SHA·pixel crop → 원본/편집 artifact | capture 실제 PTS/exactness·rotation/HDR/color/subtitle 포함 상태. UI/OSD 제외·frame source hash 연속. PNG crop 인수로 text/mask/resize·전체 편집 완료 대체 금지 |
| accepted Card PNG · R27 | exact OutputReviewTarget·position·fixed PNG·template id/version/hash → review-artifact-only PNG | 기존 card_export/renderer/ProposalStore 재사용. available_unverified·context 취소/timeout 미지원 유지. 글꼴/glyph/가독성·사용자 품질·게시 승인 별도 |
| 전체 카드 편집/묶음 · R27 | CardDocument revision·순서/문구/프레임refs·canvas preset → 순번PNG+문구/alt 초안+editable project | add/duplicate/delete/reorder/undo·overflow·한글 linebreak·비율 변경 crop·batch 부분실패/재시도. accepted 단일PNG와 전체 export_cards executor 구별 |
| 숏폼 manual/recommend/batch · R28 | 공통 clip project·구간배열·track/crop/audio/subtitles/preset → media+편집 project+summary | 수동/추천 동일 renderer·원본 cue 근거·exact vs keyframe cut·VFR/rotate/A/V sync·개별실패 retry/중복방지. 생성·인코딩 취소/미완료 결과·재시작/저장·새파일검증 |
| subject detect/segment/track · R29 | frame/시간범위·click/box/점/수정mask revision·model → mask/track revision | detection class≠사용자 이름/캐릭터 신원. 회전/원본pixel/PTS·가림/컷/소실·ID 지속성·undo·다른 프로젝트 revision 재사용. 실패 프레임을 전체 성공으로 숨기지 않음 |
| 2D asset · R30 | reviewed mask/track·RGB·pivot/bounds/trim/scale/color/alpha → RGBA/mask/sequence+manifest | source UV/offset·straight/premultiplied·checker/합성정렬·sequence PTS/order. unreviewed mask는 명시 입력/경고, 원본 파괴 금지 |
| depth/2.5D · R30 | RGB/mask/model pin·relative depth·layer/plane/camera parameters → depth+layers+projection draft | relative≠metric, near/far finite와 양수순서·depth 방향·temporal 흔들림/edge tear·홀을 미관측으로 표시. unseen 생성은 별도 origin/연구 |
| projection package · R30 | 검토된 RGB/depth/mask와 명시 coord/matrix/UV/color/alpha/unit → 파일묶음 또는 admitted GLB | manifest schema·source PTS·행렬 row/column·handedness·up/forward·depth scale 검증. 대상 import/축/UV/alpha round-trip 전 호환 성공 아님. 완전mesh/rig 생성은 기존 별도 연구 범위 |
| 결과/프리셋 · R31 | artifact/project revision·선택encoder/template/preset pin → local output/receipt | 실제 encoder/권리·지원container/track·길이/probe/open·파일충돌·부분결과. 플랫폼 규격은 versioned data, 현 local export를 외부 업로드 성공으로 바꾸지 않음 |

## 기능별 인수 공통 행렬

TI01 원 입력 hash/revision/track/model/preset 동일성, TI02 wrong/stale/foreign input 거절, TI03 unsupported/missing model/tool/권리의 no execution, TI04 strict schema/한도·malformed output, TI05 사전/실행 중/검증 중/publish 전 취소와 실제 worker cleanup, TI06 publish 뒤 ACK 유실·같은 요청 조회, TI07 disk/permission/conflict·no overwrite, TI08 프로세스 중단→Interrupted/부분결과 분리·수동 재시도, TI09 batch 성공 보존/실패만 retry, TI10 reopen/GUI-CLI 동일 usecase, TI11 실제 content/time mapping·품질/자원 관측, TI12 original/raw/현재 actual9 불변. 각 operation은 실제 적용 가능한 cell과 N/A 이유를 명시한다. 취소/timeout이 미지원인 기존 동기 export는 supported로 위장하지 않고 별도 확장 backlog에 둔다.

## 자원·모델 admission

ModelManifest의 immutable revision/files hash/license/format/precision/tokenizer/config/engine pin을 task별로 고정한다. 기존 결정된 모델을 임의 교체하지 않는다. model/task 미선정은 공백을 숨길 ‘자동 최신’이 아니라 NeedsInput이며 전체 요구는 남는다. installed≠verified≠admitted≠loaded≠quality_approved. install은 staging→파일검증→immutable installed revision, 삭제는 in-use lease 존재 시 거절/명시 대기, 실패/취소 partial을 가용 모델로 노출하지 않는다. offline 사용은 closure가 모두 있을 때만, gated 동의/HF token은 기존 OS credential 경계다.

measured_memory/dtype/provider/resource sample은 실제 관측이 없으면 null/미측정. 선언 precision을 실행 dtype으로 승격하지 않는다. GPU/VRAM capacity는 현재 호스트에서 별도 관측 전 과거 RTX8GB값을 재사용하지 않는다. PM heavy1·worker1/기존 CPU2/BelowNormal/jobs1을 유지하고 단순 path 최적화를 CPU 확대 허가로 해석하지 않는다. 새 inference/download/장시간 경쟁시험은 이번 설계로 재개하지 않는다.

연결: [제작 정본](sot-012-media-production.md), [AI 진입점](../analysis/sot-013-local-ai-projection-assets.md), [수명·저장](../execution-storage-boundaries.md), [host 표면](../api/host-support-boundaries.md), [실행 순서](../../../../plans/player/design-completion/implementation-order.md).
