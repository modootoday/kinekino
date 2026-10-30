---
id: sot:kinekino/card-project-storage
title: 카드 편집 프로젝트의 불변 저장
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 카드 편집 프로젝트의 불변 저장

R104 후속: [공통 SDK/CLI 계약](card-project-sdk-cli.md)과 [실제24호출/지정 gate](../../../verification/application/card-project-sdk-cli-20261028.md)에서 create/inspect/save 연결을 확인했다. 아래 미등록 문장은 최초 R101 범위이며 현재는 명시 저장 root 구성에서 지원한다. 전체 export_cards·Jobs/renderer는 계속 남는다.

2026-10-28 R101. [CardDocument](card-document.md)를 위한 application port와 파일 어댑터다. 원 ProjectFiles의 schema1/2 레코드는 WatchingSelection/proposal snapshot을 저장하며 CardLayout을 표현하지 못한다. 기존 이력 형식을 변경하거나 타 소유 UI/project 소비자를 덮어쓰지 않고 카드 편집 문서 책임을 별도 adapter로 분리했다. 경로/파일 guard와 writer 잠금은 기존 immutable_storage_paths를 재사용한다. 원미디어를 등록하는 새 registry가 아니다.

application/card_projects의 CardProjectStore는 load/publish, 공통 usecase는 create/save/inspect다. CardPublication(schema_version=1, publication_revision, parent_publication_sha256, document)을 고정 직렬화하며 그 bytes의 SHA256으로 레코드를 식별한다. document.revision은 편집 revision이고 publication_revision은 저장 이력의 연속 번호다. 예를 들어 편집 revision1→4를 저장하면 발행 revision1→2가 된다. undo/메모리 편집의 과정을 저장했다고 주장하지 않는다.

새 저장은 edit revision1만 허용한다. 저장 갱신은 expected publication revision이 실제 최신과 같아야 한다. source snapshot과 document ID를 유지하고 edit revision이 증가해야 한다. 여러 편집 후 같은 내용으로 돌아온 문서도 새 edit revision이면 저장할 수 있다. 파일 어댑터는 원 최신 이력 조회·CAS·준비 검증·발행을 같은 writer 잠금 안에서 수행한다. source 비교는 cue snapshot 연결이며 원미디어 provenance 인증이 아니다.

명시 root는 이미 존재하는 절대 로컬 경로여야 하며 기존 root/ancestor/reparse guard를 사용한다. card-{document_id}/r-{publication_revision}/{publication_sha256}.json에 create_new로 발행하고 sync_all을 호출한다. 기존 레코드를 덮어쓰지 않는다. writer 충돌은 RevisionConflict다. 카드별 폴더와 발행 revision 폴더는 새로 만들며 원본을 이동/삭제하지 않는다.

load는 요청한 과거 revision만 검사하고 끝내지 않는다. 전체 저장 이력의 연속 번호·단일 canonical JSON·파일명 SHA·schema·document 검증·부모 hash·원 source 불변·edit revision 증가를 확인한 뒤 요청 revision과 최신 발행 번호를 반환한다. 최신 손상·빈 발행 폴더·부모 연결 불일치가 있으면 과거 정상 이력으로 fallback하지 않는다. NeedsInput/recover_card_project_publication을 반환하고 원 상태를 보존한다. 프로세스 중단/디스크 오류로 부분 폴더가 남을 수 있으며 자동 복구/삭제와 원자적인 디렉터리 전체 발행을 보장하지 않는다.

경로 guard/잠금/새파일/sync는 기존 기반이다. 실제 전원 손실의 crash durability·권한 실패/disk-full 주입·공격자 전체 이력 재작성 방지·영속 undo·사용자 품질 승인은 미검증이다. 파일 hash는 무결성 연결이며 사용자 인증 서명이 아니다. 재시도/멱등 ACK 복구 및 exporter의 산출물 발행은 별도 경계다.

현재 공개 PlayerSdk/catalog/CLI에 등록하지 않았다. 공통 Rust usecase와 실제 파일 adapter만 추가한 단계이며 production.export_cards는 미지원이다. 이후 명시 저장 root/typed DTO/context/schema/CLI·Jobs를 연결하고 실제 저장/거절 근거를 재사용한다. 전체 카드 산출물·renderer·품질 인수는 남는다. 기존 담당 소유권·WIP·원 증거·private overlay·C-only/heavy1·WARP/추론/다운로드/수동 GUI/push 보류 유지.
