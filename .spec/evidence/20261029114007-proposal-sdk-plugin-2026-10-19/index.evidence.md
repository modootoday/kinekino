---
id: evidence:kinekino/20261029114007
title: 제안 실제9의 제품 plugin·공유 SDK 검사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제안 실제9의 제품 plugin·공유 SDK 검사

2026-10-19. SDK의237pass/17ignored checkpoint와 source freeze를 인계받아 root 정본 DTO/Client를 소비했다. [독립 plugin 검사](proposal-plugin-preflight-2026-10-19.md)와 이번 root 통합 범위를 구별한다. 새 추론·사람 품질 verdict·자동 적용은 없다.

## 실제 연결과 명령

제품 `TypedSdkPlugin`은 Meari registry/grant/lifetime을 재사용하고 typed input/output·ApiError codec만 제공한다. 공유 `src/assembly/meari/proposals.rs`가 canonical create/inspect/edit/accept DTO와 실제 ApplicationClient를 주입한다. root 역의존/DTO 복제·SDK 저장/CAS 정책 복제·root 제품 Cargo/모듈 등록은 하지 않았다. 독립 probe와 CLI가 같은 host source를 컴파일했다.

manifest는 tools/player-ui-sdk-bridge/Cargo.toml이다. BelowNormal/-j1/공유 cache/offline을 사용했다. 직접 serde dependency 반영은 offline workspace lock sync(새 registry package0) 뒤 locked로 검사했다.

| 검사 | 실제 결과 |
| --- | --- |
| fmt --check | exit0 |
| test --offline --locked -j1 | exit0,기존4+신규task3=7회귀,compile1분15초 |
| 신규3 회귀의 실제 표본 | correction/cards/highlights × Gemma/Llama/Qwen =9,2.34초 |
| strict all-target Clippy | exit0,10.09초 |
| build --offline --locked -j1 | exit0,0.99초 |

정상 프로세스를 유지했고 검증된 독립 plugin6을 불필요하게 반복하지 않았다. compile 종료와 고정 EXE/source/hash 보존 뒤 root SDK/Meari freeze 해제 경계를 인계했다. 이후 고정 CLI만 실행하고 전체 슬롯을 반환했다.

## 실제9의 원문·수정·CAS·재열기

고정 package는 `.spec/artifacts/quality-review/sdk-text-results-r3`의 `659b3cb6757259d8e770f6d3c96c4a77dafbab43c6ea540bcc7e0c58e4eb0197`이다. SDK inspect_review/read_text_asset로 package/원문/candidate를 검증하고 source_import로 변환했다. 기존 request/raw를 자동 복구하지 않았으며9개 모두 최초 NeedsRepair/import_error 상태와 raw_text를 보존했다.

각 case는 실제 ProposalFiles를 사용해 create→fresh inspect→명시 scripted 기술 edit→stale expected revision 거부→명시 accept→새 SDK/Bridge로 저장소 재열기/receipt 재조회를 실행했다. source/raw UTF8와 SHA를 전후 재조회했고 acceptance를 그대로 복원했다. 교정은 고정 document reference `/0`, 카드/하이라이트는 명시 source_ref `1`을 사용하는 기술 fixture다. 실제 사용자의 편집·사람 의미 승인이라고 표현하지 않는다. source revision1은 이 수입의 local snapshot이며 원래 project revision으로 승격하지 않는다.

시간 없는 source 값을 현재 영상 PTS로 복구하지 않았다. automatic_application_enabled=false를 확인했고 review verdict를 기록하지 않았다. GeneratedResult에 영속 generation provenance가 없는 상태를 unknown/unbound로 유지했다. 기존9 전체의 역사적 근거와 이번9개 새 plugin 실행은 별개다.

## 고정 CLI 3task/18호출

고정 exe SHA256은 `d8c5101712bfbc42c6486fe209b7a874ad0aef25192fac76b48052975211e447`이다. `--proposals STORE REQUEST.json`으로 각 task의 Gemma case를 **새 CLI store**에 재현했다. 매 호출은 별도 프로세스였으며 추가 compile0이다. task별 exit 순서는 create/inspect/edit/stale-edit/accept/reinspect의 `0/0/0/5/0/0`였다.

| task | 최초 상태 | create/accept/restored source SHA | 복원 결과 |
| --- | --- | --- | --- |
| correction | needs_repair | b282db497133af492690d5d8726549cbafa7dd203517476e64e915c8101b2e27 | document revision3 / receipt draft2·output3,automatic=false |
| cards | needs_repair | 47aecd7b66ef847c6e4a555650d3e39f57e9bfecf5cd00e64082e3da61d49cab | document revision3 / receipt draft2·output3,automatic=false |
| highlights | needs_repair | 658e5fbded1a1203472c937b8b7188c64915859446cb6c4e9b827d877b0d8093 | document revision3 / receipt draft2·output3,automatic=false |

각 stale edit는 canonical RevisionConflict/exit5였고 SDK request context/오류를 보존했다. renderer_verified/browser_verified는 false다. 원본 패키지나 raw를 쓰는 명령을 실행하지 않았다.

## 근거와 남은 범위

`runtime/quality-review/player-ui-proposals-r1/`의 bridge-gate/logs·source-at-check·lock hashes, tests의9 case request/result/technical-evidence, fixed-cli의3 새store/18 stdout-stderr-exit, final-bin/binary-hash에 보존했다. preflight ParentDir fixture 수정은 prepared/preflight snapshot과 note로 구별한다. contracts/application-proposals와 contracts/domain-proposals를 각 책임의 source 경로로 사용하고 초기 동명 merged-copy 디렉터리를 완전한 경로별 snapshot으로 해석하지 않는다.

원본 source/raw 불변은 SDK의 SHA 검증·전후 UTF8 비교로 입증했다. 품질 승인·현재 project와의 연결·미디어 PTS·실제 렌더를 입증하지 않는다. root lock과 독립 bridge lock 전후는 별도 JSON으로 보존했다.

남은 제품 작업은 source callback→typed plugin binding, UI read model의 원문/원초 raw/수정 초안/import_error/revision/receipt 표시, 키보드·편집 focus·다국어·상태·권한/취소·전체13화면군 시각 인수다. Meari assets/source IR 보완을 같은 원본 consumer로 재검증하며 별도 JS 런타임이나 비활성 최종 목업으로 대체하지 않는다.
