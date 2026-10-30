---
id: plan:kinekino/20261029113613
title: 자막 CLI 실제 batch 실행 예산 준비
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 자막 CLI 실제 batch 실행 예산 준비

후속 결과: 실제 batch는 outer terminal0/15486ms와 exact1pass·inner6 기대결과/nonforced Empty로 완료됐다. [actual-r1 정본](../../sots/verification/proposals/accepted-subtitle-export-actual-r1-2026-10-24.md)에 출력3·원상태 불변·EOF/PID/보호 반환 근거를 보존한다. 이후 통합은 a6ca29c에 커밋됐다. 아래의 “실행 전/아직0/다음 단계”는 당시 예산 준비 시점 기록이며 현재 미실행 상태를 뜻하지 않는다. 이 관측은 예산 충분성의 일반 보장이나 실제 방송·ASR 품질 승인이 아니다.

2026-10-24 PM 읽기 조사. 후보는 subtitle r3 effective13 `7C253957…`이며 r5 CPU 실행과 실제 CLI 실행을 구분한다. 연구 서브에이전트가 소스만 조사했고 PM이 r3 CLI leaf의 invoke6·각 Duration15s·사후 검사를 대조했다. build·실제 batch 결과가 아니다.

setup은 직접 SDK/file 작업으로 child0, batch는 SRT/VTT/기존출력충돌/손실미승인/손실승인/붕괴구간거절의 CLI 최대6회다. 각15s 안에 operation13s와 cleanup reserve2s가 포함돼 합계 최대90s이며 reserve를 다시 더하지 않는다. 서로 다른 호출의 원 family/Supervisor/receipt와 첫 오류를 유지한다.

setup의 CLI SHA 검사·저장소 준비·proposal3 create/accept/inspect, 요청 JSON 쓰기와 디렉터리 생성, helper 반환 후 raw 저장/JSON/출력 검사, 최종 proposal 재inspect는 inner deadline 밖이다. 크기와 횟수 제한은 있으나 동기 파일/SDK 작업의 벽시계 상한은 소스상 없다. 따라서 budget180 후보의 180초는 입장 상한이며 batch 전체 완료를 보장하는 근거가 아니다.

다음 단계는 SDK r5 반환 → 원 standalone runner budget180 CPU/producer 검증 → 실제 batch다. PM은 앞 두 정상 반환을 SDK가 직접 인수한 경우에 한해 SDK 다음 soleheavy로 exact ignored1(batch6)을 1회 실행하도록 조건부 배정했다. outer total180000ms/cleanup10000ms(operation170000ms), setup 포함 원 단일 deadline을 선택했으며 충분성을 보장하지 않는다. 배치 실행은 아직0이다. 내부15/2를 단축하거나 호출 관계를 분할하거나 기한·owner를 갱신해 성공을 만들지 않는다. outer 기한 초과·force·원 cleanup 실패는 보존하고 정상 완료와 구분한다. 각 호출의 원 결과·출력 보존·원 revision 불변과 전체 owner/EOF 증거를 실제로 확인한다.

입력은 r5 독립 test SHA2473B07A…/3920384B와 normal CLI SHAA0822DF0…/32345088B이며 06-fixed6.json의 실제 Cargo profile/features/독립copy 경로를 따른다. 새 runner는 Media CPU의 정상 fixed producer만 사용하고 이전02DD/C2/가변alias로 대체하지 않는다. 새 baseline JSON에 source21·producer·runner source8·최신carry·env3/예산을 관측과 operator 선언으로 구분해 고정한다. driver evidence는 별도 새 디렉터리, ACCEPTED_SUBTITLE_EXPORT_EVIDENCE_DIR는 절대경로·원부재, ACCEPTED_SUBTITLE_EXPORT_CLI/SHA256은 동일 fixed normal을 사용한다. Media CPU 반환 전 actual0, 실패 시 후속 미진입, SDK가 exact 결과·owner/EOF/새carry·짧은 SOT로 반환한다. source/Cargo/새build/canonical/stage0을 유지한다.
