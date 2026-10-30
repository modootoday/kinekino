---
id: plan:kinekino/20261029113626
title: A pure protocol 최소 입장·검증 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# A pure protocol 최소 입장·검증 제안

후속 중앙 명시 배정으로 [2026-10-24 실제 정본](../../../sots/verification/production/media-source-protocol-a-2026-10-24.md)에 actual12/fmt/strict·lock입장·원 cwd실패·보호·반환을 기록했다. source6은 불변이며 source8 채택 후보만 별도 인계했다. 아래 미실행 표기는 작성 당시 이력이고 새 실행 권한이 아니다. B/C/D와 생산 지원은 계속 미완료다.

2026-10-23 source-only 준비. 현재 root579f5db0778a9d48f2eb2afc251c1f232cca7de3/libraryb532, UI sole-heavy다. 실행 권한은 execution-leases.md만 따른다. 동결 source6 F0496B800F8C7A34EEA5A306507BB3A93E629635821BA8A3BEA348108A9CC7AF와 before6 5B1CBA23E2747E2DDF77B60706712E0850E4BEA74BB98D19655ADBDA6D5A4D22는 불변이며 독립 읽기 확정 P1/P2 없음·leaf fmt5 통과와 타입/등록/actual0을 구별한다. 이번에는 manifest 읽기만 했고 metadata/Cargo/S/T 실행0이다.

## 현재 S와 정확 등록 영향

root와 기존 제품 S의 Cargo.toml SHA는 모두 E784DB6ABE12BE8300F1EA460AC0CED3E31BC6B3773A6F8E4D51E08310057976, Cargo.lock은 모두 E469A7204B9DC2AC82AC729E36FD8AC41449E1C382E9167A28A00A45C45789A2다. 양쪽 members에 media-source-host가 없고 lock에도 같은 package entry가 없다.

미래 S 검증 입력은 new6 + 기존 root Cargo.toml의 workspace.members에 `packages/media-source-host` 한 항목을 추가하는 exact1이다. default-members·dependencies·features·exclude·resolver·root target은 변경할 필요가 없다. `-p media-source-host`로만 선택하며 root 제품 dependency로 연결하지 않는다. package에는 외부 dependency/build.rs/bin/worker feature가 없다. 기존 library UI 후보나 runtime13을 합성하지 않는다.

lock은 손으로 package stanza를 쓰지 않는다. 별도 중앙 입장 시에만 S의 기존 lock/preimage를 보존한 뒤 `cargo metadata --offline --format-version=1`로 Cargo가 새 local package를 해석하고 lock을 생성하도록 하는 **단 한 번의 의도된 lock-write 예외**를 요청한다. 이것은 현재 허가가 아니며 --locked 일반 규칙의 묵시 예외도 아니다. --no-deps 결과만으로 lock 입장을 증명하지 않는다. 새 local package media-source-host0.1.0의 dependency 없는 entry 외 기존 package/version/source/checksum/dependency graph 차이가 나오면 원 출력/lock을 보존하고 중단·PM 재조율한다. std-only라고 lock 무변경을 주장하지 않는다. 이후 명령은 모두 offline/locked다. registry update/download fallback·lock 손편집·새 target·clean은 없다.

실제 root 채택은 gate 후 PM이 source6+workspace 등록1+생성 lock1의 exact8을 별도 검토하는 범위다. 추가로 현재 `.gitignore`는 media-source-host 디렉터리를 허용하지 않는다. 공개 채택에는 제품 공통 protocol의 필수성·직접 작성 Rust/외부 자산 없음·비밀정보/운영자료 분리 검토 후 `!/packages/media-source-host/` 디렉터리 경계 한 항목만 별도 PM 배정해야 한다. 기존 공통 src/tests 허용 패턴을 재사용하고 파일별 예외나 비공개 runtime 경계 해제는 하지 않는다. 이 adoption 경계1은 컴파일 gate input과 구별하며 지금 편집하지 않는다.

## 별도 배정 후 최소 실행 순서

