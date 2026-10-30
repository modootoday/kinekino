---
id: sot:kinekino/model-file-verification
title: 로컬 모델 파일 검증 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 로컬 모델 파일 검증 계약

2026-10-27. `models.verify`는 [카탈로그 조회](model-catalog-reads.md)에 연결되는 read-only 무결성 관측이다. 입력은 exact model_id/revision/catalog_sha256/maximum_bytes다. maximum_bytes는 decimal1..1TiB이며 선언된 전체 파일 크기 합이 한도에 들어가는지 모델 파일 read 전에 확인한다. 최대128개 원 파일·parent directory guard와 원 catalog guard를 함께 보유한다. 기본30초, context timeout_ms1..120000ms를 한 번 고정하며 component마다 갱신하지 않는다.

카탈로그 pin 변경은 RevisionConflict, byte 예산 부족은 ResourceExhausted다. 각 파일은 missing/size_mismatch/hash_mismatch/matched를 기록한다. 원 파일의 크기·SHA256을64KiB checkpoint로 읽고, 전부 일치할 때만 성공한다. 불일치가 하나라도 있으면 NeedsInput과 error.details의 전체 검증 보고서를 반환하며 exit0로 성공을 가장하지 않는다. partial 일치가 영구 설치/승인 상태로 저장되지 않는다.

성공 보고서는 catalog SHA·exact model revision·선언/관측 byte/hash를 결합하며 integrity=matched_at_verification·persistent_admission=false다. 이는 해당 호출에서 guard를 보유한 마지막 관측 시점의 파일 일치다. 반환 뒤 lease를 부여하거나 후속 inspect를 verified로 바꾸지 않는다. 모델 포맷 내용/remote code 안전성·tokenizer 의미/엔진 호환·실제 dtype·정밀도·성능·license permission·사용자 품질 승인이 아니다. runtime_admitted/quality_approved=false,license_permission=not_verified를 유지한다.

공통 Rust `verify_model`은 원 MutationControl checkpoint 인터페이스를 사용해 취소/시간제한 오류를 그대로 보존한다. 기존 review 저장 정책을 호출하거나 model 결과를 수정하는 것은 아니다. SDK request의 timeout은 단일 원 Instant deadline이며 동기 파일 read의 hard preemption을 보장하지 않는다. 취소/Timeout 시 원 파일·BCrypt handle과 verifier slot을 정상 scope로 해제하고 성공 보고서를 반환하지 않는다. 동일 adapter의 동시 검증은 try-lock으로 ResourceExhausted다. 별도 process 사이의 heavy 배정은 중앙 PM이 직렬 유지한다.

설치/삭제/다운로드/추론·캐시 readiness 승격은 실행하지 않는다. 원 declared manifest를 수정하지 않는다. 모델 및 startup 설정 파일 adapter의 Windows module gating을 명시했으나 전체 non-Windows/Web 제품 build는 별도 미검증이다.

2026-10-27 SDK execution 후속: 원 operation cancellation과 기존 file checkpoint/deadline을 [같은 control 경로](model-verification-execution-control.md)로 합성했다. CLI wire cancel ACK는 계속 미지원이며 같은 프로세스 ExecutionHandle 경로와 구분한다.
