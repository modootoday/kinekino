---
id: sot:kinekino/speaker-pipeline
title: 화자 분리·자막 정렬 유스케이스
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 화자 분리·자막 정렬 유스케이스

2026-10-27 부분 구현 갱신: 공통 SDK/정상 file CLI의 수동 화자 JSON 조회·편집은 구현·검증했다. 자동 diarization/align/ASR·미디어 바인딩·GUI·품질 인수는 남는다. [현재 수동 계약](manual-speaker-annotations.md). 기존 전체 파이프라인 요구와 담당 소유권은 유지한다.

상태: 전체 추론/정렬 파이프라인은 설계·미구현, 수동 조회/편집은 위 계약 범위에서 부분 구현·검증. [도메인 계약](../../domain/subtitles/speaker-contract.md), [공통 SDK](../api/sdk-contract.md)를 사용한다.

`diarize_audio`는 오디오 트랙/구간·화자 수 auto/fixed/bounded·segmentation/embedding/군집 설정·모델 refs를 고정한다. `align_speakers`는 기존 ASR/단어 정렬과 speaker spans를 원본 시간축으로 합친다. `edit_speakers`는 이름/merge/split/선택 cue 배정, `export_subtitles`는 손실 검사·format/sidecar를 담당한다. ASR, 정렬, 화자 분리, 텍스트 교정은 분리된 작업이며 필요한 결과만 재실행한다.

파이프라인: 트랙 검사→PCM 전처리→발화/겹침 segmentation→speaker embeddings→clustering→원본 시간축 복원/청크 경계 군집 연결→검토 가능한 DiarizationRevision→ASR cue/단어와 정렬. 청크별 speaker1을 단순 이어 붙이지 않는다. confidence가 없는 backend는 임의 확률을 표시하지 않는다. 동시발화 원본과 편의상 exclusive label 표시는 별개 산출물로 둔다.

화자 수 기본 auto 제안. fixed/min/max는 선택 모드에서만 입력하고 1≤min≤max를 검증한다. 최대 지원 수·오디오 청크 크기·overlap 설정은 선택 모델 capability에 따른다. 모델이 인원 수 제약을 지원하지 않으면 옵션을 비활성하고 이유를 표시한다. 인원 수 자체는 임의 제한값을 모델 지원으로 오인하지 않도록 schema에 capability 범위를 노출한다.

HF에서 task `speaker_segmentation`, `speaker_embedding`을 ASR/text와 구별해 immutable revision·전처리·형식·정밀도·라이선스·런타임 지원을 검증한다. 추론 어댑터는 Rust SDK/검토한 네이티브 FFI이며 Python 실행 환경을 제품 필수 의존성으로 추가하지 않는다. 모델을 동시에 상주시키지 않고 기존 재생 우선·무거운 워커1개·청크/배치 축소·유휴 unload 예산을 적용한다. INT8/FP16 등 지원 양자화별 DER/JER·메모리·RTF를 평가하고 일괄 Q4를 강제하지 않는다.

모델/트랙 없음은 NeedsInput, 취소는 소유 작업만, 변경된 revision 결과는 이전 초안으로 표시한다. 수동 화자 라벨 편집은 모델 없이 가능하다. `subtitles.diarize`, `subtitles.align_speakers`, `subtitles.edit_speakers` 명령을 GUI/CLI에서 동일하게 사용한다. ASR→화자 분석 결합 실행은 명시한 의존 작업으로 표시한다.

미검증: Windows Rust 엔진과 양자화 모델 선정, 한/일 애니메이션 대사·음악/동시발화 품질, 청크 군집 일관성, CPU/RTX 실제 자원·재생 경쟁, export 플레이어 호환. 기존 Qwen 텍스트 CPU 실측을 화자 분리 성능으로 대체하지 않는다.
