---
id: plan:kinekino/20261029113737
title: 의존성 순 구현·검증 계획
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 의존성 순 구현·검증 계획

2026-10-26 중앙 PM. 본 문서는 실행 준비/담당/입장·출구 기준이며 실행 receipt가 아니다. 전체 요구 R01~R40을 유지한다. 다음 계획 때문에 WARP/추론/다운로드/수동 GUI/Pages/VM/DPI/push 보류를 해제하지 않는다. product375/190/정상 bins는 설계 goal 동안 미실행 상태로 유지한다. 시간 추정/새 deadline은 측정 없이 확정하지 않는다. 과거12시간 목표 미달을 새 설계 완료로 달성 처리하지 않는다.

## 실행 책임과 gate

| 단계 | 담당 / 선행 | 수행 범위 | 입장·종료 근거 |
|---|---|---|---|
| D0 요구·결정 기준선 | PM / 사용자 인수 | 최종handoff/감사5/receipt·AGENTS·40요구군·16공백·소유권/보류 고정 | 요구→정본→구현/지원조건→인수 ID 연결. 문서/소스/실제/사용자 품질 분리 |
| D1 계약 정밀화 | SDK/Media/UI/Meari 소유, PM 정규화 / D0 | host/schema/context·실행/저장·task별IO·association/draft·UI/router/plugin·release/권리/legacy | 모호한 정책/중복·지원과계획 혼합 해소. 전체원본 보존·명령/화면별 계획 인수조건 |
| D2 설계 closure 감사 | PM / D1 | 하위 요구·문서링크·결정·상태·남은질문·이행가능한 인수 계획 검토 | 각 R/G에 증거와 미완료 분류. 불명확한 외부 조건은 명시 NeedsInput/Unsupported 계약 및 필요한 입력. 구현 미완료를 설계완료로 숨기지 않음 |
| B0 새호스트 custody/preimage | PM/SDK / D2 및 실제 빌드 배정 | 아래 입력·경로·도구체인·stage·producer 소유 고정 | 원 manifest/hash 보존·375 unique/content·190 exact명·HEAD/index/WIP·원복설계·예산·writer 없음 |
| B1 고정 stage 정적/컴파일 | SDK, Meari 지원 / B0 | root/Meari fmt→type→strict Clippy→no-run JSON producer | 원 feature/package 조합 유지·실제 출력 C 경로·원 첫실패 로그. assertion/guard/목록 축소0 |
| B2 등록190·실시험 | SDK/PM, Meari OS 소유 / B1 | 실제 --list와 exact190 대조→Product145/Core18/Native26/OS1 | 같은 pin/producer. 그룹 실행 중 Cargo writer0·전후hash/identity. pass/firstfail/미실행 구분 |
| B3 정상 bins·후보 채택 | PM/SDK/독립Meari담당 / B2 | 동일pin player/player-cli 정상build/provenance→정확 경로별 adoption 검토 | 정상실행파일과libtest 분리. source375 canonical 적용은 검증된맥락별/명시stage·소유검수 후. WIP/appmod wholefile 흡수0·push0 |
| F1 저장·source bridge | SDK/Media / D1, B0~B3 통합기반 | CaseResolver/기존asset custody/MediaLease·association+drafstore·DTO/CLI·기존decision 보존 | RD01~RD10 실제 저장/재시작·EX/ST 경계. Verified 생성 없는 기능은 명시 Unverified/unsupported. 새 registry/원9덮기0 |
| F2 operation별 SDK/CLI | SDK/Media / D1, 기존유스케이스+F1 관련분만 | 자막/재생/provider/skip/download/cards/shorts/subjects/assets/projection/model IO별 missing wiring/기능 구현 | SB/TI/EX/ST·feature별 fmt/test/strict 및 실제 CLI 성공/오류/취소/저장. partial executor를 전체 제품완료로 계상하지 않음 |
| F3 미디어 시간·모델별 실측 | SDK/Media / F1/F2 및 별도 실행 배정 | 원 content/track·bounded PTS/timebase→SDK seek/capture, pinned task 모델/자원/품질 실측 | RD11/TI11·권리/admission·실제 original/derived/evidence pin·종료. inference/download/장시간 보류 해제 없이 실행0 |
| U1 얇은 UI 연결 | UI/Meari/SDK / 해당기능 CLI 인수 | 동일 source 선언→props→owner callback→SDK feedback·정상renderer | UI01~UI04. 라우터/plugin/runtime은 별도 공통 계약 인수, UI정책 복제0 |
| U2 UI 고유 묶음 인수 | UI/Meari/PM / 전체 CLI 인수·정상 bins 및 별도 수동 배정 | 공통 시안 요소 확대1:1·입력/focus/IME/a11y/반응형·Native/Web 차이 | UI05/UI06, 같은 source/build/상태. GUI로 CAS/변환 기능 반복 시험이나 CLI 미지원 우회0 |
| U3 사용자 AI 품질 | 사용자, Media/UI 준비 / 실제GUI+표본준비 | 원본/후보·필요재생/수정·명시verdict/이력 | UI07만 사용자승인으로계상. 그전은개발미완료이며사용자대기를개발차단으로쓰지않음 |
| P 배포/이관 | PM/각소유 / 관련기능·권리·user인수 | 버전/NOTICE/모델·font/native closure·package·config migration | [release contract](../../../sots/player/architecture/release-and-trust-boundaries.md)의 RE gates. publish/push/외부업로드는 별도 권한 |

