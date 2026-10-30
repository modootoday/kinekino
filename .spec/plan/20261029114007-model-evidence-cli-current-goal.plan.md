---
id: plan:kinekino/20261029114007
title: 모델 근거·공통 CLI 계약의 현재 실행 목표
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 모델 근거·공통 CLI 계약의 현재 실행 목표

2026-10-19 사용자 직접 goal 갱신 지시를 PM 경로로 받았다. get_goal 결과는 goal=null이어서 중복/상태 위장 없이 새 goal을 생성했다. token budget은 요청되지 않아 설정하지 않았다. 도구는 active objective 수정 API가 없고 create는 미완료 goal이 있으면 실패하며 update는 complete/paused/blocked 상태만 지원한다. 향후 목표가 바뀌어도 미완료를 완료 처리하거나 보류 지시를 goal paused로 바꾸지 않고 이 계획을 갱신한다.

## 현재 실행 목표와 범위

HF revision·license·실제 precision을 고정하고 경량 양자화·Rust 우선 엔진/도구를 기준으로 기존 모델·엔진·실제 자료를 감사·정본화한다. 공통 SDK/application/domain에 기반한 CLI 요청·결과·artifact·오류·취소·provenance 계약 및 마지막 사람 품질 검토를 준비한다. Rust-only·SRP·모듈화·계층 구조·원시 실패/근거 보존과 .spec 책임 분리를 지킨다.

ASR/정렬·화자분리·OP/ED·비전/OCR·에셋·임베딩/검색·교정/문구/하이라이트 등 기존 전체 설계 범위를 유지한다. 이번 R3/R4/text9 checkpoint를 전체 역할 완료로 확대하지 않는다. 23역할의 기존 실행 목록과 품질 승인0의 구분도 유지한다. 기능 검증은 실제 SDK/CLI 고정 입출력·오류·종료 근거이며 GUI 수동 점검·시각 gate를 추가하지 않는다.

## 완료한 현재 checkpoint

- R3 고정 실제 경계 자료·startup/owned stop·외부 측정·source/engine/worker 원문 보존.
- R4 source/fork freeze 및 기존 check/test1388pass5ignored/strictClippy/CLI build 자연terminal0. 정확 EXE·21receipt·hash inventory 보존. worker pin은 pending 그대로.
- 실제9 source decode/typed mismatch와 원문/명시 수정·채택의 별도 의미, CLI engine/model 입출력·cancel/provenance 인계 문서 작성.
- modoo.today skill_find로 적합성 확인 후 inference-benchmark-workload-protocol, quantization-and-vision-input-parity-check, llm-output-eval-suite 본문과 앞2개 필수 reference를 실제 읽고 적용. 정확 이름·출처·reference SHA·적용 판단은 `.spec/sots/player/infrastructure/inference/research-followups/modoo-existing-evidence-protocol-review.md`에 기록.
- 후속 읽기 감사로 실제9의 모델3개 revision/bytes/SHA·저장 tensor별 혼합정밀도·서로 다른 engine pin·3task batch 시간/메모리·알려진 length 잘림을 연결했다. inspector의 runtime_executed=false를 smoke 미실행으로 오독하지 않도록 구분했으며 license metadata와 실제 배포 권리/계보 미완료를 유지했다. 전체23역할의 새 full hash/권리 검증을 수행한 것은 아니다.
- 현행 media-jobs source·기존 실제CLI 검증 정본을 읽어 AI 접합 검토를 작성하고 초기 ‘jobs 포트 없음’ 자료에 시점 정정을 추가했다. 기존 JobExecutor/ExecutionControl 재사용, cancel expected_attempt/retry expected_revision, UTF8 descriptor와 binary artifact 분리, cancellation/publication/recovery·실패 raw 근거 인계 잔여를 구체화했다. 새 adapter/모델 실행 완료를 뜻하지 않는다.
- 비텍스트 기존 근거를 cross-role-output-evidence-audit.md에 연결했다. ASR reference3/4와실행4/4·혼합INT4,음악점수≠OPED확률,동일AMI시간분리≠독립corpus,JER계산동등성≠분리품질,검색임시gold/no-match와FP32/q4손실,OCR30출력≠30정확,깊이Q8메모리증가·2.5D한계를보존했다. 역할별CLI typedoutput/artifact/취소근거잔여를유지하며새실측은없다.
- 누적 인계 뒤 기존 고정 Rust spec-maintenance EXE SHA6a2fd4e9f6e028959a3d0bd538bb00abe5f09cd57573a593c8b2fcc080d2e391로 inspect-text-r3-source/check-links를 실제 재검증했다. 두 명령terminal0,source3asset의SHA/실제LF1경계/직접배열해석재확인·문서799/canonicalbroken0/upstream4. `.spec/evidence/verification/model-evidence-goal-protocol-2026-10-19/`에원문보존. source-decode SHA9d88e2f1cdccb73f60b48fc5e1c6c71072ae1a58e139b98ef90d507328ab797e,links.log173c2cbd0920ecd1a77d207e6957a6b735f30b3bd3b24c390f73aa65eb470521. 모델/typed수입/품질검사가아니며새compile없음.
- 보존Gemma/Llama acquisition의card/metadata4개를직접읽고nativeSHA를기존result선언과대조했다. 고정card표기·Llama내약관문구존재와originalLICENSE/수락/배포/변환계보완결을분리했다. Qwen0.6확인hub snapshot은GGUF1파일이므로고정license원문추가확보로표현하지않는다. 상세hash/경계는modoo기존근거감사에추가,새download/법적승인없음.

