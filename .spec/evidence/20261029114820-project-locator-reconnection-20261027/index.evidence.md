---
id: evidence:kinekino/20261029114820
title: 프로젝트 locator 재연결 — 2026-10-27 영향 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 프로젝트 locator 재연결 — 2026-10-27 영향 검증

중앙 PM의 private overlay 후속이다. canonical 적용·미디어 실체 재결합·AI 품질·GUI 인수를 주장하지 않는다. [계약](../../player/application/api/projects.md)의 locator-only 범위를 구현했다.

## 관측

### 시험 전용 후속 — 오류 입력·typed SDK

757은 exit0/42016ms로 종료했고 전체2244 입력 content/identity 동일을 확인했다. 아래 진행 중 문장은 종료 전 이력이다.

제품 정책 코드는 유지하고 `tests/projects.rs`만 확장했다. preimage SHA256=E28CBAA4B895C953D830048199B729E94750EF45543D6691083ADB8AB676218E, 후속 시험 SHA256=E8BA46257CEA40FD2F4568C6B742BA52A18E75587A0C14BD41841F3F78A63DD8. preimage는 `runtime/verification/project-reconnect-errors-preimage-r1/projects.rs`다. 751의 첫 빌드는 시험 내부 moved Value 재사용(E0382)으로 exit101; clone 수정 후 752 no-run exit0를 확인했다. 원 실패는 보존했다.

753 동일 이름 시험1/1은 실제CLI19회와 typed ApplicationClient 재연결 revision3을 통과했다(exit0/4087ms). 합성 locator/제안 저장소이며 실제 미디어 검증은 아니다. 상대경로·source kind 변경·인증정보 URL·unknown payload·누락/0 revision·미지원 timeout/idempotency를 거절하고 최신 revision2가 그대로인지 재조회했다. typed 호출 뒤 revision3의 proposal snapshots를 원 상태와 대조하고 fresh CLI inspect 전체가 typed 응답과 일치함을 확인했다. 증거는 `runtime/verification/project-reconnect-errors-cli-r1/`의 CLI SHA와 요청/응답/exit에 남겼다. 754 해당 Native test strict와 755 fmt는 exit0다. historical 연결·watching 부재·episode 정상 경로·전체 실패 행렬은 미검증이다.

756은 새2244 입력을 고정했다. SOURCE.md SHA256=86C9C11CAE07A8EFF385F4BEA2DE8E29261560948C0AFA27FBCF12A187A2376A, inventory SHA256=BCB464853C313D68B5DEFE503F29E377CFB6D08B8C79A7A5CD546F33C7476D9E. `runtime/verification/source375-project-reconnect-errors-input-r1/`에 보존했다. 757 전후 동일 확인은 진행 중이다. 전체190·정상bins·전체 all-targets strict를 시험 전용 변경 뒤 재실행한 것으로 계상하지 않으며 아래 SOURCE268E3482의 원 실행으로 유지한다. 기존 읽은 modoo.today fixture 스킬의 입력 격리/합성·실제 구분을 재적용했다.

- `735-project-reconnect-check`: native feature 제품 all-targets check exit0/39002ms. Meari paint의 기존 deprecated 경고는 보존했다.
- `736-project-reconnect-no-run`: projects 실제 시험 빌드 exit0/86391ms.
- `737-project-reconnect-actual`: `reconnect_locator_preserves_history_and_rejects_stale_selection` 1/1, 실제 CLI9회, exit0/2506ms. 실행 근거는 `runtime/verification/source375-datalab-b0-r1-logs/`의 같은 prefix stdout/stderr/receipt이며 격리 CLI·SHA·요청·응답·exit는 `runtime/verification/project-reconnect-cli-r1/`에 보존했다.
- `738-project-reconnect-fmt`: cargo fmt --check exit0.

합성 제안 snapshot을 저장한 프로젝트에서 D의 과거 locator를 C의 새 locator로 갱신했다. 실제 D를 접근하지 않으며 캐시 정책 변경이 아니다. 새 revision2의 title·제안 전체·work state와 과거 revision1을 비교했다. stale project revision, stale expected_source, 다른 source_id를 거절한 뒤 최신 state가 동일함을 실제 fresh CLI로 확인했다. 응답의 content_identity_verified/source_opened/playback_rebound는 모두 false다. schema가 새 payload/응답을 노출하는 것도 확인했다.

## 입력·미완료

### 후속 통합 종료

742 no-run은 exit0/114603ms로 종료했다. 743의 최초 exact 배정은 상위 로그 디렉터리 부재(os3)로 시험 실행 전에 중단했으며 원 stdout/stderr/receipt를 보존했다. 상위 디렉터리 생성 후 744 Product145·745 Core18·746 Native26·747 OS1을 실제 직렬 실행해 모두 통과했다. 각 group-return의 execution_result/producer_integrity=Ok와 SOURCE268E3482 귀속을 확인했다. Meari producer는 원622/624 Cargo JSON의 실제 파일을 재해시/식별 검증해 재사용했고 시험 자체는 이번에 다시 실행했다.

748 기존 프로젝트 시험1/1은 실제CLI11회 및 typed inspect를 통과했다. 749 정상bins2는 profile.test=false/fresh=true, exit0/6457ms였다. player SHA256=82DF7DC5B0C403C86D06297D3F2787FD0CAC53B2CEEFE366492CA61865A49269, player-cli SHA256=1DF8ADD0F6210B4E7C38CEC46943D0EA6647530086902DCB752752D4C735378D. 750 전체2244 입력 content/identity 동일은 exit0/42092ms다. root/Meari lock 해시는 B2E3D540665FFC8E68D24D0A6C531D2B983658FB19B5ABBA525659BA1ACC371D / 23792484F39356E19904DBBB34B1496979E87793CF035491EF71F7C4134CC1EE로 유지됐다. 아래 진행 중/미실행 문장은 통합 전 관측 이력이며 이 후속 종료 결과를 따른다.

preimage는 `runtime/verification/source375-project-reconnect-preimage-r1/`에 보존했다. snapshot739는 2244개 입력을 고정했다. SOURCE.md SHA256=268E3482D996A718ED2B876CBB048736D043772105415691B755EA32DEB2028F, inventory SHA256=79C67DC42A8D0571837A435EA589DF3B5D78DCEFE0BB1F074FE895DA1D0A8B46이며 `runtime/verification/source375-project-reconnect-input-r1/`에 있다. 도구의 generic canonical 제목은 overlay의 canonical 적용 승인이 아니다. 740 Native strict exit0/41752ms, 741 기본 strict exit0/30960ms다. 742 Native lib/jobs/projects no-run은 실행 중이며 등록190·정상2bins·전체 입력 전후 content/identity 검증은 미실행이다. 이전 SOURCE7144F249의190/정상bins를 현재 변경의 성공으로 승계하지 않는다. 위 시험은 전체 프로젝트 실패 행렬, historical 연결 회귀, typed reconnect 호출, 실제 미디어 존재/내용, 실행 중 취소 및 GUI를 증명하지 않는다. 후속에서 해당 공백을 구별해 검증한다.

modoo.today `modootoday/software-qa/test-fixture-and-mock-build` 본문 SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100을 읽고 합성 입력/실제 CLI 구분과 격리·원 상태 비교를 적용했다. 원 WIP·소유권·첫 실패·보류 유지, 설치·다운로드·추론·수동GUI·WARP·push 없음.
