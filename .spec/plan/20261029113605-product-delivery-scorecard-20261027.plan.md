---
id: plan:kinekino/20261029113605
title: 실제 제품 납기·목표별 인수표 — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 제품 납기·목표별 인수표 — 2026-10-27

2026-10-28 사용자 재요청의 현재 요약은 [제품 목표 기준 현황](product-goal-status-20261028.md)을 따른다. 아래 반복 checkpoint는 원 시점의 증거 이력이며 전체 제품 완성률로 합산하지 않는다.

2026-10-28 최신 build 근거는 [R50 지정190/정상bins](../../sots/verification/application/source-sdk-r50-integrated-regression-20261028.md)다.2305입력에서190/190·정상2bins·최종동일을 실제 확인했다.원 frame 후손 기한 결함도 선택 검증으로 수정했으나 receipt복구·실행중취소·전체CLI/GUI·AI품질0/9는 남는다.전체 완성률과 납기는 계속 미확정이다.

2026-10-28 최신 빌드 근거는 [R46 지정190/Native 정상bins](../../sots/verification/application/source-sdk-r46-integrated-regression-20261028.md)다.2302 고정입력에서 지정190/190·정상2bins·최종입력동일을 실제 확인했다.명시jobs host의 endpoint/typed frame API도 선택 인수했지만 전체CLI/GUI율과 납기는 미확정이며 AI품질0/9·실행 시각 인수는 남는다. 아래 이전 gate는 각 원입력의 역사로 보존한다.

2026-10-28 최신 빌드 근거는 [R43 지정190/Native 정상bins](../../sots/verification/application/source-sdk-r43-integrated-regression-20261028.md)다.2300고정입력에서190/190·정상2bins·최종입력동일을 실제확인했다.이후의R40/DFB0기록은원checkpoint로보존한다.제품전체CLI/GUI율·납기는여전히미확정이며AI사용자품질0/9·실행시각인수미완료다.

## 사용자 재요청 기준 — R34 확인 후

실행파일은 존재하지만 실제 사용 가능한 통합 제품은 미완료다. 완료일을 뒷받침할 전체 인수 항목과 잔여 작업 실측이 없으므로 날짜·전체 CLI/GUI 백분율은 미확정이다. 기존 발견 명령97개는 완료 분모가 아니다. 아래 표가 현재 요약이며 이후 단락은 원 시점의 기록으로 보존한다.

|목표|지금 구현·확인한 범위|앞으로 필요한 범위|목표 기준 상태|
|---|---|---|---|
|새 호스트 실행파일·회귀|후속 private R40에서 player/player-cli 정상빌드2/2·지정190/190·2297입력동일 재검증;원DFB0별도보존|canonical 채택·제품전체 기능/CLI/GUI 인수|R40 지정 build/회귀 gate 통과;제품전체율 아님|
|탐색·영상 접근|보존 DOM 카탈로그/작품/회차, provider/lease 계약|현재 사이트·외부 제공자·광고차단·재생의 제품 경로 및 실패 인수|부분|
|다운로드|private SDK/JobsCore/CLI 오프라인 본문·발행·취소·오류; R37 요청 전 owner 보존 및 로컬 파일/실제 ffprobe 선택9 회귀 통과|production 최초 owner 획득·receipt 실패 중 정리·실제 provider 정책/HTTP source+probe 통합·제품 등록|부분; 실제 제품 다운로드 미인수|
|프로젝트·저장·재생|저장/CAS/이력·locator 재연결, 구성 SDK의 저장/재생/재시작/EOF|내용 검증·재생 재결합 및 전체 진입 경로의 실패/복구|부분|
|자막·AI 제작|수동 cue/화자 편집, 모델 조회/파일 검증, 후보 수정·판정·이력|ASR·정렬·나머지 모델 실행/취소/저장, 실제 결과 검토 GUI|부분|
|AI 사용자 품질|과거 실제 표본9개 보존, 모두 NeedsRepair|원본/후보 비교·재생·수정·명시 판정·이력 후 사용자 승인|승인0/9; 개발 미완료|
|CLI 전체|새R40 CLI의97명령×기본/저장소두구성schema대조완료·지원표시9/28;기능별실제SDK/CLI근거별도|나머지host·명령×구성×정상/오류/취소/기한/저장·비적용근거연결및전수종료|발견정합통과;기능전체율미산정|
|GUI 전체|정상 빌드·Meari/UI 선언·일부 공통 API 연결|CLI 선행 인수, 기본 여정·SDK 연결·상태·입력/포커스/접근성|전체율 미산정|
|UI/UX|2026-10-18 설계 승인|기준 시안 요소별1:1 대조·실제 실행 재검토·오류/빈/로딩 상태 일관성|설계 승인; 실행 시각 인수 미완료|

중앙 PM의 다음 범위는 source 인수 이전의 부분 초기화 소유권 공백을 닫고 기존 file/probe 회귀를 확인하는 것이다. 이후 제품 source 등록과 전체 CLI 인수표 종료, GUI/AI 검토 연결, 묶음 시각 인수 순서로 진행한다. 반복적인 private 시험 통과를 제품 완료로 누적하지 않는다. R34 근거는 [preparation custody](../../sots/verification/application/preparation-source-custody-20261027.md), 전체 작업은 [현재 검토 빌드](current-review-build.md)를 따른다.

배포 제외·source375 private overlay·기존 WIP/담당 소유권/원 실패·추론/다운로드/수동GUI/WARP/push 보류를 유지한다. 필수 최소 의존성 설치 승인은 유지되며 보류 작업 실행 승인과 구별한다.

## 사용자 후속 보고 — 실제 사용 가능한 제품 기준

