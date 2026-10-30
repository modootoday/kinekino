---
id: sot:kinekino/legacy-api
title: 카탈로그·회차·legacy 재생의 공통 API 이관 자료
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 카탈로그·회차·legacy 재생의 공통 API 이관 자료

2026-10-19 PM 배정의 읽기 전용 조사다. 제품 src/API를 수정하거나 사이트·브라우저·재생·다운로드·추론을 실행하지 않았다. 아래 현재 상태는 Rust source를 읽은 사실이며, 제안 포트와 인수 조건은 구현 완료나 새 실기 통과가 아니다. SDK/UI 소유 세션이 제품 구현에 사용할 경계 자료다.

[정적 source 표본17파일과 SHA 원장](../../../../../evidence/verification/catalog-episode-boundary-static-2026-10-19/)을 별도 보존했다. 이는 읽은 경계의 source 표본이며 전체 제품 source/compiler closure나 새 실행 결과가 아니다. 현재 source 링크는 후속 구현에 따라 달라질 수 있다.

## 현재 지원 차이

| 범위 | 읽은 코드의 상태 | 공통 API에서 남은 일 |
|---|---|---|
| catalog.search | GUI Rust는 카테고리·주소 탐색을 제공. 사이트 검색 UI의 과거 관측은 있으나 Rust CatalogQuery/Work 결과 adapter는 없음 | provider query/result 포트·실제 source 수집·schema·capability·오류 계약 |
| episodes.list | JS가 DOM 회차 href/제목을 읽어 EpisodePage를 전달하고 Rust가 검증·현재 목록을 관리 | 관측 snapshot을 공통 application 저장/조회로 이동. 미관측과 실제 빈 목록 구별 |
| episodes.resolve | 허용 페이지·팝업·응답에서 media 후보를 관측. GUI의 바로 보기 클릭·callback에 결합 | 원본 회차→중개→embed→media의 ticket/session/provenance 포트. 후보 관측을 resolve 완료로 과장하지 않음 |
| playback.open/close/engine/previous/next | GUI가 Browser·mpv 생성/종료·route·회차 전환을 조합 | lifecycle/queue/route 정책을 application controller로 이동 |
| playback.status/play/pause/seek/set_volume | 공통 status/control port와 정상 main 조립이 있음 | 기존 포트 재사용. queued acceptance와 실제 적용/재생 완료를 구별 |

capabilities의 command 이름 등록은 구현 지원이 아니다. [현재 지원 판정](../../../../../../src/application/api/capabilities.rs)과 [SDK 조립](../../../../../../src/application/api/mod.rs)에는 catalog/episodes 및 위 legacy lifecycle 포트가 없다. 나머지 playback 제어·AI/jobs/models 지원까지 이 조사로 확대하지 않는다. 기본 in-process CLI의 provider 부재와 GUI host endpoint의 공통 SDK 호출도 구별한다.

## 책임별 이관 제안

| application 포트/유스케이스 후보 | 옮길 책임 | infrastructure에 남길 책임 |
|---|---|---|
| CatalogPort | category/query/filter/page의 검증, 안정 provider/work 식별, 결과/오류/미지원 범위 | 검증된 사이트 query 매핑과 Rust HTTP/HTML 또는 browser adapter |
| EpisodeCatalogPort | work와 source snapshot별 목록, 중복/순서/current/경계 | DOM/응답 추출·출처 관측 |
| EpisodeResolvePort | 요청 ticket·원본회차·generation·deadline/cancel·후보 판정·source lease | STA browser session, 요청/응답·프레임·origin 관측 |
| PlaybackSessionPort | open/close·backend 선택·이전/다음·일시정지 의도·전환 상태 | mpv 소유 프로세스/IPC, browser media bridge, surface 연결 |
| 기존 PlaybackStatusPort/PlaybackControlPort | 관측 DTO와 bounded typed 제어 재사용 | 실제 IPC 수집·큐·engine instance 바인딩 |

이 이름들은 설계 후보이며 아직 공개 Rust trait이나 executable API schema가 아니다. 기존 [목표 데이터 계약](../../../architecture/sot-002-product-architecture.md)과 [SDK 계약](../../../application/api/sdk-contract.md)에 맞춰 소유 세션이 확정한다. provider/work/episode/session 식별자는 표시 제품명·제목에서 만들지 않는다.

WebView2/egui는 소유 STA/UI thread에 있고 현재 Browser는 Rc 기반이다. Send+Sync ApplicationClient에 COM/WebView 객체를 직접 넣거나 unsafe Send로 감싸지 않는다. application은 typed message/ticket 포트를 사용하고 platform actor가 소유 thread에서 실행한다. GUI의 동기 API 호출이 같은 UI thread의 browser 작업을 기다려 교착하지 않도록 비동기 실행·완료 계약을 먼저 정한다. 현재 미구현 jobs.watch 등을 이미 쓸 수 있다고 가정하지 않는다.

CLI는 GUI 버튼·주소창·바로 보기 클릭의 자동화에 의존하지 않는다. 기존 DOM 클릭 메서드를 감싸는 것만으로 공통 resolver가 완료되지 않는다. 동일 application 포트의 실제 provider가 주입된 host 또는 독립 runtime이 필요하며, 준비되지 않은 standalone CLI는 unsupported를 정직하게 유지한다.

## 상세와 인수 범위

- [source provenance·외부 URL·광고 차단 경계](catalog-episode-source-provenance.md)
- [legacy lifecycle·GUI/CLI 인수 조건](legacy-playback-application-port-boundary.md)
- [기존9표본의 source·오류 계약](text-r3-import-error-contract.md): 별도 제안 기능이며 catalog 구현으로 세지 않는다.

새 네트워크 실기는 실행하지 않았다. provider의 실제 빈 목록·검색 파라미터·동적 watch 경로·만료·인증·요청 원시 association·headless 동작은 미검증이다. source-level 조사와 과거 표본 관측을 현재 전체 사이트/CLI 기능 성공으로 승격하지 않는다.
