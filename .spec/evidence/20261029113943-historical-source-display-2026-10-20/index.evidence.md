---
id: evidence:kinekino/20261029113943
title: 저장 historical 참조의 제안 UI 표시
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 저장 historical 참조의 제안 UI 표시

2026-10-20 PM 독립 표시 수리 배정. 기존 P1a 원본/핀은 보존하며 이번 소스 변경은 새 revision이다. 이전 SDK/OS/렌더 통과 결과를 새 source로 소급하지 않는다.

`ProposalView.historical_reference: Option<CompanionReference>`는 `StoredProposal.historical.reference`의 표시용 사본이다. `project(&StoredProposal)`는 구조 validate와 투영만 수행한다. 기존 generation_provenance_bound/quality_decision_known=false를 유지하고 “DTO에 참조가 없다”는 옛 주석을 “저장 참조/구조검증은 증거·품질 검증이 아니다”로 수정했다. 새 SDK/검증 API·도메인 정책·파일 접근은 없다.

공통 Rust historical_source_props가 저장 참조 없음 또는 SHA-256/bytes/item ID를 표시 문자열로 만든다. 공통 ProposalSource에 두 문자열 props를 추가해 카드/하이라이트/교정이 동일한 저장 참조 영역과 미확인 문구를 소비한다. 초기 bootstrap, live 카드/하이라이트 panel_model, 초기/live 교정 correction_panel에 같은 builder를 적용한다. 편집·Save·Accept·Refresh 명령이나 callbacks 의미는 변경하지 않는다. 표시된 hash는 companion 참조의 hash이며 review package hash로 재명명하지 않는다. 지역화된 표시 문구는 향후 공용 locale catalog로 이관할 대상이며 이번에는 단일 Rust builder에서 관리한다.

canonical `application::proposals::inspect`는 verified_load→store.load→ai_provenance::verify_bound를 호출한다. 해당 inspect 성공 관측은 그 시점의 SDK 검증 근거다. 반면 plain DTO의 표시 투영은 동일 파일/현재 작업/모델 precision/license/사람 quality를 재확인하지 않으며 별도 검증 receipt를 갖지 않는다. 따라서 stored reference 존재만으로 verified/current/quality를 true로 승격하지 않는다. 원 candidate의 review 결정이나 acceptance receipt도 편집된 output revision의 사람 품질 승인을 뜻하지 않는다.

회귀는 기존 canonical_v3 시험에 Option 참조 보존/미확인 flags를 추가하고, 신규 시험에서 세 provider×참조 유무의6경우에 초기/live props 일치·실제 hash/bytes/item 표시·없음 표시·DTO bytes 불변·미확인 flags를 대조하도록 작성했다. UI 문자열로 성공을 꾸미는 시험이 아니라 동일 SDK DTO와 실제 기존 pure props builder를 사용하는 범위다. Rustfmt는 변경 Rust6파일에 한해 실행했다. 현재 Meari가 heavy를 소유해 Cargo compile/test/clippy/제품렌더/GUI는 실행하지 않았다. 시험 작성과 통과를 구분하며 추후 중앙 배정에서 집중 시험과 필수 검사를 진행한다.

변경 범위10개는 projection/mod/bootstrap/bootstrap tests/panel_model/correction_panel 및 ProposalSource/카드/하이라이트/교정 TSX다. `runtime/quality-review/historical-source-display-r1/`에 before/final 소스·inventory와 포맷 결과를 보존한다. SDK parent broker/application/bridge/Meari/Cargo/OBJ 초안은 수정하지 않았다. root index는 PM 소유로 stage/commit0이다.

PM 후속 index 배정으로 정확10개를 `298f023706a540b6910d748d489f2fde463d217d`에 커밋했다. final inventory SHA `ACD644528C7DF7FE40332C9A59124072C914EDA7A1E4FD02AF24E595B04B73F2`, source 재해시 차이0/staged 잔여0이며 index를 PM에 반환했다. push0, 제품 bytes는 commit 전후 동일하다. 이 후속 상태가 위 작성 시점의 stage/commit0을 대체한다.

직접 fixture 영향 검색상 `tools/player-ui-sdk-bridge/tests/original_review_sdk.rs`의 props()가 sourceIdentityLabel/qualityLabel을 literal map으로 구성한다. 새 필수 문자열2개를 공통 historical_source_props로 extend하는 최소 수리가 필요하다. 현재 해당 bridge 파일은 수정하지 않았고 PM/SDK에 별도 단계로 인계했다. 나머지는 canonical props builder를 소비하지만 실제 compile/runtime 인수는 아직 없다. 과거 fixture 통과를 변경된 required props의 통과로 간주하지 않는다.