완료일은 미확정이다. 제품 실행파일 존재, private 후보 시험 성공, 사용자 검토 가능한 통합 제품을 각각 구분한다. 기존 97개 발견 명령을 전체 CLI 완료 분모로 사용하지 않는다. CLI는 명령×제품 구성×정상/오류/취소/저장, GUI는 화면×상태×입력/포커스/접근성 및 SDK 연결의 인수 항목을 고정한 뒤 통과 수로 보고한다. 그 전 전체 백분율은 미산정이다.

|목표|구현된 범위|남은 목표|현재 완료 근거|
|---|---|---|---|
|빌드 가능한 제품|새 호스트 정상 player/player-cli|후속 후보 통합 및 영향 회귀|고정 DFB0 입력 bins 2/2·지정 회귀190/190|
|영상 탐색·접근·다운로드|보존 DOM 탐색, private source IPC·SDK/JobsCore·CLI 오프라인 흐름|원 브라우저 replay 발급, 실제 HTTP/probe 및 제품 연결|부분; 실제 다운로드 인수 아님|
|저장·프로젝트·재생|저장/이력/locator 재연결, 작업/변환/범위 출력 및 구성 host 재생 근거|내용 검증·재생 재결합·전체 실행 경로와 실패 복구|부분 검증|
|자막·AI 제작|수동 cue/화자 편집, 모델 조회/무결성, 후보 수정·판정·이력|ASR/정렬/시각 편집·실제 모델 경로·결과 품질|부분 구현; 사용자 품질0/9|
|CLI 제품 인수|97명령 발견 및 구성별 근거, 여러 실제 종단 시험|제품 구성별 미지원/미검증 행렬 종료|전체율 미산정|
|GUI 제품 인수|실행파일·UI 선언·일부 공통 API 소비|CLI 선행 인수 후 기본 여정·상태·접근성·AI 비교/수정/판정 연결|전체율 미산정|
|UI/UX 일치|2026-10-18 설계 승인|기준 시안과 실제 실행의 요소별1:1 비교·재검토|설계 승인, 실행 시각 인수 미완료|

현재 작업은 기존 담당의 metadata7 변경분을 최신 private SDK 후보에 한정 통합하고 원 소유권/세대/취소 검사를 검증하는 범위다. 1113 strict 첫 실패와 preimage를 보존했으며 1114·1115 strict는 통과했다. input-r17은2287파일이며, 이는 원 root/source375 canonical 채택을 뜻하지 않는다. metadata lease를 GET replay 권한으로 승격하지 않는다. 후속 순서는 replay 발급·소비 계약 공백 해소 → 실제 source/SDK/CLI 통합과 잔여 CLI 인수 → GUI 결과 비교 연결 → 묶음 시각 검수다. 보류 중인 추론·다운로드·수동GUI·WARP·push는 이 보고로 재개하지 않는다. 배포는 목표 밖이다.

metadata7 진행 근거 추가: 1116은 terminal/exit0이며 새 provider 소유권 시험2/2 통과(928 filtered)다. 동일 원 소유권 유지·reload 후 재발급 거절·취소 후 lease 무효화의 오프라인 검사이며 실제 브라우저/HTTP 인수가 아니다. input-r17 SOURCE.md SHA256=AC682BAF0A01A5B42979E77098AF82AF2F120C69D01DBD6BE56AC4E39706E2BF, inventory=DBBC5AE5991F8E25E39E6D6D6DD8E50B3422A87EA0FA08E24E6F7E7125F64A3E. ledger 회귀·fmt·전후 custody 동일 검증은 다음 작업이며 이번2시험으로 승계하지 않는다.

## 현재 목표 기준표 — 부모 transfer R36 이후

아래 표가 현재 보고 기준이다. 이후의 각 '최신' 단락은 해당 입력의 역사 기록으로 보존한다. 납기 미확정이며 실행파일 존재와 사용 가능한 통합 제품을 구별한다. 분모가 고정되지 않은 CLI/GUI에 추정 백분율을 부여하지 않는다.

|목표|현재 구현·검증|남은 완료 조건|완성도 판정|
|---|---|---|---|
|새 호스트 빌드|DFB0 고정 입력에서 정상 player/player-cli 2/2, 지정 회귀190/190, fmt/strict|후속 private 후보의 제품 통합·영향 회귀|해당 빌드 gate100%; 제품 전체율 아님|
|탐색·회차·영상 접근|보존 DOM 카탈로그/회차 SDK·CLI, source 수명/IPC private 후보|현재 사이트/provider replay 권한·실제 source/제품 SDK·CLI 연결|부분 구현|
|다운로드|private SDK/JobsCore·정상 CLI 연결;offline 정상/EOF/종료 오류/검증 단계 취소 근거([R16](../../sots/verification/application/source-sdk-cli-cancel-20261027.md))|실제 browser replay issuer·HTTP·media probe·추가 실패 행렬·제품 반영/인수|부분 구현/검증;root 제품 등록·실제 HTTP 미완료|
|저장·프로젝트·작업|프로젝트 저장/locator 재연결·이력, durable jobs·변환/범위 출력의 실제 CLI 근거|남은 executor/실패·중단 복구 행렬, 내용 검증·재생 재결합|부분 검증|
|재생·검토|구성 host의 동일 SDK 저장/재생·제어·재시작/EOF 근거|모든 제품 진입 경로와 오류/취소/정리 통합|부분 검증|
|자막·제작·모델|cue 텍스트/수동 화자 편집, 모델 목록/조회/무결성, 후보 수정/판정/이력|ASR·정렬·시각 편집·나머지 모델 실행 경로와 실제 품질 검증|부분 구현|
|AI 결과 품질|기존 실제 후보9개 보존, CLI import/edit/판정의 기술 검증|GUI 원본/후보 비교·재생·수정·명시 판정·이력, 사용자 품질 승인|사용자 승인0/9; 원9 NeedsRepair|
|CLI 전체|97명령 발견·schema와 명령별 근거표 존재|명령×구성×정상/오류/취소/저장 인수 전수 종료|전체율 미산정; 발견97은 완료97이 아님|
|GUI 전체|실행파일·Meari/UI 선언 및 일부 공통 API 소비|전체 CLI 선행 인수 후 기본 여정·모든 상태/입력/포커스/접근성 연결 검증|전체율 미산정; 통합 인수 미완료|
|UI/UX 시각 품질|2026-10-18 설계 승인|실제 실행과 기준 시안 요소별1:1 비교·오류/빈/로딩 상태 및 재검토|설계 승인; 실행 시각 인수 미완료|

