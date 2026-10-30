---
id: sot:kinekino/tools
title: tools 기능의 독립 패키지 승격 조사
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# tools 기능의 독립 패키지 승격 조사

상태: **리서치·제안, 승격 미실행**. 2026-10-19 09:24–09:30 KST의 정적 source 관측이다. SDK의 공통 패키지 검사와 AI 엔진 검사는 별도 담당이 진행 중이며 이 조사에서 조작하지 않았다. 코드/Cargo/lock/.gitignore/기존 패키지·문서 수정, 파일 이동, fmt/build/test, 앱 종료, 브라우저·모니터 조작, 모델 추론·다운로드는 하지 않았다. 산출물은 이 정본과 [후속 계획](../../plans/architecture/tools-package-promotion-2026-10-19.md) 두 Markdown이다. 실제 승격은 PM이 각 desktop 담당에게 배정한다.

기준은 [프로젝트 규칙](../../../AGENTS.md), [.spec 규칙](../../AGENTS.md), [기능 경계 조사](package-boundaries-research-2026-10-19.md), [SDK 계약](../player/application/api/sdk-contract.md)이다. architecture·api-patterns 스킬의 단순 대안 비교, 책임·공개 계약·호환성 원칙을 사용했다. 별도 HTTP/서비스를 추가하는 제안이 아니다. 새 코드·시험·도구는 Rust다.

## 관측 범위와 수량

실제 `Get-ChildItem tools -Directory` 결과는 **24개 직계 도구**이며 각 직계 Cargo.toml 24개를 읽었다. 처음 재귀 검색에 추가로 잡힌 qwen-asr-spike/upstream/Cargo.toml은 보존 upstream이므로 25번째 도구가 아니다. 각 src 진입점·주요 application/domain/port와 관련 README를 읽었다. 소유 패키지는 현재 `packages/{meari,media-contracts,media-file-publication,media-subtitles}` 4개다. SDK가 이 3개 신규 package의 실제 검증을 진행하므로 존재를 완료로 해석하지 않는다.

24개 manifest 중 src/lib.rs를 가진 도구는 13개다. pub 모듈이 있다는 것과 안정된 외부 API·제품 연결이 있다는 것은 다르다. 24개 직계 manifest 모두 `license = ...` 필드가 없었다. upstream 라이선스/notice가 따로 보존된 경우에도 로컬 package의 재배포 조건·코드 저작권·모델 허가까지 확정된 것은 아니다. 이 조사는 법적 판정이나 배포 허가가 아니다.

현재 root Cargo는 3개 path package와 Meari 제외 workspace를 사용한다. 제품 root manifest에서 tools path dependency는 확인되지 않았다. 도구 간 실제 직접 의존은 `diarization-{der,jer}-reference → diarization-evaluation`, `model-spike → diarization-jer-reference`, `jer-function-reference → diarization-jer-reference`다. 따라서 evaluator를 승격하면 도구 2개 이상의 실제 소비자를 보호할 이유가 있다. 나머지 주요 model adapter는 별도 runtime source fork/git revision/native library에 의존하며 제품 SDK가 이를 사용한다는 증거는 없다.

동시 변경 주의: 초기 manifest 읽기 때 model-review-bundle/visual-review에는 별도 workspace 선언이 없었다. SDK 담당은 root exclude만으로 Cargo의 잘못된 workspace 소속 오류가 해소되지 않아 두 manifest에 `[workspace] resolver = "3"`를 추가했다고 보고했다. **09:29 read-only 재조회로 두 선언의 존재를 확인**했다. metadata 복구와 root155pass/9ignored·clippy/build 통과는 SDK 담당의 별도 검증 근거이며 이번 리서치의 실행 결과가 아니다. 도구 승격은 아직 실행하지 않았고 그 manifest 수정을 조사자가 수행하지 않았다.

## 판정 기준

- **부분 승격 권고**: 기존 도구의 안정된 계산/DTO/port를 private library로 추출할 가치가 있다. 폴더 전체 이동이나 즉시 배포 승인이 아니다.
- **도구 유지**: 고정 cohort·fixture·source/build provenance·검사 orchestration·시스템 관리가 주 책임이다. 필요해진 작은 공통 책임만 별도 추출한다.
- **보류**: 제품 소비 계약·품질·플랫폼·배포 근거가 부족하거나 다른 소유 패키지와 중복이다.

승격 package가 제품 기능인지 개발용 library인지도 분리한다. 검증용 library가 packages에 있다는 이유로 GUI/CLI capabilities가 supported가 되지 않는다. 모델의 실행 성공, 숫자 대조, 사람 품질 승인도 서로 다른 상태다.

