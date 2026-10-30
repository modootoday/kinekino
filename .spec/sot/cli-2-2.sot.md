---
id: sot:kinekino/cli-2-2
title: 비텍스트 역할의 기존 출력·품질·CLI 증거 경계
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 비텍스트 역할의 기존 출력·품질·CLI 증거 경계

2026-10-19 goal 후속 읽기 감사다. 기존 ASR/음악·화자·검색·OCR/비전/에셋 정본을 읽었고 새 추론·참조 실행·파일 재생성·빌드·GUI 검토는 없다. 전체23역할 범위를 text9로 대체하지 않기 위한 역할별 인계다. 스킬은 앞서 실제 읽은 modoo.today inference-benchmark-workload-protocol/quantization-and-vision-input-parity-check/llm-output-eval-suite의 **같은 입력/실패 분모·실제 적용 variant·task slice·quality/memory/time 구분**을 적용했다. 정확한 skill 이름·출처·reference SHA는 `modoo-existing-evidence-protocol-review.md`에 있다.

## 기존 관측과 인수 가능한 범위

| 역할·읽은 정본 | 이미 관측된 사실 | 품질/제품으로 넘어갈 때 남은 근거 |
|---|---|---|
| ASR: verification/ai/animation-asr-music.md | Sprite4clip INT4 ASR exit0/EOS4건. 참조 있는3clip subtitle-proxy CER25/2.03/9.6%, 자막공백clip은문장출력·CER미산출 | 자막공백≠무음·환각 확정. 사람 전사 없는clip을 CER0이나실패0으로채우지 않음. reference-available3/4와실행4/4를따로기록. 같은입력FP32대조없어quant원인미확정 |
| 음악/OPED: 같은 정본 | YAMNet4clip frame×521행렬, Music평균0.35758..0.58304, Speech와동시높음 | 점수는보정확률/상호배타라벨/OPED경계아님. 단일작품clip4개≠여러회차반복OPED. 실제반복구간·음악/대화/전후경계·오탐/누락 기준과취소 정책필요 |
| 화자: verification/ai/diarization-jer-functions-actual.md 및 matrix 후속 | 같은AMI두시간구간의6출력을원본함수로대조,corrected frame counts/JER일치·후속78행렬원소일치 | 계산동등성≠분리품질. holdoutJER약52.10/51.72%,소수화자누락은유지. 시간분리는독립meeting/작품이아님. legacy/corrected report분리·overlap/짧은발화미해결분모유지 |
| 검색: verification/ai/embeddinggemma2-quantized.md | 동일33입력/tokenIDs/mask/truncation FP32/q4,둘다잘림0. 임시5positive MRR0.82857→0.825,thanks7→8위. cosine평균0.98287 | assistant임시relevance≠human gold. 반복분산없어미세차이순위화금지. 무관질의topcosine0.61214≠match확률. no-match/다른자료·언어/thanks실패·정확variant·원본framework잔여 |
| OCR/마스크: verification/ai/ocr-natural-credits.md 및 vision.md | 자연크레딧30crop인식출력·제한렌더입력;MobileSAM에서의상누락,RFDETR person1/score0.95561 | 30crop≠30/30정확. detector/recognizer/후처리/읽기순서별gold필요. 모델predictedIoU≠정답mask실제IoU. person detection≠tracking/identity |
| 깊이/에셋: verification/ai/vision.md | 같은조건depthFP32/Q8각3회,private401596416/594804736B. GLBreader+독립CPU렌더·배열/PNG동일 | Q8의작은파일≠낮은상주메모리. 상대깊이·임의높이field≠metric/완전3D. 외부편집기roundtrip/가려진부분복원은미실행 |

위 경로의 verification은 `.spec/sots/verification/ai/`다. 이 감사는 기존 정본의 관측을 연결하며 모든 원시 binary·hash·row를 새로 재검증한 것은 아니다. 원래 label·build·모델·source pin과 반례/실패 원문은 계속 각 정본에 둔다.

## 실제 variant·메모리·시간 읽기

Sprite ASR은 FP32 encoder+INT4 decoder+FP16 embedding 저장/F32 cache이며 전체4bit가 아니다. English 강제 언어/max128·CPU intra4/inter1 조건이다. CER의 분자는15/4/12편집,분모는60/197/125문자이며 네번째clip reference없음은 별도다. 임의로세clip평균을전체4clip성적처럼표시하거나다른언어점수와합치지 않는다.

YAMNet은100ms메모리표본이각2개뿐인약0.24초이하프로세스다. 짧은모델inference구간과전체process/load를구분하고 관측최댓값을보장peak/다른장비성능으로보지 않는다. OPED의 자동skip은 후보검출점수만으로활성화하지 않고 명시채택·boundary/source revision·실패/취소·되돌리기 정책을 공통application에 연결해야 한다.

EmbeddingGemma2 FP32/q4는같은worker의각33행1회조건비교다. 전체시간8.4830909/4.9063231초,표본private872968192/447537152B이며load포함·각70/40표본이다. 원문·prefix/tokenID가동일한유효한제한대조지만반복분산/다른부하통제가없어항상더빠르다거나품질손실없음을승인하지 않는다. q4라는배포물명은모든initializer4bit/완전quantizer계보검증이아니다.

Depth3회반복은median을말할근거이나독립작품3개가아니다. Q8메모리증가라는실제반례를보존하고‘경량양자화’목표에맞추려고수치를제외하지 않는다. GLB출력100ms메모리표본1개는peak보장이아니다. 최초같은파일명overwrite이력과PID별출력후속을구별한다.

## 공통 CLI에 반환할 typed 결과의 책임

- ASR/정렬/화자: 원본오디오SHA/format·전처리/언어/variant·source offset을고정한다. 텍스트/word timestamp/overlap/unresolved는각기다른관측이다. 시간이없는출력에균등분할시간을사실로만들지않고정렬zero/repair수정과raw를분리한다. 미해결화자를reference/UEM에서제외하지 않는다.
- OPED/음악/검색: score와라벨·추천/수락을분리한다. frame/window/sourcePTS·embedding prefix/dim/normalization·index/content revision 및 ranked IDs/tie정책을보존한다. API는미지원/미관측/빈확정결과를구별한다. no-matchthreshold·자동skipthreshold는독립검증전임의값으로확정하지 않는다.
- OCR/마스크/깊이/에셋: 픽셀/회전·crop/scale/color/preprocess와원본PTS·객체좌표계를고정하고 detector→recognizer 입력연결을남긴다. 저장tensor정밀도와runtime dtype을별도기록한다. 상대깊이/단일표면/미복원영역을숨기지 않는다. output artifact와검증receipt를nooverwrite로게시한뒤completed로연결한다.

공통jobs는현행JobExecutor/ExecutionControl을재사용하며이문서가새operation/schema를등록한것은아니다. 입력descriptor와실제binary artifact를구분하고큰score행렬/벡터/raw는bounded별도artifact로보존한다. cancel/deadline/lateoutput·소유engine종료·retry/recovery는모델없는SRT근거만으로통과시킬수없다. 구체현행접합은 `ai-executor-current-jobs-integration.md`를따른다.

## 남은 gate의 상태

계산/형태동등성·모델실행·typedCLI기능·품질·사람명시채택은별도다. 원래license/revision/변환계보/정밀도고정잔여와23역할의품질승인0을유지한다. 각역할human gold·독립slice·오탐/누락·no-match·반복/precision대조는미실행후속이며신규모델실측은제품CLI준비와중앙배정전까지보류한다. 이번감사로GUIgate·외부프로그램설치·새평가실행을추가하지 않았다.