다음 구현 순서는 원 SDK의 SourceOpener/DownloadSession에 process-backed source를 연결하는 격리 검증 → 원 browser replay/current/attempt 권한 연결 → 전체 CLI 잔여 인수 → GUI 비교/수정/판정/이력 통합 → 묶음 시각 인수다. 원 executor의 저장/CAS/취소 정책을 새 adapter에 복제하지 않는다. root `kinekino_player` library를 재사용할 수 있으나 root와 private의 동일 이름/버전 owned-process path 충돌을 먼저 해소할 격리 입력을 고정해야 한다. 원 WIP와 R36을 덮어쓰지 않는다.

R36은 [원 기록](../../sots/verification/application/source-parent-transfer-io-20261027.md)을 따른다. OS3은 bootstrap 및 Open 미전송/거절/queue 후 close 사례이며 정상 HTTP 성공이 아니다. source375 private overlay·소유권·원 실패와 추론/다운로드/수동GUI/WARP/push 보류를 유지한다. 필수 최소 의존성 설치는 이미 승인됐으나 이번 표 갱신에서 설치·추론·GUI 실행은 하지 않았다. 배포는 목표 밖이다.

## 사용자 재질의 기준 — 제품 목표 유지

검토 가능한 통합 제품의 완료일은 아직 미확정이다. 실행파일 2/2·지정 회귀 190/190은 각각 해당 checkpoint의 성공이며 전체 CLI/GUI 완성률이 아니다. CLI 전체는 명령×구성×정상/오류/취소/저장 인수표, GUI는 화면×상태×입력/포커스/접근성 인수표의 분모 고정이 필요하다. 현재 전체 백분율은 미산정이며 AI 사용자 품질 승인은 0/9, 실제 실행 시각 일치는 미인수다. 배포는 목표 밖이다.

후속 private IPC B는 순수5·OS10 검증을 마쳤으나 제품 root에 채택되지 않았다. 다음 실제 공백은 HTTP source worker/부모 연결과 원 브라우저의 일회 replay 권한이다. private HTTP bootstrap 후보만 작성 중이며 네트워크 전송·다운로드 기능 완료로 계상하지 않는다. 첫 fmt 실패863을 보존하고 형식 수정 후864를 실행했다. 컴파일·strict·시험은 아직 미실행이다. 원 WIP/소유권/source375 private overlay와 추론·다운로드·수동 GUI·WARP·push 보류를 유지한다.

최신 통합 SOURCEDFB0EB90/2244: [고정 입력 통합 회귀](../../sots/verification/application/source375-dfb0-integrated-regression-20261027.md)에서 fmt·기본/Native all-targets strict·no-run·지정등록190/190(Product145/Core18/Native26/OS1)·정상player/player-cli·전체2244 content/identity 동일을 실제 재검증했다. lock/toolchain/font/C-only cache 핀을 확인했고 이번 첫 실패는 없다. private overlay·원 실패/미실행·WIP/소유권·보류 유지.190은 지정 계획의 전체이며 모든Cargo시험/전체CLI/AI품질/GUI 완료를 뜻하지 않는다. 사용자 품질0/9·GUI 시각 인수는 남는다.

동일 SOURCE28B8A1E9/2244 후속: [durable-jobs 실제종단](../../sots/verification/application/durable-jobs-cli-datalab-20261027.md)을 등록190과 별도로 실행했다. 원 host/SDK/CLI28회와 발행 예약 전/후 취소CLI6회(총34), actual2/2·no-run·전체2244 content/identity 동일 통과. 실제 변환/중복충돌/watch/list/retry/NeedsInput/fresh readonly/강제종료 후 Interrupted·pipe/SDK joined를 확인했다. 합성 gate는 발행 예약 순서만 통제하며 전체 executor/모델/네트워크·AI/GUI 인수를 뜻하지 않는다. 소스 변경 없음, 이전190/정상bins는 각 원 입력에 유지한다.

시험 전용 후속 SOURCE8ACEEE6F/2244: [historical 프로젝트 재연결](../../sots/verification/application/historical-project-reconnection-20261027.md)의 실제9/CLI103회·Native 해당test strict·fmt·no-run을 통과했다. verifier 부재=Unsupported, companion/source/candidate 누락·변조6상태=EngineFailed이며 출판 무변경/원본 SHA 보존을 확인했다. 정상 locator revision2는 historical snapshot·과거 revision을 보존한다. 제품 코드 유지,190/정상bins/전체 strict는 SOURCE268E3482의 원 실행이다. 사용자 품질0/9·전체CLI/GUI·기존 보류 유지.

