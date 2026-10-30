---
id: plan:kinekino/20261029113738
title: 설계 완성 감사 — 요구사항별 대조
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 설계 완성 감사 — 요구사항별 대조

2026-10-26. 완료를 가정하지 않고 R01~R40·G01~G16 및 사용자 지시의 산출물을 현재 파일과 대조한다. 이 감사의 대상은 **기존 전체 요구를 유지한 설계 계약·책임·통합 경계·인수 기준·의존성 계획**이다. 구현·기능 시험·정상 제품 bins·사용자 GUI/품질은 미완료로 유지한다. 근거가 소스/기존실측/계획 중 무엇인지 [추적표](requirements.md)에 표시했다.

## 검토한 기준과 해석

원 최종handoff/감사5/receipt가 요구한 소유권·pin·실패/미검증을 유지하고, 현재 host의 사용자 결정 C-NVMe를 반영한다. 설계 문서는 지원되지 않는 기능도 요구에서 제거하지 않고 입력/출력·담당·지원 조건·오류/취소/저장/복구·인수 gate를 정의해야 한다. 모델/도구/미디어가 없는 현재 환경의 실행 차단은 NeedsInput/Unsupported의 설계이며 최종 기능 요구 삭제가 아니다. 사용자 권한·모델 품질·재배포 권리를 임의 승인하지 않는다.

아래 약어: SB=[SDK host/명령](../../../sots/player/application/api/host-support-boundaries.md), ES=[실행/저장](../../../sots/player/application/execution-storage-boundaries.md), RD=[association/draft](../../../sots/player/application/api/review-source-draft-contract.md), TI=[작업별 통합](../../../sots/player/application/production/task-integration-contract.md), UI=[UI/Meari](../../../sots/player/presentation/shared/product-integration-acceptance.md), RE=[배포/신뢰](../../../sots/player/architecture/release-and-trust-boundaries.md), VP=[이어보기](../../../sots/player/application/viewing-preferences-and-resume.md), ORDER=[구현순서](implementation-order.md). 기존 기능별 세부 정본은 추적표의 링크를 유지한다.

## 요구사항 감사

| 요구 | 현재 설계 근거와 충족한 기준 | 구현·실행의 남은 범위 |
|---|---|---|
| R01 | ProductIdentity 소스와 기존 identity 정정·RE03/04의 stable ID/설정migration | 전체 표시/installer/publisher/upgrade 실제 인수 |
| R02 | SDK/계층 방향·SB host·ES owner·RE legacy 책임 | 초기 direct adapter의 Rust 이관·전체기능 동등성 |
| R03 | SB의 명령군/port/context·schema/버전/오류·SB01~04 | source375의 실제 capabilities/등록/CLI 인수 |
| R04 | ES 상태/queued-vs-running cancel/ACK/join/unknown·EX01~06 | timeout 원인·실제worker/native 정리·취소경쟁 |
| R05 | ES Job/attempt, TI 원input/검증output·SB read/write/retry | task별 실제 worker·restart/partial/output 인수 |
| R06 | ES store/CAS·RE migration, project API 정본 | 프로젝트 모든경로·손상/재열기 실제시험 |
| R07 | ES store구분·SB proposal/capture/output-review context | 각 task default route·실저장/채택/재시도 |
| R08 | UI07·품질정본의 실제 GUI 사용자 verdict/이력 | 실제표본/정상GUI·사용자 품질승인0 |
| R09 | catalog정본+SB read/provider·TI source범위·RE URL boundary | 전체검색/필터/제공자·새호스트 실제3분류 |
| R10 | provider/resolve정본·TI source/track/만료·RE01/05 | 전체provider·인증/DRM·실 source 동일성 |
| R11 | adblock정본3층·RE fixed DOM adapter/remote code거절 | iframe/redirect/popup·전체커버리지·대체동등성 |
| R12 | native정본·SB session/control/관측미확정·ES 시간분리 | 실제각engine/track/audio/seek/close |
| R13 | episode transition정본·ES retire/commit·UI watch state | 자동진행·경계/오류·실제새host 왕복 |
| R14 | UI04/06 native parent/child/geometry/focus/close | 실제창/DOM·IME/청취/fullscreen·시각 |
| R15 | skip/host정본·SB export capability·TI detect와수동분리 | 자동detect/music·inflight cancel·실제N/V/O |
| R16 | VP01~08의 intent/관측·source 재해석·CAS·restore | durable preferences/resume API/CLI 및 실제재개 |
| R17 | subtitle/range정본·SB storage/loss·TI01~12 | ASS/image/트랙/사용자흐름 전체지원 |
| R18 | ASR/VAD정본·TI task pin/sample/time mapping·admission | 제품executor/dtype/성능·실제음성/품질 |
| R19 | speaker domain/pipeline·TI overlap/cluster/revision/loss | diarize/alignment/편집·DER/JER/실제저장 |
| R20 | OCR정본·TI ROI/표본경계/읽기vs제거 | 제품executor·시간/품질·cancel/reopen |
| R21 | TI raw와typed후보·cue근거/구조오류·UI comparison | repair/accept/SDK 연결·모델/사용자품질 |
| R22 | RD 전체 typed/독립association+결합draft/단일envelope·RD01~12 | durable/DTO/API/CLI·Verified 실제PTS·GUI |
| R23 | model manifest·TI installed/verified/admitted/loaded/quality분리 | generic model manager/task admission/삭제참조/설치실행 |
| R24 | retrieval정본·TI index/source/model revision·stale/delete/cancel | 실제index/embedding/검색SDK/자원·품질 |
| R25 | download정본·기존owned executor 소스·TI 재개/HLS/nooverwrite | host등록/트랙·재시작/실미디어/오류행렬 |
| R26 | snapshot/capture정본·TI content/PTS/color/source연속 | 전체편집·HDR/VFR/시간정확도/실GUI |
| R27 | cards/card-review정본·TI 단일PNG와전체편집/batch분리 | 전체문구/layout/undo/묶음·glyph/품질 |
| R28 | shorts정본·TI 동일renderer/clip/batch/output검증 | renderexecutor·정확컷/A-V sync·전체CLI |
| R29 | subjects/OBJ정본·TI frame/mask/track/review·원identity | 모델/수정·추적실패/재사용·제품CLI |
| R30 | projection/A2D정본·TI coord/alpha/depth/UV/manifest·연구분리 | mask/depth/패키지executor·실대상roundtrip |
| R31 | preset/acceptance정본·TI output·RE version/license | 실제encoder/출력/플랫폼권리·외부게시별도 |
| R32 | 승인UI/element정본·UI 23route×상태/프레임·UI01~07 | 모든요소 실제renderer/키보드/시각·접근성 |
| R33 | UI props→producer→callback→SDK semantic trace·strict거절 | source375 실제factory/후단feedback·190 |
| R34 | Meari Router 원행렬+UI route/history/dirty/cancel | upstream API 전체구현/target trace·실브라우저 |
| R35 | Meari plugin SDK/SOT+제품plugin정본·UI grant/version/HMR | 실제native/Web bridge/OS·cancel/cleanup |
| R36 | Meari Q01~09 원정본+UI target별 compiler/runtime gate | OS1/r11·Web/WASM/mixed·전체framework |
| R37 | asset/font정본·UI asset contract·RE closure·대체전보존 | glyph/shaping/fallback·실render/권리closure |
| R38 | RE trust table/negative cases·공개분리·RE01/02/05/06 | bundle/content/secret·각boundary 실제검증 |
| R39 | host ADR·ORDER B0~B3 exact375/190/custody/producer | 제품source stage/정상build/190/adoption 미실행 |
| R40 | 기존문서 최소정정·책임별SOT/plan/evidence·명시링크검증 | 전체과거문서/외부웹 전수감사는이번범위아님 |

