---
id: sot:kinekino/source-sdk-cli
title: 숏폼 source 관측 SDK·CLI 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 숏폼 source 관측 SDK·CLI 계약

현재 검증 범위 R164: [실제 검증 정본](../../../verification/application/clip-source-inspection-api-20261028.md)에 source 관측부터 새 프로젝트 작성/수정 저장·원 Jobs 렌더까지의13 CLI/typed 여정과112 발견/schema·통합 gate를 기록했다. 아래 실행 전 기술은 최초 설계 시점의 기록이다. 수동 recipe/preset·미지원 렌더 기능·전체 오류/취소/복구·GUI/색/AI 사용자 품질 인수는 별도 미완료로 유지한다.

2026-10-28 중앙 PM, R163 private overlay WIP. [원 Jobs](clip-original-jobs.md)의 입력 관측을 제품에 노출한다. 원 소유권·WIP·첫 실패·보류를 유지한다.

`clip_sources.inspect`는 strict media_id payload만 받는다. media ID는 pinned 로컬 host의 고정 source catalog에 연결하며 호출자가 경로·URL·도구·컨테이너·색 선언을 주입하지 못한다. 기본 SDK에는 port가 없어서 Unsupported이고 명시 clip-render host에만 연결한다. application의 ClipSourceInspector가 native FfmpegClipSources를 호출하고 원 source/tool/ancestor lease를 유지한 동안 관측을 복사해 반환한다.

응답 ClipSourceObservation은 source SHA·duration·크기·정수 clock/관측 SHA·전체 track, 기준 stream index·duration_basis·raw probe/tool SHA·관측된 transfer/rotation이다. 기존 ProbeFacts/DurationBasis를 application으로 옮기고 infrastructure는 호환 alias를 유지한다. 관측 JSON은 파일 보유 권한이나 encoder admission이 아니다. 렌더는 저장 source pin과 실제 원 파일·도구·경계를 다시 검증한다. ContainerReportedSpan은 마지막 decoded frame이나 EOF 증거가 아니며 색 정확도나 품질 승인도 아니다.

context는 timeout 1..120000ms(기본30000)만 수용하며 idempotency/expected_revision은 거절한다. native hash/probe의 기존 MutationControl checkpoint에 원 deadline을 전달한다. 별도 worker나 취소 중재는 없고 in-flight cancel 지원으로 표시하지 않는다. typed SDK inspect_clip_source는 같은 command를 사용한다. source schema는 clip project의 source pin schema와 공통 함수로 공유한다. 발견 command는112개가 되지만 command 수를 제품 인수율로 사용하지 않는다.

실제 검증 여정은 원 저장 프로젝트를 단지 수동 edit recipe로 읽고 새 project root를 준비한다. 실행 중인 CLI에서 source를 관측하고 typed 결과와 대조한 뒤 그 응답의 source pin으로 새 프로젝트 생성·수정 저장·pin 조회·원 Jobs 렌더를 수행한다. 임의 path/잘못된 ID 거절도 검증한다. 아직 실행 전이며 recipe preset의 출처·품질 승인·모든 사용자 편집 기능까지 완료한 것으로 표시하지 않는다.

기존 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100), code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)의 fixture/실제 관측·실행 근거 분리 원칙을 적용한다.