F2의 독립 저자원 소스 작업은 F1이나 다른 모델 선정 때문에 전부 직렬 정지하지 않는다. 같은 공유 입력/producer를 읽고 쓰는 gate만 중앙 슬롯으로 직렬 배정한다. Meari Q01–Q09의 library 개발은 제품 CLI 전체완료를 선행으로 강제하지 않되 제품 수동 GUI 인수는 SDK/CLI 우선을 유지한다. 자동 OS1과 수동 시각 인수도 서로 다른 범위다. OS1 계획을 수동 GUI 재개 허가로 사용하지 않는다.

## B0 새호스트 입력 고정 상세

- root d13cbfca…/Meari0844f6c2… 및 dirty/untracked/index를 실제 시작 직전에 캡처한다. source375 A09B51AC…와 exact190 082C68B4… 원본은 수정하지 않는다. Media B47E9913…도 별도 기능 source로 유지한다.
- manifest의 source slash를 정규화해 old D root→대상 C root로 custody 매핑한다. old C font/license5는 canonical Meari 폰트/라이선스의 동일SHA로 해소한다. 원 절대경로·원FileID는 역사로 보존하고 대상 위치·bytes/hash/fileidentity·preimage/absence를 별도 발행한다.
- 기존 source375 selected source와 canonical diff를 path별로 고정한다. 과거 union/branch/pending/WIP3를 자동 overlay하지 않는다. root HEAD만 빌드한 것을 선택375 검증으로 부르지 않는다. 한 통합 stage에서 원본/부재를 보존하고 단계 종료 후 정확 복원한다. 새 source stage는 원 restore자료·역사 manifest가 아닌 명시 새 목적지여야 한다.
- root/Meari Cargo.lock·필수 local dependencies/fonts/assets·toolchain exact release/commit/host·MSVC/linker/windows SDK의 실제 사용 경로·환경을 고정한다. 현재 관측 rustc/cargo1.99.1은 준비 근거이고 동일 source 정상빌드 성공이 아니다. --locked --offline 기본, missing dependency이면 정확히 기록하고 다운로드 보류를 우회하지 않는다.
- <원 호스트 저장소>/runtime/build-cache/{native,meari-native,resource-runner,tmp}의 workspace 문맥을 사용한다. 별도문서도구는 충돌 없는 전용 tool namespace를 같은cache루트에 둔다. launcher의 CARGO_TARGET_DIR·--target-dir·TMP/TEMP와 실제 JSON target/build directory가 일치해야 한다. --manifest-path만으로 workspace config가 바뀐다고 가정하지 않는다.
- Cargo가 보고한 원산출물 EXE/PDB/package/features/fresh/source pin/bytes/hash/FileID/mtime을 등록한다. 모든 통과 산출물의 독립 복사를 반복하지 않는다. 실패 증거/필요재현파일은 예산 안에서 정확히 보존한다. group 시작/종료 hash·매실행 metadata와 writer/stream 반환을 기록한다.

## 인수 ID 연결

R01~R08/R39~R40 → SB01~07·EX01~06·ST01~07·RE01~08. R09~R16 → SB/TI 및기능별native/provider/skip원gate. R17~R31 → task별 TI01~12, R22는 RD01~12 추가. R32~R37 → UI01~07 및 Meari 공식행렬/정확target검증, R38 → RE/신뢰경계. 적용불가cell은이유와대체검증을기능정본에기록하며전체검사면제범위로확대하지않는다.

## 현재 실행 상태

D0 초기표/공백과 D1 통합계약 작성 진행 중. B0~P는 계획이며이goal에서실행0. 제품의기존성공·실패·미실행은최종handoff범위대로보존한다. 설계goal 종료판정은 D2 completion audit로만 하고 B/F/U/P의 구현완료를허위로요구충족처리하지않는다. task 계약상 남은세부모델/encoder/정밀도는 기존pin과capability/admission값으로명시하고 임의선정하지않는다.