최신 SOURCE268E3482/2244: projects.reconnect_source를 공통 Rust SDK/CLI에 locator-only로 연결했다. 새 실제CLI9·기존프로젝트CLI11/typed inspect·기본/Native strict·fmt·no-run·등록190/190·정상2bins·전체2244 content/identity 동일 통과. [계약·근거·실행 전 디렉터리 첫 실패](../../sots/verification/application/project-locator-reconnection-20261027.md). 과거 revision/제안 snapshot을 보존하며 미디어 내용 검증·재생 rebind는 false다. typed reconnect/전체 실패 행렬·historical 회귀·전체CLI/AI/GUI는 미완료, 품질0/9·기존 보류 유지. 이전 각 checkpoint의 성공은 해당 원 입력의 근거다.

## 현재 사용자 보고 기준

실행 파일은 존재하지만 사용자 검토 가능한 통합 제품은 미완료이며 완료일은 미확정이다. 기능군별 완료는 공통 SDK/CLI의 정상·오류·취소·저장 인수와 GUI 연결 및 해당 시각 인수로 판정한다. CLI 97개는 발견된 명령 수이며 전체 인수 분모가 아니다. GUI 화면×상태 인수 분모도 아직 고정하지 않았으므로 임의 백분율을 제시하지 않는다.

프로젝트 소스 locator 재연결의 제한 인수는 SOURCE268E3482/2244에서 schema·실제 CLI9·기존 프로젝트CLI11·strict·등록190·정상bins·입력동일로 확인했다. typed reconnect 호출·전체 오류 행렬·historical 재연결 회귀는 남는다. 이 재연결은 저장된 경로/URL 선택만 갱신하고 미디어 내용 검증·재생 권한 재결합을 제공하지 않는다. source375 private overlay·기존 WIP는 canonical 적용과 구별한다.

남은 순서는 프로젝트 재연결 SDK/CLI의 CAS·원 snapshot 보존 및 거절 시 무변경 확인 → 전체 명령×구성×시나리오 공백 해소 → GUI 기본 여정과 AI 비교/수정/판정/이력 연결 → 실행 시안 비교다. 사용자 AI 품질 승인0/9, 원 후보9 NeedsRepair를 유지한다. 배포는 목표 밖이며 기존 수동 GUI·추론·다운로드·WARP·push 보류는 유지한다.

최신 SOURCE7144F249/2242: EndpointClient controlled 호출에 전송 전 checkpoint를 연결했다. 실제 pipe의 전송 전 취소=not_sent/server calls0·명시 다음 요청1, 전송 뒤 취소=원 성공/실행1회 보존을 확인했다. LocalHost23(새3 포함)·실행기19·화자6/CLI14·기본/Native strict/fmt/no-run·선택190/190·정상2bins·전체2242 content/identity 동일 통과. [계약/검증](../../sots/verification/application/endpoint-pre-transmission-cancel-20261027.md). 서버 원격 cancel/query/terminal ACK·일반 CLI Ctrl-C는 미완료다. 원본/WIP·소유권·품질0/9·GUI/보류 유지.

최신 SOURCE311E2EE6/2241: 원 PlayerSdk/SDK execution의 화자 read/edit에 cooperative control을 연결했다. 실제 파일의 저장 직전 취소는 output/임시 파일 없이 끝나고, 저장 완료 뒤 늦은 취소는 원 성공·immutable output을 보존함을 새2시험으로 확인했다. 화자6(기존CLI14 포함)·실행기19·기본/Native strict/fmt/no-run·선택190/190·정상2bins·전체2241 content/identity 동일 통과. [계약/검증](../../sots/verification/application/speaker-annotation-execution-cancel-20261027.md). 원 source/lease와 worker ACK/close를 확인했으며 CLI 원격 ACK·GUI·추론/사용자 품질 인수로 확대하지 않는다. 원본/WIP/소유권·보류 유지.

최신 SOURCEEAB91917/2240: 원 PlayerSdk/SDK execution의 models.verify에 cooperative 실행 중 취소를 연결했다. 최종 R2 실행기19·기본/Native strict·no-run·fmt·선택190/190·정상2bins·전체2240 content/identity 동일 통과. R1의 모델CLI4/실제115·실제9 historical CLI92도 원 실행으로 보존하며 R2 재실행으로 계상하지 않는다. [계약/검증·fmt/배정 첫 실패](../../sots/verification/application/model-verification-execution-cancel-20261027.md). nonparticipating client는 원 실제 응답 유지; 별도 wrapper/CLI 원격 ACK·hard preemption은 미지원이다. AI 사용자 품질0/9·GUI/기존 보류 유지.

최신 SOURCEE22215BA/2239: 실제9 historical 시험에 교정·카드·하이라이트의 deferred 판정 저장/재요청/이력과 외부 candidate 누락 시 새 판정 거절·기존 이력 보존을 추가했다. 실제CLI92·기본/Native strict·no-run·선택190/190·정상2bins·fmt·전체2239 content/identity 동일 통과. [검증/원 setup 실패](../../sots/verification/application/historical-output-decisions-20261027.md). 제품 정책 코드는 변경하지 않았으며 시험 저장소의 deferred는 사용자 품질 승인으로 계상하지 않는다. 원 후보9 NeedsRepair·품질0/9·GUI 미완료/보류 유지.

동일 SOURCE667DC07A/2239 후속: 새 호스트에서 실제 텍스트 결과9의 historical import/fresh inspect·세 task 기술적 edit/accept/retry·project capture/reopen·누락/변조/불완전 publication 거절을 실제CLI73으로 검증했다. 원본 전후 SHA와 source2239 content/identity 동일 통과. [새 실제 실행 근거](../../sots/verification/application/actual-nine-cli-datalab-20261027.md). 기술적 fixture 수정은 원 모델 결과의 품질 수선/사용자 승인이 아니며 원 후보9 NeedsRepair·사용자 품질0/9·range=null·GUI 미인수를 유지한다. 추론/다운로드/수동GUI를 실행하지 않았다.

