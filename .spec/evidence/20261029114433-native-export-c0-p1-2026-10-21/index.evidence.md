---
id: evidence:kinekino/20261029114433
title: C0 P1 원 native row 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# C0 P1 원 native row 검증

2026-10-21 PM의 정확7 source 감사 및 단일 heavy/shared 실행 배정으로 검증했다. 원 기준3032와 root `bf98c4ec678c339b23351fbe73e6804f5a143f39`의 before7 일치를 확인하고, 소비9r2 protected→integrated9-r2의 snapshot baseline1401(Meari412 포함)에 정확7만 적용했다. 원 source7 SHA256 **ABC598812F754442AC2ADC08DD1941111384D31FC00A7E7BC64D44F23A2751BD**를 수리 없이 유지했다.

## 관측 결과

| 최종 구성 | 실제 고유 시험 |
|---|---:|
| Native libtest SDK 기존199+P1 신규9+소비 skip 추가5 | 213 |
| 같은 최종 source의 Native integration native_skip_cli | 6 |
| 합계 | 219 |

실제 이름과 원 stdout은 `runtime/quality-review/sdk-native-export-c0-p1-r1/actual-tests.json`을 따른다. Native fresh libtest compile은185599ms 한 번이며 이후 같은 binary의16필터로 확인했다. standalone_export_tests6에 새 engine3을 포함했고 guarded_command58에 scope2/issuer4를 포함했다. 기존 helper2 포함, broad/하위 필터 중복 가산0이다. integration target 명칭이 실제 player-cli.exe 실행을 뜻하지 않는다.

긴 링크 전에 default all-target strict31385ms, Native all-target strict84401ms, workspace fmt3791ms가 전 exit0이었다. 수리0이고 모든 시험이 같은 최종 source pin이다. default는 타입/lint만 확인했고 runtime0이다. 실제 CoCreateGuid·mpv/미디어·브라우저/GUI·CLI executable·DPI/VM·AI 실행은 하지 않았다.

## 검증된 경계

원 HostScope OnceLock의 nonce 성공/실패 캐시를 주입 seam으로 검증했다. 실제 생성 어댑터는 CoCreateGuid지만 OS 생성 성공 자체는 이번 시험 범위가 아니다. opaque issuer는 원 Weak만 보존하며 동일 nonce의 외부 Weak 소유자를 거부한다. 기대 issuer를 Active lock/native Observe 전에 확인하고 원 owner 만료를 현재 세션/route로 바꾸지 않는다.

같은 원 admission과 read cell을 row/caller가 공유한다. v1 metadata와 실제 큐 packet의 target 일치, enqueue 실패의 row 제거, annotation 입력 후속 수정과 원 저장 값 분리, ref 유실 시 metadata list 복구를 확인했다. 기존 Rust export row는 공식 목록에서 숨기고 추측한 ref의 consume도 NotFound로 거부한다.

scope/ref 확인→원 Weak→row→saved origin 확인→원 read owner→pending/terminal 순서를 유지한다. origin이 틀리거나 pending이면 row를 삭제하지 않는다. row lock 밖에서 read/consume/원 owner lease 정리를 수행하며, 원 id 재사용 없음과 origin/terminal 상태 불변을 전제로 한다. close 후 원 결과 조회 및 경쟁 terminal consume 하나 성공을 확인했다. 새 map/counter/eviction 또는 Active/route 재선택은 없다.

원64 row/8KiB payload, readOwner256 및 2KiB metadata 정적 charge 검사를 유지했다. 저장 origin은 기존 provenance/원 ticket identity에서 투영하고 annotation/target 고정 값만 추가한다. P0의 native_result_export.rs1만 선언 변경이며 mod.rs·codec/metadata/projection4는 불변이다. P2/C8 forwarding3과 후속 session_engine 경계 조정은 이번 source에 넣지 않았다.

## 보호와 반환

정확7 staging/frozen/snapshot 길이/hash, root before7, root1441 및 snapshot1401 비입력, 소비9/Meari412/P0 원 bundle/Cargo/lock 보호 차이0이다. index hash와 원 Native binary/사본도 불변이다. integrated7.json SHA256 **CB64B0522FE2FA144ABC2584E02A2CC5D5094EB1D502A43E3BA3D0C7369E3950**.

최종 Native libtest163078656bytes, SHA256 **FA12B4FB6ED861CCD5D6B715A4BCAEC44AB95C53B230C2A0F40E47E884FA8044**와 integration 사본을 artifacts/에 보존했다. 원20receipt 전 terminal/exit0, 기록40PID 현재 부재, 본인 원 exec/handle0을 확인했다. closeout.json SHA256 **386C0A11BFA5B1388C762A94A204446ADCE6C95E4581D55819AB1A25B0F554C3**. 필요한 운영 대조 후 상세 문서 작성 전에 PM에 heavy/shared를 명시 반환했으며 추가 Cargo/shared 실행0이다.

원 runner conservative2/2CPU/affinity3/BelowNormal/descendant Job·기존 snapshot/target·offline/locked/-j1/thread1을 유지했다. root 소스/index/commit 채택은 PM 책임이다. 전체 실행 정본은 runtime/quality-review/sdk-native-export-c0-p1-r1/의 receipts.json, actual-tests.json, protection-differences.json, closeout.json 및 handoff.md다.

## 잔여

이번 결과는 P1 standalone 원 row와 Rust API 경계 인수다. P2 explicit original binding/router 전달, wire activation, 전역 등록/출력, 실제 producer CLI 및 mpv ACK/landing은 별도 단계다. UIhost/출력 미완성 overlay는 합성하지 않았다. 제품 지원을 true로 바꾸거나 실제 OS/미디어 성공으로 승격하지 않는다.

[C0 생산 계획](../../../plans/player/native-result-export-c0-producer.md) · [공식 계약](../../contracts/playback/native-result-export/index.md)
