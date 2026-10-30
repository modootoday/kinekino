---
id: sot:kinekino/timeline-production
title: 원본 시간축 증거 생성 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 원본 시간축 증거 생성 계약

## 책임과 현재 구현

공통 Rust infrastructure의 `timeline_evidence::production`은 보관 증거를 읽는 resolver와 별도로 실제 도구 실행의 원 owner와 관측을 유지한다. application/domain의 Verified 판정 정책을 복제하지 않는다. 현재 구현은 영상 프레임과 오디오 clock component이며 전체 issuer는 미완료다. [영상 검증](../../verification/application/timeline-video-production-20261027.md), [공통 실행 수명·오디오 검증](../../verification/application/timeline-audio-production-20261027.md).

입력은 보유 중인 GuardedReviewMedia와 trusted Rust 설정·SHA 고정 PinnedTimelineProbe다. 외부 요청의 임의 경로·도구·JSON receipt를 실행 권한으로 사용하지 않는다. runtime과 원본 저장소는 분리하며 공통 review-runtime.lock과 중앙 무거운 작업 1개 배정을 유지한다. 설치·전역 PATH 탐색이나 모델 다운로드는 수행하지 않는다.

## 수명과 실패

하나의 세션은 원본 lease와 원 deadline을 보유한다. component별 deadline을 갱신하지 않는다. 원 Job 생성 예산을 검증하고 프로세스 생성 후 실패는 원 child/Job을 가진 PendingVideoProbe로 반환한다. caller는 최초 오류를 보존한 채 물리 종료까지 해당 owner를 유지·관측해야 한다. PID로 재결합하거나 Drop을 완료 근거로 사용하지 않는다. 미완료 Drop은 세션을 재사용 불가로 만들며 ABANDONED 표식은 자동 복구하지 않는다.

정상 영상 관측은 원 child exit0·비강제·Job empty, 출력 크기 제한 및 parser 검증이 필요하다. 성공 관측도 source/tool/session borrow를 유지한다. 취소·deadline 실패는 정상 관측으로 발행하지 않는다. 출력 상한은 polling/최종 확인 경계이며 디스크의 강제 quota나 프레임 단위 실시간 preemption을 보장하지 않는다.

## 남은 설계·구현

후속 ProducedTimelineBundle이 source/video/PCM 계보/세 capture를 동시에 보유하고 공통 TimelineContent 규칙으로 결합 mapping을 검증했다. [실제 결합 근거](../../verification/application/timeline-live-bundle-20261027.md). 아직 custody/Verified 권위는 아니다. 다음 연결은 [기존 package 새 버전 발행](live-timeline-custody-publication.md)이며 원 case나 원 모델 결과를 수정하지 않는다.

후속 origin/끝점 capture component는 기존 capture의 공통 인자 plan·PNG/PTS 검증을 재사용하고 원 Job 수명으로 실행한다. [실제 origin·끝점 근거](../../verification/application/timeline-capture-production-20261027.md). 개별 관측은 구현됐지만 이들을 함께 보유한 결합 proof/custody/admission과 제품 startup은 미완료다. 음수 timing·HDR/ICC/player color parity·GUI 시각 인수는 지원/검증으로 승격하지 않는다.

후속 PCM 계보 component는 원본 PCM을 보유한 상태로 raw mono16k/s16le 재생 디코드를 실행해 전체 바이트를 비교하고, 원본 clock의 정확한 sample 범위 클립을 보유한다. [실제 계보 검증](../../verification/application/timeline-pcm-lineage-20261027.md). 파생 입력 guard도 pending owner가 유지한다. 이는 아래에서 남았던 PCM 계보 부분의 후속이며 origin/capture·결합 authority는 아직 미완료다.

후속 원본 PCM component는 raw s16le stdout과 같은 실행 clock의 정확한 sample 수를 검증하고 원 파일을 보유한다. [실제 PCM 근거](../../verification/application/timeline-pcm-production-20261027.md). 이것은 재생·클립 계보의 완료가 아니다. raw 출력의 설정 상한은64MiB이며 기존 clock parser12MiB 한도는 유지한다.

전체 issuer는 영상에 더해 오디오 원본/재생/클립 계보, decoded origin과 범위 끝점 capture, 각각의 원 실행 receipt와 source identity를 함께 보유해야 한다. 이 결합된 증거의 opaque proof·custody 귀속·제품 admission 공급과 CLI startup 소비는 아직 구현되지 않았다. caller DTO, 저장 Verified, archive reader 통과만으로 이 권한을 만들 수 없다. 기존 nomination API는 보존하며 Sprite를 제품 기본값으로 하드코딩하지 않는다.

AI 실제9의 데이터셋·판정과 별개다. GUI의 원본/후보 비교·사용자 명시 품질 판정과 시각 인수는 CLI 선행 인수 이후 수행한다. 수동 GUI·추론·다운로드·WARP·push 보류를 해제하지 않는다.
