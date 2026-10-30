---
id: sot:kinekino/private-http-metadata-v1-payload
title: Private HTTP Metadata v1 payload
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Private HTTP Metadata v1 payload

2026-10-27 중앙 PM private 초안이다. 제품 root 미채택이며 기존 MSH1 v1 Kind::Metadata의 opaque payload 안에만 넣는다. binding/sequence/offset/credit/cutoff는 외부 MSH1과 원 Exchange의 책임이다.

## Layout

정수는 little-endian이며 고정 header는20byte다.0..4=`MHM1`,4..6=version1,6=flags(bit0:length present,bit1:type present),7=reserved0,8..16=response length u64,16..18=URL byte length u16,18..20=type byte length u16이다. 뒤에 UTF8 URL과 ASCII Content-Type을 넣는다. unknown flags/version/reserved, truncated/trailing, 모순된 presence/length를 거절한다. length absent이면 u64 값도0이어야 한다. Some(0)과 None은 다르다.

URL은1..8140byte,HTTPS/host 필수,username/password/fragment 없음이다. type은 존재하면1..256byte printable ASCII이며 HTML을 거절한다. 합계 최대8416byte로 기존 Metadata32KiB 상한 이내다. 가변 문자열은 선언 길이 검사 뒤 parse/copy한다. URL query는 민감 정보일 수 있으므로 Debug/Serialize/argv/diagnostic에 출력하지 않는다.

## Meaning

whole response·identity encoding의 worker 관측을 전달하는 v1이며 trusted SHA는 없다. parent는 원 admitted URL/maximum과 대조하고 원 browser current/attempt/one-use replay grant·media probe·CAS·실제 physical close를 별도로 확인한다. worker payload 생성/수신을 source 권한·품질 승인·physical ACK로 삼지 않는다. status200·range 없음·encoding 등 worker local 검사를 이 payload만으로 외부 관측했다고 간주하지 않는다.

구현/검증 선택 입력·실패 이력은 [private 후보 기록](../../verification/application/source-http-bootstrap-private-20261027.md)에 둔다. 기존 public/root 계약을 바꾸지 않으며 후속 parent/worker 통합까지 private 초안으로 유지한다.
