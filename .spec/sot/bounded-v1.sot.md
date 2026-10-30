---
id: sot:kinekino/bounded-v1
title: bounded v1 출력
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# bounded v1 출력

상태·인수 한계는 [C0 index](index.md)를 따른다. 아래 수치는 출력 계약이다. P0 r3에서 작성한 bounded DTO의 최대64 item·compact/pretty envelope·escaping 단언은 통과했다. 실제 transport/CLI writer 연결·전체 출력 강제 인수는 후속이며 이 단언으로 승계하지 않는다.

| 항목 | 상한 |
|---|---:|
| scope/ref ASCII | 정확37/54bytes |
| list item compact JSON | 1,024bytes |
| list rows | 원64개 |
| JSON 전체 envelope | 98,299bytes |
| 전체 wire 예산 | 98,304bytes(96KiB) |
| failure/core error message prefix | UTF8 경계1,024bytes |

JSON 예산은 LocalHost 길이 헤더4bytes와 CLI newline1byte를 보수적으로 함께 예약한 값이다. 실제 LocalHost는 binary length header+JSON이며 newline이 없고, CLI는 JSON+newline이며 header가 없다. envelope/api_version/request_id/ok/data/error·escaping을 포함한다. 기존 전역4MiB 값은 변경하지 않는다. CLI compact/pretty는 각각 bounded writer로 검증하고 동일 전체 cap을 지킨다. 유효한 최대64-item compact/pretty 응답 모두 Rust max-bound 시험에서 확인해야 하며 실패하면 조용히 항목을 줄이지 않고 projection/계약을 재검토한다.

list item은 reference와 원 bounded origin/annotation/target/native identity·state/read_mode만 포함하며 오류 본문·전체 SkipSnapshot.segments는 넣지 않는다. scope는 응답 공통 값이다. source identity는 engine_instance_id/resolution이고 Media URL·OS handle·Native ticket/Any·guard를 노출하지 않는다. submitted response의 session_state는 session/revision/pass 최소 projection이다. 기존 inspect/segments Vec에 새 개수 정책을 몰래 추가하지 않는다.

공식 NativeFailure projection은 code/stage·message_prefix/original_message_bytes/sha256/truncated·고정 raw_os_code/kind만 제공한다. 원 native error의 code와 단계·원 길이/hash를 보존하고 임의 ApiError.details Value를 복사하지 않는다. core 후처리 error는 별도의 bounded code/retryable/message prefix/원길이/hash/truncated와 details_omitted/remediation_omitted 표시를 쓴다. 저장/출력하지 않은 details·remediation·원 응답 전체를 복구했다고 주장하지 않는다.

bounded DTO를 먼저 제한된 writer로 encode하고 필요한 Value도 그 제한된 bytes에서만 만든다. 무제한 Value를 먼저 만든 뒤 자르지 않는다. 최종 LocalHost server/client와 CLI writer도 같은 명령별 예산을 사용한다. cap 초과는 bounded ResourceExhausted 응답이며 silent truncate/항목 누락0이다. submit 이후 encode/write 실패여도 원 row/ref를 남기되 이미 terminal consume가 제거한 row를 항상 보존한다고 일반화하지 않는다.

최대 request_id(기존128 Unicode chars·control 금지), UTF8/quote/backslash/6배 제어문자 escaping 오류, u64 max decimal, 모든 enum 최장 이름, 최대식별자,64items, error/data/null, compact/pretty, fallback error와 LocalHost header/CLI newline을 Rust로 확인한다. 전체 응답 cap은 SDK retained8KiB·일시적 allocation/CPU/allocator RSS 상한과 별개다.
