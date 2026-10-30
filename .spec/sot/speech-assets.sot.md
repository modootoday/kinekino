---
id: sot:kinekino/speech-assets
title: 일본어 원음 평가 에셋 확보
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 일본어 원음 평가 에셋 확보

## 우선 경로: 공식 FLEURS 원본 archive

기존 관측: `ja_jp` datasets-server row API는 Parquet 크기 제한으로 500을 반환했다. 음원 자체 부재를 뜻하지 않는다. 이번에는 모델·음원을 받지 않고 공식 HF tree의 경로/크기를 확인했다.

고정 revision은 [목록](revisions.md)의 google/fleurs다. 공식 [일본어 원본 디렉토리](https://huggingface.co/datasets/google/fleurs/tree/70bb2e84b976b7e960aa89f1c648e09c59f894dd/data/ja_jp)에 `dev.tsv`, `test.tsv`와 audio archive가 있다. API tree가 보고한 압축 byte 수는 dev 179,387,192, test 448,762,391, train 1,332,680,503이다. 서버 row API 의존성이 없는 dev archive를 첫 시험으로 권한다. dev/test 분할을 바꿨다는 사실과 개별 ID를 결과에 명시한다.

원래 세션이 구현·실행한 절차의 재현/검토 기준(이 조사 세션은 실행하지 않음):

1. 작은 `data/ja_jp/dev.tsv`를 Rust reqwest 어댑터로 고정 SHA에서 받아 실제 열 구조·음원 basename·raw/정규화 정답을 확인한다. 헤더 유무와 열 순서를 추측하지 않는다.
2. 해당 SHA의 `data/ja_jp/audio/dev.tar.gz`를 디스크에 스트리밍 저장한다. tar/flate2 Rust 어댑터로 명명된 첫 표본만 추출한다. 경로 탈출·symlink·중복 덮어쓰기 금지, 압축해제 byte 상한을 둔다. archive 전체를 RAM에 올리지 않는다. gzip stream이므로 단일 WAV에 HTTP Range만으로 직접 접근할 수 있다고 가정하지 않는다.
3. transcript ID ↔ WAV basename을 대조하고 원본 hash·archive hash·PCM sample rate/채널/duration·변환 옵션을 보존한다. 서로 다른 transcript ID 3개를 고정하며 화자 독립성은 보장하지 않는다. 초기 1개와 확대 평가를 구별한다.
4. CC-BY-4.0 출처·제작자 attribution과 원문/변형 구분을 manifest에 남긴다. 번역 자막은 일본어 ASR 정답으로 사용하지 않는다.

명시적 URL 패턴: `https://huggingface.co/datasets/google/fleurs/resolve/70bb2e84b976b7e960aa89f1c648e09c59f894dd/data/ja_jp/dev.tsv` 및 같은 prefix의 `audio/dev.tar.gz`다. 일시 CDN 서명 URL을 정본에 저장하지 않는다.

현재 CLI 재확인: `speech-fixture`는 `ja` 또는 `ko`만 허용한다. 앞선 문서의 `speech-fixture ja_jp`는 잘못된 명령이므로 사용하지 않는다. 원래 세션은 별도 `speech-archive`, `asr-japanese` 명령과 `infrastructure/speech_archive.rs`, `application/japanese_asr.rs`를 추가했다. [실행 계약](execution-readiness.md)의 실제 명령을 따른다.

원래 세션의 [음성 검증 정본](../../../../verification/ai/audio.md)과 [archive manifest](../../../../../../runtime/ai-spike/assets/speech-ja-archive.json)를 참조한다. 재검토 시 manifest에는 dev ID `1516`, `1634`, `1578` 및16.74/12.12/9.0초가 기록되어 있다. 원본은16kHz mono float32이고 Rust hound로 round/clamp PCM16 변환했으며 resampling은 없다. manifest의 speaker_metadata는 성별 정보이며 화자 고유 ID가 아니다. [초기 ASR 집계](../../../../../../runtime/ai-spike/results/asr-japanese.json)와 [원본 길이 제한 후 집계](../../../../../../runtime/ai-spike/results/asr-japanese-bounded.json)는 원래 세션이 생성했다. 정본은 padding 추가구간 반복 제거와 본문 오인식 잔존을 구분한다. 이 조사 세션은 문서·파일을 읽었으며 재실행·독립 CER 계산·품질 승인을 하지 않았다.

## 차선 경로

[공식 FLEURS Parquet 스냅샷](https://huggingface.co/datasets/google/fleurs/tree/fdd74f7cdf01fc7641ab3072b6a38d364b3667f6/ja_jp)은 validation 약216MB, test 약542MB로 표시된다. archive 접근 실패 시 Rust parquet/Arrow로 row group을 제한해 읽는 대안이다. 기존 archive SHA와 다른 스냅샷이며 ID/정답 대응을 확인해야 한다. Parquet audio struct의 실제 schema를 먼저 읽고 전체 decode하지 않는다.

[Mozilla 공식 일본어 Common Voice](https://mozilladatacollective.com/datasets/cmu5xbhnf00fmnq07z9p3aa70)는 두 번째 독립 데이터원 후보다. 현재 배포 조건·로그인·릴리스·음원/문장 라이선스·다운로드 크기 검토 전 자동 수집하지 않는다. FLEURS의 직접 공식 archive가 있으므로 미러나 재배포본을 먼저 선택할 이유는 없다.

FLEURS는 읽기 음성 시험이다. 실제 애니메이션의 BGM/가림 발화/감정/겹침을 대표하지 않는다. 해당 품질은 권리 확보한 별도 원음과 수작업 정답으로 나중에 평가한다. Parquet/Common Voice 대안은 이 조사에서 미다운로드·미청취·미실측이며, 원래 세션의 공식 archive 관측과 구분한다.