## 잔여와 다음 gate

1. 저비용 우선: 기존 근거에서 workload 길이·실제 weight/KV precision·cache/concurrency·잘림/실패 분모·quality/memory/time의 관측/미관측을 case별 구분한다. CLI 계약의 source/model/profile·typed 결과·raw/output·오류·cancel/late callback·revision/재시작/nooverwrite 간극을 소유 세션에 인계한다. 평가/도구 승격은 기존 소비자·protocol/float grid·pin을 보존하는 설계부터다.
2. 제품 SDK/API/CLI 기능 및 소유권 조율을 우선한다. 제품 source·공유 Cargo는 소유 세션/PM lease 없이 변경하지 않는다. 필요한 코드 변경이 배정되면 변경 freeze의 fmt/test/strict all-target Clippy를 수행하되 같은 checkpoint를 중복 검사하지 않는다. 정상 실행 작업은 중단/재시작하지 않는다.
3. 신규 추론·모델 다운로드·cold worker build·실측 예약은 현재 유보다. 제품 CLI 준비와 중앙 자원 배정 뒤에만 R4 worker native gate/pin·archive profile admission·actual controls/commit/ACK·SDK source admission/strictjoin·비간섭/실제 모델 input/output 검증을 진행한다. 아직 실행하지 않은 검사를 완료로 표시하지 않는다.
4. 실제 workload에 맞는 길이/캐시/동시성·paired 반복과 품질/메모리/시간 결합 평가는 미실행 후속이다. baseline과 같은 case·task slice·오류 분모를 고정하고 원문/관측 None을 유지한다. 사용자 사람 품질 검토는 개발 검증·코어 준비 뒤 마지막 별도 단계이며 지금 승인 요청으로 전환하지 않는다.

스킬의 synthetic 숫자·다른 언어 예제·추가 CI/인프라/과금·모델/GUI 실행을 프로젝트 실행 근거 또는 자동 gate로 가져오지 않는다. active goal은 이 전체 잔여를 포함하며 이번 문서 작성만으로 complete/paused/blocked 처리하지 않는다. 주요 완료나 실제 장애만 PM에 묶어 보고한다.

현재 실행 경계: PM 최신 진행을 읽은 시점에는 실제3카테고리의 페이지 구조차이/첫요청timeout을SDK가보존원문으로수정중이었다. 제품전체CLI준비/AI자원해제를추정하지 않는다. 이관한정적감사·계약검토와위2비추론검사는완료checkpoint이고,fullgoal의license/actualprecision·신규worker실제관측·품질잔여는미완료다. 이기록을완료신호로새추론/download/workerbuild를자동재개하지 않는다.

## 보류 의존성 재검증 기록

첫 점검(직전 턴은 license evidence 확인의 진척): 현재 PM turn에는 SDK parser v3 필수검사/CLI고정→Meari Core9→UI consumer 예약이 명시됐으며 AI worker/추론 자원 해제는 없다. 현재 R4 worker engine-pin은zero SHA/bytes0/pending이고 recommended runtime-controls-engine-pin-r4.json도 없다. 이는 소유 source를 읽은 사실이며 실제 live process를 기다리는 verified wait가 아니다.