1. 최신 root/library/lock과 new6 부재·raw6·workspace preimage를 다시 확인한다. 기존 S/T·runner를 사용하고 fresh RAM/GPU/C·writer 부재와 원 index/root/library/S 비입력 bytes/UTC mtime·고정 BIN·normal aliases/FileID·knowncache4709 두 bytes를 보존한다. 실제 build/link 중 hash 감사를 병행하지 않는다.
2. exact7만 S에 반영하고 변경 bytes에만 fresh mtime을 적용한다. 새 package admission용 metadata/생성 lock을 위 별도 승인 범위로 실행·원 JSON/receipt/lock diff를 보존한다. package 실제 id/path/edition2024/target lib+integration codec/외부deps0을 확인한다. 이 성공은 타입/시험 통과가 아니다.
3. `cargo fmt -p media-source-host --check`, `cargo clippy --offline --locked -j1 -p media-source-host --all-targets -- -D warnings`, `cargo test --offline --locked -j1 -p media-source-host --test codec --no-run --message-format=json`을 기존 conservative2 runner로 직렬 실행한다. root 전체 재빌드나 다른 package 시험은 이 추가 전용 library의 소비자가 없으므로 제안하지 않는다. metadata가 다른 closure를 요구하면 자동 확대하지 않는다.
4. Cargo JSON의 실제 codec executable/path/package/target/features/profile/fresh/SHA/bytes를 고정 copy하고 같은 BIN의 --list를 회수한다. 아래12개 이름과 실제 등록을 대조하고 추가/누락/ignored 여부를 판정한 뒤 선택 실행한다. 단순 rustfmt/no-run 성공을 실제12로 계상하지 않는다. `--exact --test-threads=1`로 각 이름을 한 번씩 실행하거나 동일 고정 BIN의 검증된 전체12를 한 번 실행하며 둘을 중복 계상하지 않는다.
5. 원 오류·BIN·로그·receipt를 보존하고 같은 exact6 안의 명백한 최소 수리는 별도 revision으로만 진행한다. 추가 경로/정책/owner 변경은 재배정한다. terminal·원 handle/worker session 정리·고정copy/원 normal alias·root/library/S비입력·lock·BIN producer 차이를 확인하고 SOT/명시 반환한다. 현존 cache 미귀속2는 그대로 미해결/all-cache=false다. S의 승인 workspace/생성 lock input 차이는 보호 실패와 구별한다. root 적용/commit은 PM만 소유하며 push0이다.

## source-derived codec 예상12

모두 package media-source-host, integration target codec이며 아직 실제 목록이 아니다.

```text
kind_and_length_limits_are_explicit
cutoff_values_cannot_be_renewed_by_intersection
fixed_header_roundtrip_rejects_version_flags_and_trailing_bytes
oversized_prefix_fails_before_reading_or_allocating_the_declared_body
transport_eof_truncation_and_body_end_are_distinct
sequence_credit_and_offsets_are_consumed_once
credit_overflow_and_replay_fail_closed
wrong_binding_direction_offset_and_order_are_rejected
absolute_cutoff_never_restarts_at_receive_and_control_needs_no_credit
failure_is_bounded_utf8_and_does_not_authorize_late_success
two_bounded_frames_can_carry_a_source_chunk_larger_than_64kib
clock_rollback_and_terminal_reopening_are_rejected
```

## 완료 의미와 미지원

실제12가 통과해도 순수 byte/state 계약만 입증한다. 동기 Read는 nonblocking 부모 IPC가 아니고 clock integer는 OS cross-process clock 검증이 아니다. BodyEnd/EOF/child exit/family empty/physical ACK는 구별한다. HTTP/auth/replay issuer·worker/DNS·B/C/D·root 제품/SDK 등록·제품 CLI 지원은 없다. metadata가 opaque이므로 URL/권한 token을 임의 해석하지 않는다. 실행 배정 전 현재 actual0을 유지한다.

이미 읽은 modoo.today test-fixture-and-mock-build SHA a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100(필수 추가 참조 없음)의 정의 기반 fixture와 synthetic/actual 구분을 적용했다. 첫 진단/실제 target/BIN 목록/원 보호를 구분하는 기존 QA 원칙도 유지한다. 성능은 not measured이며 예상 시간/속도 개선을 주장하지 않는다.