## 전체 24개 도구의 분류·소유 인계

| 도구 | 관측 책임·실제 의존 | 판단·추출할 책임 / 도구에 남길 책임 | package 후보·인수 담당 |
| --- | --- | --- | --- |
| boundary-observation | lib; schema3 decoder, typed Observation/TrustedPins/Artifacts, event accounting·projection·verify. serde/sha2 | **부분 승격 우선**: 순수 계약/decoder/검증 library. 고정 Gemma cohort와 FEA producer profile, 실제 artifact 경로·capture 연결 CLI는 tools | media-boundary-evidence(개발용); SDK 소유, AI 연구 producer 계약 협업 |
| diarization-evaluation | lib; interval/UEM/Hungarian assignment/DER/JER, bounded RTTM/audio/report 입력. pathfinding/ordered-float/hound | **부분 승격 우선**: 순수 metric kernel·명시 Protocol·Scores. local 파일 수집·고정 crop·보고서 생성은 tools | media-diarization-metrics; AI 연구, SDK typed caller 협업 |
| diarization-der-reference | lib; evaluator 직접 의존, pinned md-eval/Perl 외부 실행·대조·증거 | **도구 유지**: reference oracle 실행을 제품 core로 옮기지 않음. 공통 kernel은 승격 evaluator 소비 | 기존 tools 유지; AI 연구 |
| diarization-jer-reference | lib; evaluator 직접 의존, dscore source 검증·입력 준비·offline 출력 해석. source는 external parity false 명시 | **도구 유지**: 미충족 runtime gate와 caller-supplied 관측의 한계를 유지. 향후 reviewed scorer adapter는 개발용 선택 의존 | 기존 tools 유지; AI 연구 |
| engine-lab | bin; 고정 mistral.rs source/patch·build·resource admission·preservation. tar/flate2/sha2 | **도구 유지**: 엔진 연구/빌드 orchestration. 재사용 process/resource port 필요가 실제 생기면 작은 책임만 분리 | 향후 media-process-supervision 검토; AI 연구, SDK job port 협업 |
| forced-aligner-spike | lib; LocalModel/OriginalAudio/PreparationPort, 언어·텍스트·public alignment 관측. runtime lumos baseline fork/Candle | **제품 adapter 승격 보류**: adapter port의 출발점. 고정 모델·dictionary·prompt와 실험 실행은 tools | 향후 media-asr-aligner-adapter; AI 연구 |
| forced-aligner-repair-diagnostic | lib; public fix_timestamp/parse_timestamp를 3개 synthetic case로 관측. baseline fork | **도구 유지**: synthetic repair 진단은 실제 neural timing 또는 제품 자동 보정 정책이 아님 | 기존 tools 유지; AI 연구 |
| forced-aligner-observation-spike | lib; 별도 raw-head 관측 fork·compiled-input identity·public result | **도구 유지**: baseline과 관측 삽입의 대응·noninterference 근거를 보존. baseline에 덮어쓰지 않음 | 공통 tensor DTO 필요 시만 개발용 package; AI 연구 |
| forced-aligner-segmentation-spike | lib; 별도 segmentation-control fork·raw head·dictionary 관측 | **도구 유지**: segmentation 정책 변경의 실험 대조군. observation과 이름만 보고 합치지 않음 | 기존 tools 유지; AI 연구 |
| forced-encoder-observation | lib; PreparationPort/EncoderPort, tensor shape/dtype/bytes/nonfinite·internal encoder 순서. encoder fork | **도구 유지**: encoder 내부 tensor 증거. 일반 product embedding/ASR 반환 DTO와 분리 | 향후 media-tensor-evidence 선택 후보; AI 연구 |
| forced-head-reference-observation | lib; HeadPort/PreparedHead, encoder downstream head/timestamp 결과. 다른 head fork | **도구 유지**: encoder 관측과 다른 stage/입력 고정·oracle 역할. 제품 결과 성공으로 대체 불가 | 향후 media-tensor-evidence 공통 값만 검토; AI 연구 |
| jer-function-reference | bin; pyo3/Windows + private pinned CPython/dscore, mel/encoder/head/timestamp/segmentation 함수 oracle | **도구 유지**: Rust host adapter이며 외부 Python ABI/수치 oracle를 제품 runtime으로 확대하지 않음 | 기존 tools 유지; AI 연구 |
| model-review-bundle | bin+export bin; 고정 결과 package·source/hash 검증·HTML/read-only loopback·SDK export. 현재 lib 없음 | **부분 승격 우선**: generic review package validation/asset identity를 현역 제품 review 계약과 한 정본으로. 고정9 export cohort·HTML server는 tools | media-review-artifacts 또는 기존 review context; SDK 소유, AI 자료/UI viewer 협업 |
| model-spike | bin; acquisition/미디어 준비/모델별 실행·비교/다운로드·자원감독·대규모 report. ORT/reqwest/windows 등 | **통째 승격 금지**: pure OCR·audio fingerprint·postprocess 등 실제 소비별 kernel은 후속 개별 판단. fixed suite/acquisition/provenance는 tools | 향후 media-audio-analysis 등 맥락별 후보; AI 연구+SDK |
| native-gui-evidence | bin; 고정 r137/pinned run/host diagnostics의 seal/verify, SHA·copy·fresh publication; GUI 실행 안 함 | **부분 승격 후순위**: bounded evidence copy/manifest verification을 개발용 library. 시나리오별 pin·진정성 한계는 tools | media-evidence-archive; SDK 소유, UI·AI 소비 협업 |
| native-ui-workbench | lib; Catalog/Story/Violation 검증·schema/list/validate만 구현, planned HMR/GUI. serde only | **승격 보류**: 기존 catalog scaffold와 실제 Meari HMR/IR의 차이를 먼저 결정. 중복 새 UI framework package 생성 금지 | 필요 시 meari story catalog API로 개별 인수; HMR 소유, UI 소비 |
| qwen-asr-spike | lib; AsrPort·TranscribeRequest, fixed model/tokenizer/output policy. pinned git Candle Qwen engine | **제품 adapter 승격 보류**: typed inference port/검증 분리 가치 있으나 실제 품질·cancel/job·배포 gate 필요 | 향후 media-qwen-asr-adapter; AI 연구, SDK application 계약 |
| qwen-onnx-spike | bin; FP32 encoder+INT4/FP32 decoder exact variant, tokenizer audit·transcribe. runtime transcribe-rs fork | **제품 adapter 승격 보류**: ONNX variant/fallback/threads를 명시 adapter로 추출 가능. 준비·비교/고정pin은 tools | 향후 media-qwen-onnx-adapter; AI 연구 |
| runtime-controls-spike | bin; R4 exact engine/model/fixture pin·run receipt·collector·supervision·archive. reqwest/windows | **도구 유지**: 현재 R4 실행 준비/검사와 engine-specific diagnostics. generic port를 분리해도 actual 관측은 생성하지 못함 | 향후 process supervision 일부만; AI 연구, boundary SDK 소비 |
| sentence-embedding-reference | lib; InputPort/EnginePort, CandleCpuBert, strict token/pooling/vector reference. 33-row fixture/384dim/128tokens/4threads 고정 | **부분 승격 보류**: pure pooling·bounded token/vector contract 후보. generic embedding adapter는 fixture·고정모델 검사를 분리 후 | 향후 media-embedding-adapter; AI 연구, SDK retrieval 계약 |
| spec-maintenance | bin; .spec 경로 재배치·링크·고정 engine/build/source evidence·보존 검사 | **도구 유지**: repository 전용 mapping/exceptions를 제품 규칙으로 승격 금지. evidence-archive 공통 IO만 협업 가능 | 기존 tools 유지; AI 연구 현 소유, PM 문서 조율 |
| tokenizer-contract | bin; pinned tokenizer pair/prompt/audio-position 비교·보존. tokenizers0.21.4/hf-hub | **도구 유지**: 특정 token ID/processor 계약을 일반 tokenizer로 취급하지 않음. engine별 tokenizer version 분리 | 향후 tokenizer-contract 개발용 작은library만; AI 연구 |
| validation | bin; resource preflight/bench/soak/report + VM cleanup/display driver/DPI/system adapters. windows | **도구 유지·격리**: read-only resource sampling만 추출 후보, driver/install/remove/cleanup는 호스트 관리 도구에 유지 | 향후 media-resource-observation; SDK job 자원 계약+AI sampler, 호스트 작업 PM 조율 |
| visual-review | bin; PNG crop/nearest 확대/alpha count. image | **부분 승격 후순위**: pure bounded region/scale/alpha inspection API. CLI path/출력 파일 policy는 wrapper·publication adapter | media-visual-inspection(개발용); UI 소유, HMR 재사용 |