## G01~G16의 설계상 해결과 이행 분리

G01 ORDER B0~B3, G02 원상태를보존한최소정정+이문서, G03 SB, G04/05 ES/VP, G06 TI, G07 RD, G08 TI/model정본, G09 TI/VP/RE, G10 UI, G11 UI+Meari 소유SOT, G12 UI/RE, G13 RE, G14 UI07, G15 RE legacy표, G16 추적표+이감사로대응한다. 각G에명시한실행/구현미완료는위표의이행범위로남는다. 미구현을지원완료로정정하거나모델선정·권리를임의확정하지않는다.

## 사용자 지시 산출물/불변성 감사

| 기준 | 현재 증거/판정 |
|---|---|
| goal등록·중앙PM·소유권 | create_goal 성공/active, README와각정본의소유경계. 새세션/agent/타세션메시지0 |
| 전체요구추적·공백 | requirements R01~40/gaps G01~16, 이표의책임/계약/인수연결. feature/test완료개수아님 |
| API/SDK·CLI우선/수명/저장 | SB/ES/RD/VP, GUI/CLI 동일usecase와명령별실제context차이 |
| UI/Router/plugin·상태 | UI 23route 연결, 기존승인197요소/정본유지, fullMeari범위축소0 |
| 모델·미디어·품질 | TI/RD/RE, actual9 NeedsRepair/null·cue≠PTS·품질승인분리 |
| 구현·검증순서 | ORDER D/B/F/U/P, exact375/190/정상bins 계획·실행구분 |
| WIP/index/보류 | goal간git status/index·Mearidocs2확인, public제품변경0·stage/commit/push0·원source375/190 SHA불변. 새코드는private Rust문서도구뿐 |
| 경로·언어 | C NVMe privatecache·문맥별target/TMP/TEMP. 새도구std-onlyRust·계층분리, 전역설정/비Rust도구추가0 |
| 검증범위 | private 도구 fmt/test5/strict Clippy/build 통과. 첫 명시11신규문서110 local links pass. 기존11문서는119 local pass+기존1anchor strict unsupported로 보존한 뒤, unique plain ASCII heading 검사와 missing/duplicate/fenced/complex 음성회귀를 추가했다. 최종 명시문서 검사는 [검증 기록](../../../evidence/reviews/design-completion-verification-20261026.md)을 따른다. 원 spec-maintenance build101/anyhow 부재 보존·다운로드0 |

## 현재 판정

감사표는 설계 기준선의 각 요구에 책임·기존 근거·계약과 실패/취소/저장/복구 인수를 연결했다. 실행 미검증을 설계 공백으로 숨기거나 반대로 설계를 구현 완료로 계상하지 않는다. [최종 검증](../../../evidence/reviews/design-completion-verification-20261026.md)의 명시28문서259 local links/지원anchor 실패0, private Rust tool 시험5/fmt/strict/build와 WIP/선택pin 불변을 확인했다. **설계 goal 완료 조건 충족**으로 판정한다. 제품·라이브러리 구현/실제 CLI·OS/Web·사용자 품질은 위의 남은 범위로 유지하며 새 실행 권한을 만들지 않는다.
