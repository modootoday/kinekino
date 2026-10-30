---
id: sot:kinekino/sdk-cli
title: 카드 프로젝트의 공통 SDK와 CLI
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 카드 프로젝트의 공통 SDK와 CLI

2026-10-28 R102/R103. [편집 문서](card-document.md)와 [불변 저장](card-project-storage.md)을 공통 PlayerSdk에 연결한다. production.export_cards와 다른 편집 프로젝트 저장 기능이다. 카드 묶음 출력·renderer·Jobs·사용자 품질 승인은 아직 남는다.

| 명령 | payload | context | 동작 |
|---|---|---|---|
| card_projects.create | document: CardDocument | expected_revision 없음 | edit revision1의 새 불변 발행1 생성; 기존 ID는 충돌 |
| card_projects.inspect | document_id, 선택 publication_revision | expected_revision 없음 | 요청 이력과 최신 발행 번호 조회; 전체 이력 검사 |
| card_projects.save_revision | document: CardDocument | 양수 expected_revision 필수 | 실제 저장 head에 CAS; edit revision 증가/원 source 불변 검사 |

세 명령 모두 idempotency_key/timeout_ms는 미지원이며 명시 값은 InvalidArgument로 거절한다. 동기 저장이므로 in-flight cancel을 지원한다고 표시하지 않는다. 문서/payload는 unknown field를 거절하고 기존 DecimalU64와 context 요청 검증을 사용한다. create/save/inspect는 같은 공통 application usecase와 저장 port를 호출한다. 타입 안전 client는 ApplicationClient의 create_card_project/save_card_project/inspect_card_project이며 Arc<dyn ApplicationClient>에서도 사용할 수 있다.

기본 SDK에는 카드 저장소를 주입하지 않는다. with_card_project_store가 주입된 구성만 capabilities/schema에 supported를 표시한다. CLI는 --card-project-root EXISTING_ABSOLUTE_DIRECTORY로 동일 파일 adapter를 주입한다. root를 자동 생성하거나 기본 위치를 선택하지 않는다. --endpoint와 로컬 root를 섞지 않으며 다른 host를 자동 시작하지 않는다. JSON 요청은 --input FILE|-를 사용한다. 조회도 writer 잠금과 이력 검증을 수행하므로 lock 충돌/손상 오류가 가능하다.

전체 기본 카탈로그는 이전100에서103으로 증가한다. 별도 execution host의10항목과 합산하지 않는다. 명령 수 증가는 제품 완료율이 아니다. schema는 document/layout/card/canvas/frame/source와 발행 결과 구조·한도를 제공한다. 숫자 범위·UTF8 bytes·참조/인용/이력/CAS는 Rust usecase에서 검증한다. 독립 JSON Schema 엔진의 모든 경계 동등성 검증은 아직 하지 않았다.

원 source cue hash·frame 선언PTS·문서 저장만으로 실제 미디어/producer provenance·권리·AI 품질을 인증하지 않는다. 최초 문서는 편집 입력이며 accepted output이나 사용자 판정으로 변환하지 않는다. 실제 원 proposal을 가져오는 UI/CLI 경로는 해당 ProposalStore/계보 검증을 별도로 거쳐야 한다. 카드 snapshot에 들어간 문구/alt/게시문은 초안이다.

대상은 private mirror이며 canonical/root WIP/다른 담당 소스를 덮지 않는다. 기존 source375·원 실제9·첫 실패/증거·C-only/heavy1·보류를 유지한다.
