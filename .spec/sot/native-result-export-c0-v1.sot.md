---
id: sot:kinekino/native-result-export-c0-v1
title: Native result export C0 · v1 계약 정본
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Native result export C0 · v1 계약 정본

2026-10-21. rootc485의 B11 인수 뒤 구체화한 C0 설계 정본이다. 후속 P0 정확5의 pure lookup/DTO/max-envelope·기본 Unsupported는 [Native199·양strict/fmt](../../../verification/application/native-export-c0-p0-2026-10-21.md)로 인수했다. **실제 GUID/원row·packet target·최종 transport/CLI 지원은 아직 인수가 아니다.** 기존 A/B와 segments.skip·지원false를 유지한다. scope128/ref256 제안은 아래37/54 형식으로 대체한다. DTO 크기 단언 통과와 실제 wire 출력 계약 인수를 구분한다.

- [식별자·strict codec·원 issuer handle](codec.md)
- [원 metadata·실제 target 결속](metadata.md)
- [API·조회·오류·소비·복구 수명](lifecycle.md)
- [bounded projection·전체 출력 예산](projection.md)
- [단계별 exact/소유권·구현 계획](../../../../plans/player/native-result-export-c0-producer.md)

공통 SDK 생산자와 소비 구현을 분리한다. application 계약·공식 codec·원 row/issuer/resolver는 SDK 소유다. 소비 담당은 기존 skip 정책을 재사용해 선택된 annotation을 같은 호출에서 전달하고 strict DTO/schema/client를 연결한다. CLI 조립과 전역 등록·출력 경계는 마지막 통합 단계다. 현재 미디어 담당의 sole heavy를 침범하지 않는다.

공식 v1 명령은 `segments.skip_export`, `segments.export_list`, `segments.export_peek`, `segments.export_consume`로 설계 고정한다. 이름은 표시 제품명과 독립이며 등록/지원 구현은 아직0이다. 목록/참조는 read-only이고 guard/submit 권한을 직렬화하지 않는다. 원64 grant→command row·원 ticket/admission/OnceLock·Weak owner와 EOF 정리를 유지하며 별도 command ID/map/미소비 eviction/tombstone을 추가하지 않는다.
