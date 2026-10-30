---
id: sot:kinekino/revision-2
title: 카드 묶음의 편집 문서와 revision
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 카드 묶음의 편집 문서와 revision

프레임 참조의 source_sha256는 이 문서의 cue source snapshot에 대한 연결이다. 실제 원미디어 bytes SHA와 같다고 가정하지 않는다. 후속 application은 기존 review media association/producer에서 cue source와 원미디어·프레임의 관계를 별도로 검증해야 하며, 이 문서의 hash 일치만으로 미디어 provenance/PTS를 인증할 수 없다.

2026-10-28 R99 구현 계약. 전체 [카드 제작](cards.md)과 [task 통합](task-integration-contract.md)의 첫 구현 단위다. 전체 export_cards·Jobs·SDK/CLI 연결과 렌더/사용자 품질 인수는 아직 완료되지 않았다. 원 ProposalPayload::Cards의 Card(front/back/source_refs)를 재사용하며 proposal raw/accepted output을 덮어쓰지 않는다.

domain/card_document는 CardDocument(schema_version=1, document_id, revision, 원 CueSourceSnapshot, layout)를 소유한다. layout은 canvas/template id·version·SHA, 순서 있는 카드와 게시문 초안이다. 카드는 역할(표지/본문/마지막), 텍스트 종류(인용/요약/수동), 원 Card 내용, 선택 frame artifact/hash/source hash/선언PTS, alt 초안을 가진다. 선언 PTS는 producer 검증/권리/품질 승인이 아니다. 문서 등록만으로 frame 실제 파일/PNG나 template 가용성을 입증하지 않는다.

CardEditor는 유효 문서를 소유하고 읽기 참조만 제공한다. source를 바꾸는 편집은 없다. 추가·복제·삭제·같은 ID의 내용/프레임/역할 교체·정확한 전체 순열 재배치·canvas/게시문 수정을 하나의 정책으로 처리한다. expected revision이 현재와 다르면 거절한다. 후보 전체 검증에 성공한 경우에만 문서/undo를 함께 갱신한다. undo도 새 revision이며 u64 소진을 거절한다. 최근 32회 layout만 메모리에 보관하며 영속 저장 이력/프로세스 재시작 undo 지원을 주장하지 않는다.

인용은 refs 순서의 원 cue 텍스트를 LF로 연결한 값과 back이 정확히 같아야 한다. 요약은 비어 있지 않은 유효 cue refs를 요구하지만 사실성/품질을 자동 승인하지 않는다. 수동 초안은 빈 텍스트/refs·빈 카드 묶음도 허용한다. 초안 유효성과 export preflight는 별도이며 빈 초안을 렌더 완료로 표시하지 않는다. 중복 card ID·누락/중복/외부 refs·다른 source hash의 frame·컨트롤 문자·허용량 초과를 거절한다.

운영 한도: 카드 128개, 문서 직렬화 512KiB, canvas 각 축 1..4096, front/alt 8KiB, back/게시문 16KiB. 이 값은 플랫폼 규격/품질 승인이나 모든 크기의 renderer 지원을 뜻하지 않는다. 현재 renderer는 명시된 템플릿/캔버스만 지원하며 후속 application preflight가 실제 가용성을 검사해야 한다. 비율 변경 뒤 기존 시각 적합성을 승계하지 않는다.

저장 형태는 deny_unknown_fields로 불명확한 승인 flag 등을 거절한다. Deserialize 뒤에는 반드시 validate 또는 CardEditor::open을 호출해야 한다. CardEdit는 현재 도메인 편집 값이며 공개 wire schema가 아니다. API 연결 때 기존 DecimalU64/context/CAS·한도 계약에 맞는 DTO를 명시해야 한다. 원본 store 검증·전역 revision CAS·동시 writer·취소/발행은 후속 application/인프라 책임이다.

기능 소유권은 SDK application/domain/API/CLI, Media 원본/시간축, UI 표시/템플릿을 유지한다. 첫 구현은 private mirror에만 있으며 canonical 적용/기능 지원 승격을 뜻하지 않는다.
