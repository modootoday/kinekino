---
id: evidence:kinekino/20261029114311
title: 原 Supervisor mutable cleanup CPU 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 原 Supervisor mutable cleanup CPU 검증

2026-10-21 root22cccd7/library0b79d57 기준. source3 SHA256 `FB29E8612CC708970FCE1E96B0B46AAF3CB4DE2650C01E9BF8BECF2B1B581736`는 원 owner를 소비하지 않는 begin/poll/record cleanup을 추가한다. 기존 finish 본문은 유지했다. 원 deadline 단축, EOF 요청1·force1, force 이후 별도 원 child/family 관측, 최초/제한 후속 오류 및 partial/final receipt 구분을 검증한다.

원 package manifest의 독립 snapshot에서 offline metadata·fmt·all-target strict와 고정 BIN 시험이 통과했다. 실제 목록/실행은 library2 + 기존 state9 + 신규 cleanup9 =20이며 실패/ignored0, 소스 수리0이다. 실제 `Supervisor<WindowsFamily>: Send` 정적 타입도 컴파일됐고 unsafe Send 구현을 추가하지 않았다. test-support·실제 child/GUI/model 실행은0이다.

Cargo-generated lock SHA256 `D9B51D989D58E64CFE5CC55DABB6A80260BD3F37D440BA86EC7BFC8FD0FA90D3`는 불필요한 root workspace 항목을 제거했으며 선택 registry15의 version/source/checksum은 원 lock과 동일하다. 원 receipt10 terminal0, 기록 PID20 현재 부재, pending exec0을 확인했다. root1506/동결3/독립 source/index/이전 BIN 보호 차이0이며 허가한 generated lock만 변경됐다. heavy/shared는 문서 작성 전에 반환했다.

원 목록·결과·BIN pin·lock tuple·보호·receipt는 [검증 인계](../../../../runtime/verification/supervisor-polled-cleanup-r1/HANDOFF.md)에 보존한다. 이 CPU 검증은 실제 VAD worker 종료·원 stdout/stderr EOF·AI 취소 ACK·모델 품질·제품 지원을 증명하지 않는다. Media adapter 실제 소비와 P1b 전체 caller/lease/게시 연결은 별도 검증 대상이다.