기존 허용 범위의 증거 감사·계약 인계·비추론 source/link 검사는 위 checkpoint로 보존했다. 남은 actual controls/precision·실패/취소·비간섭·품질 증거에는 보류 해제와 소유 구현/중앙 자원 단계가 필요하다. 같은 문서 재서술·동일 검사 반복을 진척으로 세지 않는다. 이번은 이 동일 의존성의 첫 no-progress 점검이며 goal은active 유지한다. PM에 다음 소유 저비용 과제 또는 CLI준비/중앙 자원 해제 변경 인계를 요청한다. 이 기록만으로 paused/complete/blocked 상태를 만들지 않는다.

후속 PM 과제 수신으로 위 no-progress 연속은 종료됐다. 현 proposal 원문→edit→accept와 media-jobs source를 대조하고 기존 문서와 겹치는 내용은 재작성하지 않았다. PM에 source line 근거로 실제model job provenance→GeneratedResult(task/raw만) 상관 연결, correction document→실제subtitle 적용, card mapping/export, highlight ref/time→skip/render의 별도 잔여를 인계했다. 기존 create/edit/accept/재시작의 실제CLI근거는유지하며9건fixture::technical_edit를사람수정/품질승인으로세지않는다. accept는CAS저장·automatic_applicationfalse이며렌더/외부파일적용이아님을현재코드에서확인했다. 구현/compile/network/inference0,UI source-only 실행승격0이다.

접합 과제 완료 후 첫 의존성 점검: PM/중앙 계획은해당차이의SDK인계를확인했고,현재검사슬롯은UI consumer→SDK회차fixture순서다. 새AI과제·추론/workerbuild해제는없고현workerpin도pending이다. 소유범위내새실행가능작업이없어이turn은no-progress로기록한다. 동일감사/동일검사반복을하지않으며goalactive유지. 이전연속은새접합과제로종료됐으므로이번은새연속1회다.

두번째 연속 의존성 점검: 현재 중앙 계획은 최종 두 계약의 SDK 인계와 인수항목 미실행·동일감사 반복금지를명시한다. AI 실행/workerbuild해제나새소유과제는없으며workerpin도pending그대로다. 문서재작성·반복검사를진척으로세지않는no-progress2회이며아직blocked상태로바꾸지않는다. 다음필요변화는담당구현/CLI준비및중앙자원인계또는새소유과제다.

세번째 연속 의존성 점검: 동일 중앙 계획과현workerpin을다시읽었고조건변경/새소유과제가없다. 보류된신규실행을시작하거나타담당source를변경하지않고서는남은actualprecision/controls·모델취소/비간섭·품질인수근거를만들수없다. 승인범위의정적감사/인계는이미완료했고같은내용재작성은진척이아니다. 동일외부의존성no-progress3회에따라goal을blocked로전환한다. 전체objective/미완료/보존자료는그대로유지하며CLI준비와중앙자원배정또는구체새소유과제가인계되면재개한다. 완료/사용자품질승인/goal paused로대체하지않는다.

## 사용자 직접 재개 지시 이후

사용자가 최신 AGENTS 반영·PM 조율·현재 가능한 저비용 작업부터 재개하도록 직접 지시했다. root/.spec AGENTS와 중앙 계획을 다시 읽고 작업을 재개했다. 전체 목표·미완료는 유지한다. SDK/CLI 우선, modoo.today 본문/참고자료의 실제 적용, Rust 정책, 승인된 Meari TSX UI 선언의 제한된 예외를 구분하며 신규 추론/download/worker build는 계속 중앙 배정 전까지 보류한다. PM에 재개와 독립 소유 범위 조율을 전달했다.

첫 저비용 변경은 research-followups/README의 초기 시점 ‘모두 미실측’ 범위 오해를 바로잡고, 현재 jobs/CLI·실제9·modoo 근거 감사·사람 검토 준비의 책임별 진입점을 연결한 것이다. 기존 감사를 반복하거나 모델 실행 근거를 새로 만든 작업이 아니다. 코드/Cargo/모델 artifact/source freeze는 변경하지 않았다.

도구 상태: 사용자 지시 후 get_goal은 여전히blocked를반환했다. update_goal은complete/paused/blocked만지원하며active/resume인자가없다. 기존미완료goal을완료로위장하거나중복create하지않는다. 사용자재개권한으로실제작업은재개하고API상태제약을PM에알렸다. 이후blocked감사필요시이번재개부터새로계산하며과거3회를재사용하지않는다.