## 우선 후보의 구체 공개 API와 제약

### 1. 경계 증거 계약

근거: `tools/boundary-observation/src/lib.rs`는 application/domain/infrastructure를 공개한다. `application/projection/mod.rs:21–33`의 `ArtifactDecoder` 및 `convert(&TrustedPins, &Artifacts, &impl ArtifactDecoder)`는 파일·프로세스 없이 raw를 join한다. `application::verify`는 Observation과 ExpectedIdentity를 대조한다. `infrastructure::{strict_json,raw_v3,conversion_adapter}`는 SHA/decoder adapter다.

승격 표면은 typed Observation/TrustedPins/Artifacts/ConversionReceipt/Failure와 검증 함수다. manifest 경로 로딩·NEW_OUTPUT 생성·16MiB 한도 file adapter·CLI parse는 tools thin wrapper 또는 명시 infrastructure에 둔다. raw3와 Observation1을 같은 schema로 바꾸지 않는다. profile을 범용화하려고 producer stage/seed/rope/수치 검증을 약화하지 말고 `ProducerProfile`의 명시 버전을 검토한다. 검증 도구이므로 제품 SDK의 ApiError로 모든 실패 이유를 지우지 않고 단계·artifact pointer·원시 failure를 보존한 뒤 facade에서 요약 매핑한다.

