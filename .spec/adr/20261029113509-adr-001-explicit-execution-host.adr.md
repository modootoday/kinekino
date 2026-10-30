---
id: adr:kinekino/20261029113509
title: "ADR: jobs worker는 명시 실행 host가 소유한다"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# ADR: jobs worker는 명시 실행 host가 소유한다

상태: 2026-10-19 PM 조율을 통해 채택한 구현 결정. 실제 jobs 구현 성공과 구분한다.

oneshot CLI가 비동기 접수 응답 직후 종료하면 worker 없는 Queued가 남을 수 있다. GUI와 명시 headless CLI host가 같은 공통 JobService를 소유하고, 일반 CLI는 명시 endpoint로 실행 요청한다. read-only local root는 executor가 없으면 submit/retry를 unsupported로 반환한다. 숨은 host 자동 기동이나 상주 서비스는 추가하지 않는다.

취소 요청과 publication 시작은 같은 gate에서 선형화한다. Cancelled는 worker ack 뒤에만 기록하고, publication이 먼저 완료되면 결과 receipt를 보존한다. host가 재시작 ownership을 획득하면 미완료 작업을 Interrupted로 남기며 자동 재실행하지 않는다. 실제 source 본문 snapshot SHA와 현재 파일의 변경/소실 오류도 구분한다.

독립 media-jobs context는 domain 상태와 application 포트만 소유하며 root SDK/GUI에 역의존하지 않는다. 처음에는 기존 SRT/VTT 변환을 재사용하고 frame·AI executor는 실제 취소/검증 adapter가 준비될 때까지 unsupported로 둔다. [정본 계약](../../sots/player/application/api/jobs.md)을 따른다.