PM 독립 소유 배정 후 tools/model-review-bundle의고정source/sdk_export를읽고기존에없는historical companion source를추가했다. domain/application/infrastructure와4회귀를분리하고기존pins/digest/publish재사용,SDK DTO/rootCargo불변이다. 고정실제9 package/source/candidate/model/engine연결·bounded reads·명시absent/declared/observed·historicalunknown보존이범위다. source/fmt와fmtcheck만exit0,central예약필요한test/strictClippy/exportbin build·actual9비추론생성/재조회는미실행이다. README및model-engine-cli-contract-handoff.md에현상태·소스링크·미검증범위를반영했고필요검사명령을PM에전달했다. 새추론/download/workerbuild0,제품연결/품질승인아님이다.

PM은UIconsumer→SDK회차jobs-target/Clippy→AI companion의test/Clippy/exportbin/fixedactual9순으로검사를예약했다. publication원자성요구에맞춰companion전용staging+hardlink no-replace/재읽기adapter와회귀를추가(총새5개),기존export/SDK게시정책은불변이다. 최신source는fmtcheck0까지만확인했고예약전추가compile/모델실행없음이다. 슬롯인계후필수Rust검사와actual9readback·원본불변/누락·손상실제결과를보존해야한다.

## 2026-10-19 중앙 배정 검사 결과

사용자 goal reset 뒤 get_goal=null을 확인하고 전체 연구 범위와 보류 항목을 보존한 새 goal을 생성했다. 현재 상태는 active다. 이전 blocked/API 재개 제약 기록은 과거 이력이다.

PM 검토를 반영해 독립 companion의 중복 파일 게시 구현을 제거하고 기존 media-file-publication::publish를 Rust 어댑터로 재사용한다. 도구만 이름/크기 제한, RevisionConflict 이후 동일 bytes 재조회, 다른 오류 거부를 소유한다. tool Cargo.toml의 두 path dependency와 offline lock만 갱신했으며 root Cargo/제품 DTO/공통 package source는 변경하지 않았다.

중앙이 UI→SDK 검사 종료 후 AI 슬롯을 배정했다. BelowNormal/-j1/--offline에서 lock 갱신, fmtcheck, locked test(기존 포함 11 pass/0 fail), strict Clippy all-targets, export binary build를 완료했다. 첫 test는 serde unit variant가 absent의 불필요한 value를 허용해 1건 실패했으며 원로그 test.log를 보존했다. Absent {}로 최소 수정한 test-repair-1.log는 모두 통과했다. 신규 companion 회귀는 6개다.

고정 실제9에 companion 생성→verify-companion→동일bytes 재게시 모두 exit0이다. companion SHA256=7a7cd05eafa6294e846c8080c976600ea135aa91e9da315ac9f770e45f624b10, 도구 EXE SHA256=ee85e5b6c204ecaebdb656e9dbcd3f2491f05e18e91770f8931f75c6e6e2604c. 원본 objects/package 전후 해시 목록 SHA256은 모두 8a6dcbcc360fdd0197e56fdaa43b03d3ff4ffc263574ba6121e17e406c652f12다. 손상/누락/덮어쓰기/상태 승격 거부는 Rust 회귀 fixture에서 검증했으며 실제9 원본을 훼손하지 않았다. 실제9를 복사한 부정 시나리오는 미실행이다.

로그: [.spec/evidence/verification/historical-companion-2026-10-19](../evidence/verification/historical-companion-2026-10-19/). 새 추론/download/worker build는 0이다. 도구 초안의 검증이며 제품 SDK admission·품질 승인·전체 goal 완료가 아니다. 후속 SDK 인계와 잔여 검증 범위는 유지한다.