**한계**: supplied-value 검증은 capture 인증이 아니다. 원래 source provenance와 full compiler closure, noninterference, 실제 child/supervisor exit 증거는 별도다. R3 startup sidecar 통과와 full join은 다르다. R4 strict full join은 이 조사에서 실제 실행하지 않았고 새 derived DTO가 actual 관측을 대신하지 못한다. synthetic tamper test가 실제 missing controls를 채워주지 않는다.

### 2. 화자 평가 kernel

근거: `diarization-evaluation/src/domain/{intervals,assignment,metrics,time}.rs`; `metrics::evaluate(reference,hypothesis,uem) → Scores`, 별도 `evaluate_legacy_endpoint_grid`가 있다. `Scores`에는 DER/JER·분모·할당·frame counts·matrix가 있다. `application::Request/run`은 파일·오디오·evaluator EXE hash를 결합해 report를 만든다. DER/JER reference 두 도구가 실제 path 의존한다.

승격은 interval/time/Speakers/ScoreProtocol/typed score 계산이다. RTTM/UEM parser도 generic bounded input contract로 추출할 수 있으나 고정 report path·audio SHA·crop 10ms..60s·freeze replay는 실행 wrapper에 남긴다. 60초 cap을 임의 해제해 복잡도·메모리가 무한해지지 않게 한다. legacy endpoint grid는 이름·version을 유지하고 새 default로 몰래 바꾸지 않는다. 공개 kernel에는 filesystem/Win32/Python/Perl이 필요하지 않다.

정확한 collar/overlap/10ms binary64 grid/RTTM onset+duration/assignment 방식은 artifact에 보존한다. 외부 DER oracle 실행, JER 숫자 대조, 모델 화자 품질 승인은 각각 별도 gate다. 평가값을 사람이 승인한 gold로 바꾸지 않는다. interval이라는 이름이 같다고 subtitle cue나 UI time과 자동 합치지 않는다.

### 3. 검토 자산 포맷

근거: `model-review-bundle/src/application/sdk_export.rs`는 고정 TEXT_SOURCES·manifest/measurement pin·정확9건·출력 `.spec/artifacts/quality-review`를 사용한다. src/main.rs와 export_main.rs는 bin이고 `domain::Bundle/Sample`은 historical viewer 표면이다. 제품은 이미 `src/domain/model_review/**` 및 `application/model_review/**`에서 ReviewPackage/asset/decision/read_text_asset를 소유한다.

권고는 **제품 현역 review 계약을 기준 정본**으로 선택해 generic package/asset validation을 분리하는 것이다. historical Bundle과 product ReviewPackage를 같은 이름의 두 SDK로 만들지 않는다. importer/exporter는 명시 schema별 adapter, 기존9 fixed source는 fixture provider, HTML escaping·loopback server는 개발 viewer로 남긴다. 모델 텍스트·JSON·HTML은 신뢰하지 않으며 bounded bytes/strict UTF8/참조 asset SHA·길이 검증·path confinement을 유지한다. 채택·수정·모델 판정은 해당 application context 소유로 남고 asset library가 자동 실행하거나 승인하지 않는다.

### 4. 증거·시각 개발 library

`native-gui-evidence`는 lib가 아닌 bin이며 r137/후속 pinned/host diagnostics 시나리오를 가진다. read-lock/copy/re-read hash/fresh stage/partial failure 원문을 보존하는 일반 부분은 재사용할 수 있다. “서명/원본 capture 인증/컴파일 입력 완전성”을 generic library가 보장한다고 표시하지 않는다. spec-maintenance/model-spike의 개별 preservation을 합치기 전에 artifact 범위와 trust model 차이를 비교한다. 과거 manifest를 신규 format으로 덮어쓰지 않는다.

