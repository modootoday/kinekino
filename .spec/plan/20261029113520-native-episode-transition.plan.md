---
id: plan:kinekino/20261029113520
title: 저장 목록에 결합한 실제 회차 전환
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 저장 목록에 결합한 실제 회차 전환

기준은 [옵션 r3 실제 인수](../../sots/verification/application/native-options-cli-2026-10-19.md)다. 준비20파일의 당시 책임·포맷 관측·미검증 경계는 [준비 관측 정본](../../sots/verification/application/native-transition-preparation-2026-10-19.md)에 분리했다. 등록 이후 결과는 [실행 정본](../../sots/verification/application/native-transition-cli-2026-10-19.md)에 기록한다. 이전 누적 메모는 runtime/coordination/native-transition/plan-before-consolidation-2026-10-19.md에 보존했다.

Meari와 UI 최종 검증 및 동결 반환 뒤 중앙에서 SDK 단일 heavy 슬롯과 root 등록을 승인했다. 준비20파일을 책임별로 등록하고 기존 파일은 문맥 패치로 연결했다. 등록 전 입력과 준비본은 runtime/quality-review/native-transition-cli-r1/source-before 및 pending-source에 보존했다. 첫 fmt 차이는 정리했으며 전체 타깃 cargo check는 성공했다. unused 필드/접근자 경고 두 건을 제거한 뒤 transition 필터 테스트는 12개 통과했다. 이 중 새 전환 검사는 8개이며 기존 검사 4개가 포함된다. 준비한 startup/출력 진단 검사까지 모두 통과했다는 뜻은 아니다. 전체 test·strict·최종 fmt·bin pin·실제 인접 두 회차 검증은 진행 중이다. fresh CLI/provenance/product 등록 잔여 범위를 유지한다.

1. 준비 application/infra/API/client를 책임별 목표 경로에 등록한다. 기존 EpisodeQueue·CatalogResults·EpisodeResolutionResults·JobsApi·native owner를 재사용하고 source 생성 가시성을 넓히지 않는다.

[등록 경로·공유 영향·최소 강한 gate 원문](native-transition-checkpoint.md)을 다음 checkpoint 인계로 사용한다. API root 후보는 전체 덮어쓰기 대상이 아니다.
2. root SDK router/capabilities/schema·typed client 및 GUI/CLI 공통 host 조립을 연결한다. 현재 별도 native/provider mode만으로 자동 이전/다음 회차가 완성됐다고 표시하지 않는다.

SDK 등록 지점은 api/mod.rs의 Option<Arc<EpisodeTransitions>> 필드/기본None/각 generic builder 전달/with_episode_transitions, request.validate 뒤 전환 execute·schema 분기, capabilities 마지막 갱신이다. global COMMANDS에 새 transition.observe/commit/release3개를 추가한다(previous/next는 이미 있다). 지원 여부는 같은 Option에서 산출하고 호스트 없는 schema도 unsupported로 조회 가능해야 한다. api.pending의 공통 COMMANDS와 capabilities 갱신 helper를 준비했으며 실제 router 등록은 하지 않았다. client 확장은 client child로 등록·재수출하고 runner는 serve-playback 분기를 기존 모드와 중복 거부하도록 연결한다.
3. 전환 시 paused/volume/rate/track·자동 다음 회차 정책을 공통 계약으로 확정한다. URL 산술·제목 추정·임의 target/실행 옵션·첫/끝 자동 순환을 사용하지 않는다.
4. 등록 뒤 준비한 Rust 검사12개와 old-close/new-launch 실패·EOF 경합·stale instance/artifact·다른 job/attempt/target·해시/크기 손상·slot 소비/종료 검사를 실행한다. owned_startup2개는 Rust child probe/worker 실패·정리 미확인 recovery 재시도 검사이며 아직 미실행이다. 실제 native engine/API 소유까지의 fault 인수도 남는다.
5. cargo fmt --check, cargo test, cargo clippy --all-targets -- -D warnings 및 두 bin pin을 수행한다. 단일 heavy·두 CPU·BelowNormal·offline/locked/-j1·단일 테스트 thread와 기존 cache를 유지한다.
6. 실제 목록의 서로 다른 인접 두 회차로 fresh resolve→기존 엔진 유지→원자 commit→old 종료→정확 대상 descriptor→새 영상·시간 진행·전체 PNG→old ID 거부→options/4controls→EOF를 검증한다. 기존 capture4초 게시기한과 실패 원문을 유지한다.

접수·resolve 완료·프로세스 생성·실제 재생·사람 품질 승인은 별개다. 목록 완전성, 전체 회차, 청취, subtitle 양성, 정확 PTS/색상, GUI 시각 일치 및 새 모델 추론/다운로드의 성공으로 확대하지 않는다.