최신 SOURCE667DC07A/2239: subtitles.edit_cues를 기존 proposal 교정 초안·typed client·명시 proposal-root CLI에 연결했다. cue ID/원문/시각/generated를 보존하며 전체 교정 초안을 새 revision으로 교체한다. 새 영향2(실제CLI22)·기존proposal1·기본97schema1·기본/Native strict·no-run·선택190/190·정상2bins·fmt·전체2239입력 content/identity 동일 통과. [계약/검증](../../sots/verification/application/cue-text-editing-20261027.md). 생략 cue는 원문 사용, 새 edit는 기존 acceptance 해제. 시각 편집·ASR 문서 교정·in-flight 취소·AI/GUI 인수로 확대하지 않는다. 앞 입력의 실제 기능 시험은 각 원 checkpoint에 유지한다.

최신 SOURCECAB24961/2237: review 저장소 discovery의 낡은 가정을 수정하고 기본/Native all-targets strict·no-run·두 구성 discovery2시험(실제CLI196)·선택190/190·정상2bins·fmt·전체2237입력 content/identity 동일까지 통과했다. 기본file 구성9supported/88unsupported, 명시review roots 구성18/79이며 그중 review 저장소9supported·재생4unsupported다. [검증과 첫 실패](../../sots/verification/application/review-discovery-boundaries-20261027.md). 미주입 재생을 지원으로 바꾸지 않았다. 수동 annotation/모델 기능의 이전3BBB 실제 회귀를 새 입력 실행으로 승계하지 않으며 전체CLI·AI/GUI는 남는다.

최신 갱신 — SOURCE3BBB37E8/2237: 수동 화자 조회·편집 `subtitles.speakers/edit_speakers`를 공통 Rust SDK와 정상 file CLI에 연결했다. 원본·시각·unknown/overlap·편집 이력을 보존하며 rename/merge/split/배정/삭제/undo/redo를 새 파일 revision으로 기록한다. 영향4·원 handle 해시1·기본 discovery1·모델 회귀4(실제CLI115)·선택190/190·정상2bins·저장CLI3·strict/fmt·전체입력 동일을 통과했다. 기본 구성9supported/88unsupported, model-root 구성12/85이며 97명령 발견은 제품 완료율이 아니다. [수동 화자 검증](../../sots/verification/application/manual-speaker-annotations-20261027.md). 자동 화자 추론/ASR/정렬·실제 AI 품질0/9·GUI 연결/시각 인수는 남는다. 아래 최신 표기는 각 과거 checkpoint의 당시 기록이다.

최신 SOURCE4D4B86FE/2232: models.verify·원 catalog/file guard·크기/SHA/예산/취소를 구현했다. 합성 파일의115실제CLI·97schema·불일치 NeedsInput/일치/중간 취소·주입 Timeout/busy/guard4시험,선택190/정상2bins/strict·저장CLI3/fmt/입력동일 통과. [근거](../../sots/verification/application/model-file-verification-20261027.md). checksum은 시점 관측이며 실제 모델/엔진/권리/품질이나 lifecycle 완료가 아니다.10supported/97은 해당 구성의 발견 결과이며 제품 완성률로 환산하지 않는다.

최신 SOURCE2E9CE8EB/2230에서 모델 카탈로그 list/inspect·공통 SDK/명시 --model-root를 구현했다. 합성 catalog/stub의109실제CLI·구성97 schema/재열기/상태/readonly2시험, 선택190/정상2bins/all-targets strict·저장CLI3/fmt/입력동일 통과. [근거](../../sots/verification/application/model-catalog-reads-20261027.md). 실제 모델 지원·해시/lifecycle·품질 승인·GUI 소비는 남는다.9supported/97은 해당 구성의 발견 결과이며 제품 완성률이 아니다.

최신 SOURCEA2FF8328/2226에서 제품 명시 review startup을 연결했다. 실제 fresh producer·정상2host 저장/재생/EOF/재시작·이전 session 거절/저장 보존과 별도 broken ready_write/os232/exit8·미디어 미생성, 선택190/정상2bins/all-targets strict·CLI3/옵션2/fmt/전후입력 동일 통과. [근거](../../sots/verification/application/review-product-startup-20261027.md). 준비 중 취소/강제crash/cleanup 미완료·전체CLI·AI/GUI는 남으며 이전 startup 미연결 설명은 당시 기록이다. 전체 완성률/납기는 여전히 미확정이다.

최종 확인: SOURCE49C38F20/2221의 기존 저장CLI3·fmt·전체입력 content/identity 동일까지539~541에서 통과했다. 아래 같은 입력의 ‘진행 중’은 종료됐으며 실행파일/선택190 성공과 제품 전체 인수는 구분한다.

최신 SOURCE49C38F20/2221에서 fresh live v2의 동일 SDK 저장·재생 연결을 구현하고 실제CLI40(create1→Verified2→replay/inspect→같은 연결 재생/제어/종료)을 통과했다. 선택190/정상2bins/all-targets strict도 같은 입력에서 통과했다. [근거](../../sots/verification/application/timeline-live-sdk-20261027.md). 최종 저장CLI/fmt/입력동일 확인 진행 중이며 제품 startup·전체CLI·AI/GUI는 남는다. 아래 재점검의 ‘다음 구현’은 이 후속으로 진전했으며 새 다음 공백은 startup/producer 공급과 재시작 입장이다.

