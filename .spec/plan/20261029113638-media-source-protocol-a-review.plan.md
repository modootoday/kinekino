---
id: plan:kinekino/20261029113638
title: Source protocol A exact6 읽기 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Source protocol A exact6 읽기 검토

연구 담당이 source6 SHA F0496B800F8C7A34EEA5A306507BB3A93E629635821BA8A3BEA348108A9CC7AF 및 raw6 읽기 전후 일치를 확인했다. PM은 HANDOFF 전체를 읽었다. 확정 P1/P2 발견0이며 타입·실제 검증0이다.

고정 header kind/length 검증 후 bounded 할당·body read, trailing/truncated/boundary EOF 구분, clone 후보에 상태 변경 후 성공 반영과 오류 poison, 방향별 sequence/offset/credit checked 연산·원 absolute cutoff 비연장·clock rollback/terminal 재개 거절을 조사했다. 작성12는 메모리 fixture이며 실제 allocator/IPC/OS/DNS 측정이 아니다.

wire binding은 권한이 아니며 BodyEnd/EOF는 physical ACK가 아니다. 동기 Read는 별도 blocking 정책이 보장된 reader용이며 취소 가능한 부모 pipe 구현으로 사용할 수 없다. URL/header/replay issuer·OSclock·실제 source는 미구현이다. A 실행 배정은 아직 없고 UI가 soleheavy다. 새 package workspace/lock 입장은 별도 exact gate가 필요하다.

기존 modoo QA 본문·필수 verdict 참조의 원 호출 경로와 작성/실제/권한 경계 분리 기준을 적용했다. 연구자 수정·빌드·실행0, 구현은 Media 담당 소유다.