`visual-review/src/main.rs`는 crop/scale 범위검사와 image IO/출력을 한곳에서 처리한다. `inspection.rs::inspect`는 alpha count를 출력한다. 후보 API는 `Region/Scale`, `crop_zoom(&RgbaImage,region,scale) → RgbaImage`, `inspect_alpha(&RgbaImage) → AlphaStats`와 typed BoundsError다. CLI parser/PNG path/new-file publish는 wrapper/adapter로 분리한다. 이는 확대 근거 생성 도구이며 실제 앱 시각 일치 판단·1:1 검토를 자동 통과시키지 않는다. UI source/approved assets를 Meari 범용 패키지에 옮기지 않는다.

### 5. 모델 adapter·감독 포트

실재 port는 qwen-asr의 AsrPort, forced-aligner의 LocalModel/PreparationPort, encoder의 EncoderPort, head의 HeadPort, sentence-embedding의 InputPort/EnginePort다. 그러나 일부 application이 직접 infrastructure를 호출하고 Prepared에 engine config/fixture 정보가 섞여 있다. pub trait 존재만으로 clean boundary 완료가 아니다.

새 product port는 JobId·cancel/deadline·진행·입력 source revision·모델 manifest/revision/license/정밀도·정확 variant·명시 output를 application 계약으로 받아야 한다. engine별 runtime/tokenizer/ONNX/Candle/native thread 설정은 adapter다. model path/argv/shell을 웹/모델 출력에서 받아 실행하지 않는다. 제품의 models.install/job queue 구현보다 experiment downloader를 먼저 붙이지 않는다.

process supervision이 필요하면 subprocess launch/owned job/timeout/output cap/resources capture를 한 adapter 책임으로 추출한다. engine-lab의 compile admission과 inference admission은 다른 profile이며 CPU<60 같은 실험 기준을 모든 제품 SDK 호출에 고정하지 않는다. threads=4 환경 선언은 실제 OS thread cap·GPU budget과 다르다. 사용자 앱 종료·driver 관리·전체 시스템 정리는 supervisor에 넣지 않는다.

## 라이선스·배포·플랫폼 인수 조건

새 package는 우선 private/publish=false다. 24 manifest의 license 필드 부재를 자동 MIT로 채우지 않는다. 라이선스 출처·변경 코드·원본 notice와 dependency SPDX/SBOM, 모델 카드의 revision·license·gated 조건을 담당이 확인한다. qwen-asr는 MIT 원문 보존이 README에 명시돼 있지만 실제 전체 배포 closure는 별도 검토다. Windows·ORT DLL·Candle/onig/esaxx·dictionary·CPython ABI·외부 scorer runtime은 Rust source와 별도 artifact다. 자동 binary download/copy-dylibs가 있는 ORT 설정은 reproducible/offline 배포 경계에서 재검토한다.

현재 pure library는 원리상 portable하더라도 타 OS 빌드 증거가 없으므로 지원 완료로 표시하지 않는다. Windows-only feature를 core 기본 의존에서 분리하고 기존 각 tool의 독립 workspace/lock/target를 유지한다. 사용 가능한 기능의 GUI/CLI 공통 facade는 root가 조립하며 package가 root SDK·presentation·Meari를 역으로 import하지 않는다. media-contracts는 최소 wire 값만, media-file-publication은 nooverwrite adapter만, media-subtitles는 자막 기능만 소유한다. 평가/모델/증거를 이 셋에 무차별 추가하지 않는다.

## 승격이 완료됐다고 판단할 근거

대상 kernel/port/공개 API·기존 소비자·schema/error/version/자원·platform·license·배포 범위를 먼저 확정한다. 기존 fixture와 raw 실패를 보존하고 새 library의 pure test, 모든 tool wrapper 소비자 test, fmt/test/clippy all-targets-Dwarnings를 같은 freeze에서 통과한다. 실제 CLI old/new 결과와 artifact bytes/float policy 차이를 대조한다. 제품 지원을 추가한다면 동일 GUI/CLI SDK client·capabilities·unsupported/error·cancel/restart 및 실제 UI 동작 증거가 추가로 필요하다.

현재 조사에서는 시험을 실행하지 않았다. tests 디렉터리 유무/inline unit source와 historical README의 성공 서술은 이번 실행 결과가 아니다. 새 추론·모델 품질 검토는 사용자 우선순위에 따라 API/SDK/UI 사용 가능 단계 이후다. 도구 승격 때문에 이 순서를 뒤집지 않는다.