고정 실제9 복사본 부정검증 후속: 중앙 배정에 따라 ai-companion-negative-20261019 scratch만 변경했다. 동일 고정 binary의 baseline verify exit0 이후 missing-object(os error2), corrupted-object(SHA256/bytes differ), declared→observed 승격(independently reconstructed evidence differs), 기존변조companion 게시(no overwrite), malformed-companion(EOF JSON) 모두 실제 CLI exit1을 보존했다. no-overwrite 전후 companion 해시는 동일했다. 원본 objects/package 목록은 부정검증 후에도 최초 목록 SHA256 8a6dcbcc360fdd0197e56fdaa43b03d3ff4ffc263574ba6121e17e406c652f12와 동일하다. copy-*.log가 근거이며 이전 ‘actual9 복사본 부정 미실행’ 항목을 이 범위에서 해소한다. scratch는 의도적으로 잘못된 fixture이고 제품/품질 승인 입력이 아니다. 추가compile/추론/download/worker build는 없다.

SDK 수신 확인: 담당이 companion근거를 backlog로 인수했고 제품admission/provenance adapter는별도6단계gate로남겼다. 현재 회차→episodeDOM→resolve/native우선이며 새도구/모델실행요청은없다. 정본/README의과거미실행문구를최신검증상태로통합했으며원로그/이력은보존했다. 남은소유구현과중앙배정은미해결이고새독립저비용과제를PM에요청했다. 이 의존성은 이번 재개 후 첫점검이다.

후속 의존성 점검: 중앙 최신계획은 AI actual9 복사본 부정검증 인수를확인했고 다음Meari31→SDK episode관측검사를예약했다. SDK는companion소비를backlog로인수했으나제품provenance gate/보류해제/새AI소유과제배정은없다. 앞회차중앙live대기이후이번점검도새구현·검증진척은없다. 동일반복감사를시작하지않고goal active/원범위유지,현재자원및소유구현의존성을기록한다.

차단 감사(현재 재개 구간): companion 구현·Rust검사·실제9정상/부정CLI·원본불변·SDK인계 이후, 연속 세 회차에서 같은 외부 의존성을 확인했다. 중앙 live작업 대기/최신계획 및 SDK상태 점검에도 제품provenance소비 구현 배정·새AI소유과제·모델실행자원해제는없고 runtime enginepin도pending이다. 이미끝난검사를반복하거나타담당소스를침범하거나추론보류를해제하지않고는남은정밀도/취소/비간섭/품질·제품인수근거를만들수없다. 전체목표는미완료로보존하고goal을blocked로기록한다. 재개조건은SDK소비계약/구현인계 또는 구체새저비용소유과제 또는 중앙의명시자원배정이다.

PM 즉시 재개 지시 후 실제 source 작업을 재개했다. goal API는 blocked이고 active/resume 인자는 없으므로 기존 목표를 완료 처리하거나 교체하지 않았다. SDK가 신규 src/application/ai_provenance/ 경계를 지정했다. DTO/rootCargo/assembly불변으로 evidence-reader port·private prepared binding·stored原文 검증과 actual Gemma 원문을 사용하는 SDK create/edit/accept/직렬화 재조회 fixture를 작성했다. 도구에는 read_verified/inspect-import 선택 원문+companion 반환/재hash 회귀를 작성했다. fmtcheck 통과, test/Clippy/build는 중앙 SDKframe→UIcontroller→Meari→AItool 슬롯 순서 대기다. 제품 연결 및 실제 CLI/원자적 provenance저장·project capture는 미완료이며 기존 회귀결과를 새source 통과로 승격하지 않는다. 신규 추론/download0 유지. 현재 구현 상세는 player/ai-provenance-integration.md를 따른다.

중앙이 Meari 반환 뒤 AI tool 실행슬롯을 명시배정했다. resource-runner(실제BelowNormal/2CPU/descendants receipt)와lockedoffline-j1에서fmt/test/strictClippy/build모두0,일반12pass1ignored 및고정bin사용Rust actual9ignored인수1pass0fail을수행했다. 정상verify/import0·missingitem1,실제scratch의source/candidate누락·동일크기변조·companion승격/누락각exit1,baseline/restored0·원문/원본bytes및원manifestpins불변통과. sourceinventory전후e223570e2e23ffb6d51ae2c095791063918b5fbb284fc976fb72907045482689동일,고정EXEe00c1726603ef7d0167c6d58deb79e694071044c20a2723de8abe8a504f45e41. 슬롯을PM에즉시반환했다. 이번소스첫실행은수리없이통과;기존다른gate실패이력은보존. 새추론/download/worker0. SDKroot미등록제품module·MemoryStore검사는포함되지않았으며전체goal미완료다. 로그/명령/다음연결은player/historical-import-tool-gate.md와player/ai-provenance-integration.md에기록했다.