## 사용자 납기 질의 후 중앙 PM 재점검

2026-10-27 재점검: 실행 파일은 있으나 사용자 검토 가능한 통합 제품은 아직 미완료다. 내일 완성을 약속할 근거는 없다. 아래 기능군 표를 제품 목표로 유지하고 CLI 97명령 발견·선택 회귀190/190·정상 bins2/2를 제품 전체 완성률로 환산하지 않는다. CLI/GUI 전체 인수 분모와 남은 실제 시나리오별 처리 실측을 확보한 뒤 검토 빌드 예상일을 제시한다. 배포는 범위 밖이다.

코드 재점검에서 `src/infrastructure/review_playback_sessions.rs`가 재생 worker마다 `TrustedTimelineAdmission`으로 archive resolver를 재구성하고, `src/assembly/review_playback/sdk.rs`도 같은 archive 구성에 고정됨을 확인했다. 새 live v2 application 입장 성공과 이 경로의 제품 연결은 구별한다. 다음 구현 배정은 기존 archive 호출자와 WIP를 보존하면서 같은 live resolver를 SDK 저장·재생 조립에 공급하고, 새 빈 user root의 생성→Verified 저장→동일 요청 replay→재열기→같은 연결 재생을 공통 SDK/CLI에서 검증하는 것이다. 이 항목은 아직 구현·실행 전이다.

후속 순서는 제품 startup 연결과 실패/재시작 → 전체 CLI 인수 공백 해소 → GUI 기본 여정·AI 비교/수정/판정/이력 연결 → 실행 시각 비교다. 기존 수동 GUI·추론·다운로드·WARP·push 보류는 이번 납기 질의로 해제되지 않는다. AI 품질 승인은 실제 제품 GUI에서 사용자가 수행하며 현재 승인0/9를 유지한다.

최신 통합 SOURCE1E2B3767/2220: fresh live publisher→v2 raw/trace→기존 application evidence 입장과 정책26/strict·선택190·정상2bins·기존 저장CLI3·fmt 통과. [근거](../../sots/verification/application/source375-timeline-live-reader-regression-20261027.md). 첫 ID 실패는 보존했으며 새 proof의 SDK/정상CLI 저장·재생·제품 연결·전체CLI/AI/GUI는 남는다. 이전 ‘최신’ 설명은 각 당시 기록이다.

최신 영향 SOURCEDA8C42F7/2216: 로컬 새 package 버전 발행/재열기·원 모델 결과/판정 보존·취소 미출판·같은 package replay와 정책23/strict/no-run/fmt 통과. [근거](../../sots/verification/application/timeline-package-publication-20261027.md). commit 기록은 Verified 권위가 아니며 v2 reader/입장·SDK/CLI·제품 연결·AI/GUI는 남는다. 최신 전체190/정상bins는 B1B46C92다.

최신 영향 SOURCE53105D42/2211: 실제 관측 동시 보유·결합 mapping·잘못된 끝점 결합 거절·쓰기 차단과 공통22/strict/no-run/fmt를 확인했다. [근거](../../sots/verification/application/timeline-live-bundle-20261027.md). 결합 record는 custody/Verified 권위가 아니며 package 새 버전 발행·제품 연결·전체CLI/AI/GUI는 남는다. 전체190/정상bins의 최신 통합은 B1B46C92다.

최신 영향 SOURCEB1FE4AA4/2210: 실제 origin·17초/25.5초 source PTS/PNG·취소 정리, 정책22/frame_capture6/strict/no-run/fmt 및 PCM계보 회귀 통과. [근거](../../sots/verification/application/timeline-capture-production-20261027.md). 결합 proof·custody/admission·제품 연결·전체CLI/AI/GUI는 남고 최신 전체190/정상bins는 B1B46C92 입력이다.

최신 통합 SOURCEB1B46C92/2208: 실제 원본/재생 PCM 전체 바이트 일치·clip 계보/취소와 정책21·strict·선택190·정상2bins·저장CLI3·fmt를 검증했다. [근거](../../sots/verification/application/source375-timeline-lineage-regression-20261027.md). origin/캡처 결합 issuer·제품 연결·전체CLI/AI/GUI는 남는다. 아래 ‘최신’ 표기는 각 과거 기록의 당시 상태다.

최신 영향 SOURCEC06BD8DF/2207: 원본 PCM10078204 samples와 같은 실행의 clock/바이트 수 일치·실제 생성 후 취소 정리·영상/clock 회귀·정책19/strict/no-run/fmt 통과. [근거](../../sots/verification/application/timeline-pcm-production-20261027.md). 재생/클립 계보·전체 issuer·제품 연결·AI/GUI는 남으며 최신 전체190/정상bins는 C27 입력이다.

최신 영향 SOURCECAFFCBBA/2206: 공통 producer 실행 수명과 오디오 clock을 추가해 영상240프레임/오디오27128프레임·각 생성 후 취소/정리·정책18/strict/no-run/fmt를 검증했다. [근거](../../sots/verification/application/timeline-audio-production-20261027.md). PCM 계보와 전체 issuer는 미완료, 최신 전체190/정상bins는 아래 C27 입력이다.

최신 통합 SOURCEC27B77C0/2204: 선택190(Product145/Core18/Native26/자동OS1)·정상2bins·저장CLI3·fmt 통과. [후계 통합 근거](../../sots/verification/application/source375-timeline-production-regression-20261027.md). 영상 producer의 실제 관측/취소에 더한 회귀이며 전체CLI·AI품질·GUI 완료율은 아니다. 아래9F 및 ‘재실행 전’은 과거 시점 설명이다.