PM의 다음 단일파일 배정으로 original_review_sdk.rs의 props()에 historical_source_props extend3줄을 추가했다. 기존 assertion/SDK 호출/fixture 정책은 수정하지 않았다. fixture-before/after.rs와 pin을 별도 보존하며 after SHA `E19500CA19E233067F92CF95C21803114C928ADDD2A6119E2083E7C67466DD2C`다. 직접 rustfmt/check0, Cargo/GUI0, 아직 미컴파일·시험 미실행이다. rootindex는PM소유로 stage/commit0이다. 표시10+fixture1은 후속 집중 검사의 새 입력이며 SDK parent5/P3 snapshot 또는 이전 공개 gate 결과에 섞어 소급하지 않는다. 저장 참조 유무6경우/unknown flags/initial-live props 회귀와 원 3provider imported graph 시험은 후속 중앙 gate에서 실행한다.

후속 exact1 index 배정에서 fixture를 `8d15f6700c8fcbff826716d2f097836fb197f203`으로 커밋했다. cached check0/원 SHA 유지/staged 잔여0, 다른 source·OBJ·lock 변경 없이 index를 PM에 반환했다. 이 관측이 앞선 fixture stage/commit0을 대체한다. 실제 기능 검증은 아직 실행하지 않았다.

## UI11 공개 snapshot 집중 gate

PM 배정 후 실제 cwd `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1`에서 실행했다. 원 공개 baseline1fd8ca1+overlay4/P3 a50905e5/Meari2c08832를 유지하고 UI11만 별도 revision으로 적용했다. [입력 계획](../../../plans/player/presentation/historical-source-display-gate.md)과 snapshot-input-ready.json이 정확 경계를 기록한다. root는 원 C native cache, bridge는 원 D consumer cache를 재사용했고 고정 runner/mask3/2CPU/BelowNormal/descendant Job/offline locked j1/test thread1을 사용했다.

| 검사 | 실제 결과 | receipt ms |
|---|---|---:|
| root feature-on bootstrap 필터 |12pass/0fail, 신규 참조유무6경우 initial/live props 회귀 포함 |145537|
| bridge feature-on original_review_sdk |2pass/0fail, 원 Card/Highlight imported graph와 실제 SDK |381763 전체2target|
| bridge feature-on studio_frame_sdk |5pass/0fail, 교정 포함3provider·atomic/recovery |위와 같은 실행|
| root fmt |terminal0|2233|
| bridge fmt |terminal0|5613|
| root feature-on all-target strict |terminal0|56775|
| bridge feature-on all-target strict |terminal0|54248|
| bridge feature-off original_review_sdk focused strict |terminal0|19865|

bootstrap 뒤 실행 전환에서 증거 상대 경로를 snapshot cwd로 잘못 해석해 log 조회와 owned 목록 파일 저장이 실패했다. 원 exec는 종료0이었고 다음 명령 전 Get-Process count0 확인은 수행했으나 해당 bootstrap-owned 파일은 남지 않았다. bootstrap receipt terminal0은 직후 절대 경로로 재확인했다. 이 기록 누락을 사후 owned[] 생성으로 보완하지 않았다. 이후 증거는 절대 경로/ErrorAction Stop을 사용한다.

root fmt 뒤 전역 조회에 미확인 cargo836/rustc46156이 잠시 관측돼 다음 명령을 시작하지 않았다. 즉시 CIM 조회에는 없어서 소유/명령줄을 확정하지 못했다. 원 root-fmt-owned.json을 보존하고 다음 부재 확인 후 진행했다. 외부 프로세스 종료·검사 재시작·중복 실행은 하지 않았다. PM과 조율해 이후에는 본인 원 handle/receipt 가족을 기준으로 확인한다.

최종 checkpoint: 배정된 집중 gate를 완료했다. 7receipt 모두 terminal/exit0이고 각 원 exec가 종료했다. final-overlay-differences.json=[]는 live/snapshot의 UI11 핀 일치를, final-protected-differences.json=[]는 snapshot의 기존 overlay9/Meari344/manifest-lock16 불변을 확인한다. final-owned.json=[] 및 final-build-processes.json=[]이며 PM에 단일 heavy 슬롯을 명시 반환한다. 이번 gate 중 제품 소스 수리는 없고 index는PM소유/stagecommit0이다.

입증 범위는 저장 historical 참조 유무와 unknown flags/세 provider initial-live props, 원 imported graph 및 기존 SDK interaction/atomic recovery의 집중 회귀다. GUI/OS/전체 시각13·P3 실제 parent/media·Meari Boolean 인수로 이 결과를 확대하지 않는다. P3 source5를 포함한 같은 기초 snapshot을 썼지만 기존 fake5를 다시 실행하지 않았으며 신규 parent wake 소스는 섞지 않았다. 기술 검사의 성공은 원 candidate 또는 편집 output revision의 사람 품질 승인으로 승격되지 않는다.
