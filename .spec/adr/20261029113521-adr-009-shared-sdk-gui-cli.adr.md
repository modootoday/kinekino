---
id: adr:kinekino/20261029113521
title: GUI·AI CLI의 공통 SDK와 변경 가능한 제품명
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# GUI·AI CLI의 공통 SDK와 변경 가능한 제품명

사용자 확정: 사람용 `player.exe`, AI용 `player-cli.exe`를 제공하고 공통 API/SDK가 설계 기준이다. Source Player는 가제이며 나중에 변경하기 쉬워야 한다.

공개 Rust facade와 typed application 계약을 두 presentation이 공유한다. UI 핸들러를 비즈니스 API로 사용하거나 CLI가 GUI를 클릭하는 구조는 요구에 맞지 않는다. GUI가 있으면 named pipe로 동일 host에 연결하고 없으면 지원 기능에 한해 CLI in-process 코어를 사용한다. 두 정책 구현을 복제하거나 불필요한 세 번째 필수 daemon을 두지 않는다. GUI 스레드·WebView 필요 작업의 headless 제약은 capability로 드러낸다.

이 방식은 SDK 계약·DTO 직렬화·동시성/저장소 잠금 시험을 추가로 요구한다. Rust SDK ABI 안정성을 보장한다는 뜻은 아니며 wire version과 SDK SemVer를 독립 관리한다. 제품명은 주입 가능한 metadata, 지속 ID는 표시명과 독립한다. 기존 실행파일/데이터의 호환 이관은 구현 게이트다.

규범 계약: [공통 SDK](../../sots/player/application/api/sdk-contract.md), [CLI](../../sots/player/presentation/cli/automation-contract.md), [런타임](../../sots/player/infrastructure/ipc/local-runtime.md), [제품 식별](../../sots/player/domain/product/identity.md). 사용자 요구는 승인됐으나 예시 API 세부/한도는 설계 제안이며 구현·실측으로 검증해야 한다.