최신 영향 SOURCEC27B77C0/2204: 원본 결합 Rust 영상 producer를 추가하고 정책18통과/5ignored·strict/no-run·실제240프레임 관측·child 생성 후 취소/원 Job 정리·실행 슬롯 재예약을 검증했다. [근거](../../sots/verification/application/timeline-video-production-20261027.md). 영상 관측 한 단계이며 Verified issuer·제품 startup·AI 품질·GUI 인수는 아니다. 변경 후 전체190/정상bins는 아직 재실행하지 않았다.

최신 영향 DC5F6B85/2199: 공통 host I/O driver의24정책/strict/fmt/no-run·정상EOF/재시작·실제BrokenPipe에서 host오류8/미디어정상정리·전후입력동일 통과. [근거](../../sots/verification/application/review-stdio-20261027.md). 제품 serve startup/새producer 공급은 남고 아래9F602A06이 마지막190/정상bins 통합이다. 오류 처리 시험의 통과를 host 정상 성공으로 계상하지 않는다.

현재 통합 기준 SOURCE9F602A06/2197: 동일 구성 SDK의 새 빈 user root에서 CLI Verified 저장→그 연결 재생을40process로 검증했다. 공통21/strict/fmt/no-run·readonly source 별도OS2host EOF/재시작·선택190/정상2bins/저장CLI3·전후2197파일 동일 통과. [근거](../../sots/verification/application/source375-review-configured-sdk-regression-20261027.md). 제품 serve startup·제품 producer의 trusted admission 공급·crash/IO실패 전수·전체CLI/AI/GUI는 남는다. 아래CE23/E28은 이전 기록이다.

후속 시험 SOURCECE23E8A8/2195에서 별도OS host2/CLI20/미디어child2의 EOF 정상 종료·재시작·이전참조 거절·strict/fmt/no-run·전후입력동일 통과. [근거](../../sots/verification/application/review-process-host-20261027.md). 제품CLI serve설정/승인 입력 공급·crash/입출력 실패 전수는 남는다. 제품 소스 변경 없이 시험 fixture3파일을 확장했으며 아래E28 전체 통합 근거를 새 입력 실행으로 승계하지 않는다.

현재 기준 SOURCEE28CA06B/2194: 공통19/strict/fmt/no-run·실제CLI35·pipe/sdk Joined/3owner 물리완료·선택190/정상2bins/저장CLI3·전후입력동일 통과. [근거](../../sots/verification/application/source375-review-transport-regression-20261027.md). 제품 CLI serve설정·trusted admission·별도 프로세스 재시작·전체CLI/AI/GUI는 남는다. 아래EB63/3940/2088 설명은 각 이전 checkpoint의 역사다.

최신 영향 SOURCEEB637CA2/2192의 host 전체 종료는 공통17/strict/fmt/no-run·실제1/원 child2직렬·3owner 물리 완료·전후2192입력동일을 통과했다. [근거](../../sots/verification/application/review-host-shutdown-20261027.md). 제품 기본 CLI host 설정·trusted admission·transport 종료 연결/별도 프로세스 재시작은 남는다. 아래2088의190/정상bins는 변경 전 전체 통합 checkpoint다.

최신 영향은SOURCE394039FA/2190의 공통 host 조립이다. 실제CLI35·새 host의 scope 변경/ordinal 재사용/이전 참조 상호 거절·Unverified child미생성·정책14·strict/fmt/no-run·전후입력동일을 통과했다. [근거](../../sots/verification/application/review-host-assembly-20261027.md). 기본 제품 host 자동 연결과 별도 프로세스 재시작 전수는 남는다. 아래2088의190/정상bins는 변경 전 전체 통합 checkpoint다.

## 진행 갱신 — 세션 제어

공개 SessionRef/engine instance에 결합된 제어와 관측 clock 기반 coverage 경계 일시정지를 구현했다. 새 SOURCE20886037/2188에서 정책/세션14·worker9·strict/fmt/no-run·실제 CLI35프로세스·discovery97·선택190·정상 player/player-cli·저장CLI3·전후2188파일 content/identity 동일이 통과했다. [통합 검증](../../sots/verification/application/source375-review-control-regression-20261027.md). 경계 일시정지는 polling 관측 정책이며 프레임 단위 강제 정지나 AI 품질 근거가 아니다. 이전 FECF checkpoint와 원 실패는 보존한다.

중앙 PM. 배포는 제외한다. [현재 현황](product-readiness-latest-20261026.md)과 [전체 제품 조건](full-product-integration-gates.md)의 증거를 요약한다. 기존 담당 소유권·WIP·원 실패·보류는 유지한다. 소요시간 실측과 전체 기능 인수 분모가 없어 제품 전체 백분율과 날짜는 아직 확정할 수 없다. 실행 파일 생성은 사용자 검토 가능한 제품의 완성과 구별한다.

