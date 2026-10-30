---
id: adr:kinekino/20261029113534
title: owned-test-runner 개발지원 패키지 경계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# owned-test-runner 개발지원 패키지 경계

Status: Proposed, 2026-10-24. PM 인수·구현·공개 승인 아님. 사실·단계·검증·rollback의 정본은 [통합 계획](../../plans/player/production/owned-test-runner-integration-boundary-2026-10-24.md)이다.

공통 owned-process는 child-family 소유권/물리 종료를 담당하고 caller의 protocol 의미를 소유하지 않는다. runner는 고정 artifact/정확 libtest 선택·원 cutoff·결과 성공/실패를 판단한다. CPU·priority launcher resource-runner와도 책임이 다르다. 현재 runner 후보는 runtime/S에 있으며 canonical tools에는 없다.

| 대안 | 장점 | 비용/판단 |
| --- | --- | --- |
| tools standalone binary 유지 | 최소 경로변경, 기존 CLI gate와 가까움 | lib API 소비/CLI 단독정책 분리 필요. 검증 checkpoint로는 유효 |
| owned-process에 CLI/lib 흡수 | 기존 package 한 개 | SDK에 libtest/JSON/artifact 성공 정책이 유입돼 media-vad-executor 등 실제 caller 의미와 충돌하므로 비추천 |
| resource-runner에 timeout/시험정책 추가 | launcher 한 개 | CPU admission과 원 가족 종료/시험판정을 섞고 기존 no-timeout 계약을 바꾸므로 비추천 |
| packages/owned-test-runner 한 lib+bin | SDK owner 유지, CLI와 Rust caller 같은 test usecase, 책임별 공개 root | 신규 root/권리·source graph·cfg·경로 영향 재검증 필요. 추천 |
| test SDK와 CLI를 두 새 crate로 분리 | 독립 배포 가능 | 실제 복수caller0인 현재에는 추가 workspace/버전 경계 비용, 유보 |

추천은 publish=false/독립 workspace의 단일 packages lib+bin이며 root default/release graph에는 자동 편입하지 않는다. 이는 제안이고, 단순 폴더 이동이 아니라 application API와 원 owner lifetime의 공통화가 완료 조건이다. SDK Complete를 성공으로 재정의하거나 GUI/제품 player-cli 기능을 개발 도구로 우회하지 않는다. license/secret 검토와 forced actual gate가 미완료이므로 지금 공개 채택하지 않는다.

재검토 조건은 실제 SDK caller 추가, root workspace 통합 필요, Windows 외 실지원 요구, 독립 배포/권리 확정이다. 권리 결정을 포함한 미검증 사항은 계획에 남겼다. architecture trade-off 스킬의 단순 대안·비용 비교를 적용했다.
