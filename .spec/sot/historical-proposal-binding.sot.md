---
id: sot:kinekino/historical-proposal-binding
title: Historical proposal binding 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Historical proposal binding 계약

2026-10-20 구현 중이다. 설계·source 계약이며 fresh CLI 영속 gate의 성공 기록이 아니다. [구현 계획](../../../../plans/player/historical-provenance-product-checkpoint.md)과 [기존 evidence 계약](../../infrastructure/inference/research-followups/model-engine-cli-contract-handoff.md)을 따른다.

`proposals.import_historical`은 `proposal_id`, 새 import의 `source_id`/`source_revision`, `reference { sha256, bytes, item_id }`를 받는다. 원 source request/candidate raw는 명시 설정한 pinned Rust verifier가 재구성한다. payload로 실행 파일·package 경로·raw text·품질 판정을 덮어쓸 수 없다. 과거 job/attempt/project revision은 발명하지 않는다. 응답은 persisted `StoredProposal`이며 generated raw를 수리하지 않고 NeedsRepair를 유지한다. technical edit/accept는 별도 요청이다.

저장은 기존 writer lock/CAS의 단일 immutable JSON을 쓴다. 선택적 `historical { schema_version: "1", reference }`가 document/acceptance와 같은 publication에 포함되며 이후 revision에서 source/generated와 함께 불변이다. 필드를 생략한 기존 일반 proposal은 같은 bytes/capture SHA를 유지한다. 기존 데이터의 일괄 migration/rewrite는 하지 않는다. 이전 실행 파일이 새 필드를 읽는 forward compatibility는 보장하지 않는다. binding 내부 reference 누락/알 수 없는 version을 거부한다.

새 bound document의 schema_version은3이며 historical(version1)이 필수다. 따라서 첫 revision에서도 binding 전체 제거만으로 legacy가 되지 않는다. source snapshot 자체는 기존 version1/2를 유지하고 일반 document도1/2를 유지한다. document version까지 바꿔 전체 checksum/이력을 다시 작성하는 것은 아래 trust 한계와 구분한다.

bound state의 inspect/edit/accept와 기존 receipt 재반환 전에는 증거를 검증한다. first/latest revision의 불변 결합 차이는 storage recovery 오류다. 저장 JSON 전체·모든 revision·checksum을 임의로 다시 쓴 별도 legacy 이력에 대한 서명/trust root는 이 checksum 방식에 없다. 이를 단일 binding 제거 탐지와 혼동하지 않는다.

project capture의 canonical hash는 bound state 전체를 포함한다. live/saved capture, project inspect/reopen과 save의 이전 revision에도 reader 검증을 적용한다. working draft·original generated·acceptance는 별도 상태다.

시작 설정은 `--proposal-root`와 `--historical-verifier`, `--historical-verifier-sha256`, `--historical-workspace`, `--historical-package` 전부를 명시한다. GUI/CLI는 같은 설정 계약과 infrastructure factory를 쓴다. 개발 EXE 자동 검색·shell·browser가 고른 실행 경로·remote endpoint와의 혼용을 허용하지 않는다. 미설정 import는 Unsupported이며 bound state를 미검증 일반 state로 강등하지 않는다. 배포물의 실제 설치 성공은 별도 gate다.

verifier는 동기 처리이며 stdout 상한2MiB·stderr 상한8KiB·실행 deadline15초를 사용한다. deadline은 cleanup 완료를 뜻하지 않는다. 실패 시 원 오류 코드와 cleanup_complete를 구별하고 kill/wait/pipe 완료를 확인하지 못하면 owner를 보존한다. 후속 호출은 cleanup만 수행하고 동시에 replacement를 시작하지 않는다. Drop은 best effort이며 현재 pinned verifier source의 자식 비생성을 전제한다. 일반 프로세스 트리 관리를 보장하지 않는다.

Immediate UI-thread에 동기 verifier를 적용한 것이 무차단 GUI의 증거는 아니다. worker scheduling·typed Pending·terminal completion/wake는 별도 UI 통합 잔여이며 Rc registry/GL을 worker로 보내지 않는다. evidence 검증·technical acceptance는 사람 품질 승인, model admission, subtitle/render/export 적용 완료가 아니다.