| 목표 | 지금 구현·확인한 범위 | 남은 구현·검증 | 목표 기준 완성도 |
|---|---|---|---|
| C-only 빌드 환경 | NVMe C의 native/meari-native/resource-runner/tmp 분리 | 후속 변경도 동일 정책 적용 | 경로 정책 적용 완료 |
| 실행 파일 | SOURCEDFB0EB90 입력에서 정상 player/player-cli 빌드 확인 | 후속 기능 변경 시 재빌드 | 해당 checkpoint 2/2, 제품 완료율 아님 |
| 통합 회귀 | SOURCEDFB0EB90에서 지정 Product145/Core18/Native26/OS1 | 후속 기능 변경 시 영향·통합 회귀 | 해당 checkpoint 190/190, 전체 제품 인수 아님 |
| 카탈로그·제공자·재생·스킵 | 공통 SDK/CLI와 과거 실제 종단 근거 | 최신 구성의 오류·취소·복구·자동 감지·광고차단 인수 | 부분 구현·부분 인수 |
| 프로젝트·자막·작업·제안 | 저장/CAS/수정/채택/재열기·화자 편집의 저장 전 취소/저장 후 성공 보존·교정 초안 편집, 일부 실제 IPC | 기본 제품 여정 연결·작업별 실제 인수 | 부분 구현·부분 인수 |
| 원본 연결·범위 초안 | custody·사용자 저장·replay/CAS·복구 및 SDK/CLI 5명령·제품 host Verified 저장 | RD 잔여 행렬·실제9 귀속·GUI 연결 | 명령 연결 5/5, 기능군 전체 인수 미완료 |
| 원본 재생·시간축 | fresh live v2·동일 SDK CLI40와 제품 명시 startup의 저장/재생/EOF/재시작·ready 출력 실패 | 준비 중 취소·강제crash/cleanup 미완료·추가 IO실패·전체 여정 | 명시 제품 시작 경로 실제 인수, 전체 오류 인수 미완료 |
| 다운로드·제작 | 계약·명령 표면·일부 독립 실험 | executor별 실제 출력·취소·충돌·부분결과·재시작 | 부분, 제품 종단 인수 미완료 |
| AI 결과 검토 | 실제9 import/수정/채택·세 task deferred 판정/이력·증거 누락 거절 CLI92·검토 표면 | 실제 모델 IO/취소·GUI 원본/후보 비교·재생·수정·명시 판정·이력 | 실제9 표본 모두 NeedsRepair, 사용자 품질 승인 0/9 |
| 모델 관리 | 공통 list/inspect/verify·fixed catalog/manifest·파일 시점 크기/SHA·원 SDK 실행기 cooperative 취소/오류 정리 | 실제 모델/엔진/lifecycle·install/remove·타 target/GUI·remote 취소 ACK | 3명령의 제한 인수, 모델 관리 전체 미완료 |
| CLI 전체 | 고유97명령 발견, review playback4의 실제 구성 host CLI35process 등 일부 인수 | 명령×구성×성공/오류/취소/저장 및 비적용 사유 고정·실행 | 전체 % 미산정, 97은 발견 분모 |
| GUI 전체 | Meari/typed host·일부 화면·수명·재열기 | 최신 SDK 연결·기본 여정·오류·입력/포커스/접근성 | 전체 % 미산정, 화면×상태 인수 미완료 |
| UI/UX | 10/18 설계 승인·공통 자산/컴포넌트 | 기준 시안 요소별 확대·1:1 대응·실제 실행 비교 | 설계 승인 완료, 실행 시각 일치 미인수 |

## 사용자 검토 빌드까지의 순서

현재 우선순위는 **전체 CLI의 host별 명령/시나리오 근거 매핑·미구현 executor와 잔여 실패/취소 인수 → GUI/AI 검토 연결·시각 비교**다. fresh proof의 동일 SDK 저장·재생에 더해 SOURCEA2FF8328에서 명시 제품 startup/재시작·ready 출력 실패를 실제 인수했다. 아래1~2는 이전 checkpoint 당시 순서를 보존한 기록이며, 현재 새 작업으로 반복 배정하지 않는다. 개발 검증이 끝난 검토 빌드가 준비된 뒤 실제 제품 GUI에서 사용자 품질 판정을 받는다. 전체 분모와 잔여 처리 실측이 확보되기 전 날짜나 통합 완성률을 추정하지 않는다.

1. Verified 저장 기반은 SOURCE6381DE6F의 실제 봉인 SDK 저장/재열기 및190/정상bins/CLI3/전후2176입력 동일까지 검증했다. 호출자가 제출한 요약 JSON은 권한이 아니며 저장 기록만으로 playback_ready를 true로 만들지 않는다. Verified 정상CLI 성공과 frame 실제 관측은 남는다.
2. 공개 SDK command/schema와 구성 host의 정상CLI18process 연결을 FECF 입력에서 검증했다. **다음 구현:** 제품 기본 host/fresh scope 공급·source-bound 제어와 실패/취소/재시작 인수. 같은 입력의 최신190/정상bins/전후2183입력동일도 통과했다.
3. 전체 CLI 추적표의 미구현 executor와 실제 시나리오 공백을 닫는다. 각 기능은 구현·검증·비적용을 분리한다.
4. 새 입력 고정 → 공통 Rust SDK/CLI 영향 검사 → 선택190 → 정상 bins. 과거 checkpoint의 성공을 새 입력으로 승계하지 않는다.
5. CLI 선행 인수 후 GUI 기본 여정과 AI 원본/후보 비교·수정·판정·이력을 연결하고 시안 대조를 수행한다. 실제 제품 GUI의 사용자 품질 판정이 AI 최종 승인이다.

납기 답변은 현재 **실행 파일은 존재, 사용자 검토 가능한 통합 제품은 미완료, 완료일 미확정**이다. 1~3의 잔여 시나리오 수와 처리 실측을 먼저 확보한 뒤 검토 빌드 예상일을 갱신한다. 예전 12시간 목표를 전체 제품 납기로 재사용하지 않는다.

수동 GUI·추론·다운로드·WARP 변경·push 보류를 임의 해제하지 않는다. 기존 modoo.today software-qa/code-change-review-verification 및 test-fixture-and-mock-build의 주장/실제 근거·합성/실제 구분을 적용했다. 원 증거와 상세 pin은 링크한 정본이 소유한다.
