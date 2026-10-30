---
id: plan:kinekino/20261029113524
title: 영속 jobs 첫 수직 구현
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 영속 jobs 첫 수직 구현

상태: 첫 SRT/VTT 수직 구현과 root SDK/명시 CLI host 통합을 완료했다. [고정 CLI·전체185/9·최종검사·한계](../../sots/verification/application/jobs-2026-10-19.md), [jobs 계약](../../sots/player/application/api/jobs.md), [ADR](../../adrs/jobs/adr-001-explicit-execution-host.md)을 따른다. 아래 단계는 이 checkpoint의 인수 순서이며 frame·모델 등 후속 executor 완료를 뜻하지 않는다. GUI 수동 검토는 선행 조건이 아니다.

1. 미등록 media-jobs에 domain 상태/attempt/취소·완료 경합과 executor/store/event 포트를 분리한다. root 역의존 없이 순수 테스트로 전이·terminal 불변을 검사한다.
2. 명시 host ownership·bounded immutable journal/source blob·read/list/event cursor를 구현한다. torn/hash/gap/중복 admission·크기 거부와 재시작 Interrupted/무자동 실행을 검사한다.
3. 기존 SRT/VTT 변환의 실제 Rust executor를 연결한다. 원본 snapshot·변경 source 오류·손실 NeedsInput·no-overwrite·publication 이전 cancel ack·완료 경합 receipt를 검사한다. AI와 frame은 미지원 유지한다.
4. root SDK/API/schema/typed client/명시 headless CLI host를 thin adapter로 연결한다. 기본 unsupported, read-only local root, endpoint execution capability 차이와 기존60 command·project/proposal wire 불변을 검사한다.
5. fmt/test/strict Clippy/bins 후 고정 CLI/host에서 실제 submit→get/list/watch→cancel/retry→재연결 cursor→종료/재시작을 검증한다. 실패 원문·source SHA·host/CLI pin을 보존한 뒤 UI에 별도 fixture를 인계한다.

완료 기준은 실제 source 변환 결과와 영속 상태/취소 ack/재시작 관측이다. 타입·화면·Queued 응답만으로 완료하지 않는다. 새 모델 추론·다운로드, 사용자 앱 종료, VM/DPI 조작은 필요하지 않다.
