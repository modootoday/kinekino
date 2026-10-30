---
id: evidence:kinekino/20261029114224
title: CLI·browser fixture 계약 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# CLI·browser fixture 계약 검토

## 적용 스킬과 범위

[modoo.today](https://modoo.today)의 정확한 스킬 `modootoday/software-qa/test-fixture-and-mock-build`를 skill_find로 검색하고 설명의 적합성을 확인한 뒤 skill_call로 본문을 읽었다. QA package로 좁히기 전 상위 후보인 portable-demo·법률 versioning·frontend-first는 현재 Rust SDK/CLI 계약 검토에 맞지 않아 채택하지 않았다. 출처 도구는 mcp__modoo_today_skills__skill_find/skill_call이며 runtime 검증 표시가 없는 QA 스킬을 검증된 runtime이라고 주장하지 않는다.

본문의 정의 기반 mock, 지연/오류, setup/teardown 후 재획득, 성능 주장의 실제 측정 및 mock 한계를 적용한다. 다른 언어 예제·새 인프라·추론/GUI gate·추가 승인을 도입하지 않는다. 진행 중인 정상 검사는 유지하고 수정은 자연 경계 뒤 등록한다.

## 발견과 수정

기존 SyntheticBrowser는 poll/cancel/release의 ticket을 무시하고, 활성 work를 덮어쓸 수 있으며 즉시 성공했다. 기존 검증의 증거 범위는 실제 저장소/CLI wire·해시·재오픈이며 실제 browser 수명이나 오류를 이 fake로 검증했다고 보지 않는다.

준비한 Rust synthetic_browser.rs는 공통 BrowserWork를 재사용하고 foreign/released ticket=not_found, 활성/보존 한도=resource_exhausted, 활성 release=revision_conflict, stopped host=needs_input을 구별한다. Queued→Running→capture/failure를 poll step으로 나누고 queued-poll 지연 knob·engine_failed 주입을 제공한다. cancel은 Cancelling으로 접수하고 다음 worker step에서 Cancelled를 확인한다. 세대는 재획득 뒤 증가하며 기존 SDK/CLI 파일 검증 assertion을 제거하지 않는다.

| 경계 | 근거/한계 |
| --- | --- |
| mock 성공·ticket 격리·취소·자원 재획득 | 새 scoped fake 회귀 등록 후 jobs 5pass/3ignored, fmt/strictClippy exit0. 실제 엔진 증거로 확대하지 않는다 |
| 실제 bounded queue full/disconnect/poison·DOM 초과 | production Handle/decoder의 network 없는 Rust 회귀. fake로 대체하지 않는다 |
| 실제 WebView2·사이트·네트워크 timing | 고정 CLI의 별도 ignored 관측이 권위다. synthetic-no-engine 결과로 대체하지 않는다 |
| 실제 종료 후 자원 복구 | 실제3category 기능 실패 수집 뒤 정상 EOF/이전 endpoint 폐쇄/같은 jobs·catalog·profile root의 새 소유 host 개폐 assertion 통과. 두 stop receipt의 worker_exited=true. 복구 host에서는 새 사이트 요청0 |
| 초기화 중 영구 COM 대기·default dialog/download 실패 주입 | 여전히 미검증. 전체 OS 자식 종료로 확대하지 않는다 |

원문 변이/손상 테스트는 각 테스트가 만든 private fixture root만 변경한다. 실제 r2 golden DOM은 read-only로 재사용하고 원 capture revision을 보존한다. raw reader는 매번 전체 SHA를 검증하므로 이 검증을 속도 목적으로 캐시하거나 생략하지 않는다.

## 실행과 속도

현재 catalog-parser-raw-r1의 정상 full 검사를 스킬 읽기 때문에 중단/재실행하지 않는다. source 수정 뒤 변경 fixture의 jobs target 및 fmt/all-target strictClippy를 필요한 범위로 검사한다. 이전 full-product source와 fixture 보완 source의 검증 범위를 분리한다. 속도 향상은 측정하지 않았고 주장하지 않는다. 전후 동일 명령 후보는 `cargo test --test jobs provider::synthetic_browser_job_uses_actual_files_and_cli_reopens_verified_result -- --exact`이며, 정확한 baseline이 없으므로 추정 비율/예상 시간을 쓰지 않는다.

후속 focused 검사는 fixture-clippy.log의 1.33초 exit0, fixture-test.log의 5pass/3ignored·18.82초를 확인했다. parser/rawreader 제품 코드 변경 없이 fixture만 보완했으며 기존 full-product 218pass/12ignored를 중복 실행하지 않았다. 보존 DOM offline 해석 및 실제 고정 CLI raw readback도 1pass/7.06초이며 새 네트워크 요청0이다.
