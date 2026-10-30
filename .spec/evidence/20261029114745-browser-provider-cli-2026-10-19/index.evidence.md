---
id: evidence:kinekino/20261029114745
title: 화면 없는 browser provider·jobs·CLI 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 화면 없는 browser provider·jobs·CLI 검증

## 범위

공통 BrowserProviderPort/순수 BrowserWork, 독립 STA/숨김 부모/WebView2, 기존 adblock, DOM의 bounded serializer, 기존 jobs executor/게시 gate, 불변 원문/결과 파일과 CLI 재오픈을 연결한다. 첫 범위는 catalog.search(category-only)/episodes.list의 experimental browser jobs다. episodes.resolve/media/native·새 추론·수동 GUI 검토는 이 검증에 포함하지 않는다. [공통 계약](../../player/application/browser-provider.md), [카탈로그](../../player/application/api/catalog.md).

## r1

runtime/quality-review/browser-provider-cli-r1: all-target check0(28.27초), fmt0, strictClippy0(9.85초), workspace test **213 passed/10 ignored**(컴파일2분44초), 두 bin build0(0.95초). 실제 사이트 요청은 없다. 추가6 순수 회귀는 요청/세대/취소/DOM·network 구분/단계 이동/timeout, infra5는 queue/poison/끊긴 worker/바이트·잘림 경계다. synthetic browser 1개는 실제 jobs/files/named-pipe/CLI 결과 재오픈과 source identity·손상 거부를 검증했다. synthetic-no-engine을 실제 WebView2 결과로 승격하지 않는다.

## r2와 실제 관측

r2는 script dialog 비활성화·DownloadStarting 취소 guard 및 archive-only CLI 조립을 추가했다. 코드·원문·결과의 정본은 runtime/quality-review/browser-provider-cli-r2/source-at-audit, lock SHA는 0cf44760431b804cdf18afd50aa82f57b50579cfe1465f9f6d6e9ee7b1aa96cc다. 미등록 media_observer.rs는 복사된 준비 소스이며 컴파일/실행 대상이 아니다. fmt0/strictClippy0(10.71초), workspace **213pass/10ignored**(컴파일1분51초), 두 bin build0(0.80초)를 통과했다.

고정 CLI SHA **aa7166c3c069a6f0727ebb52b78afcc4a407bf4b14869b381f96ef16becb8221**, GUI SHA **3865c5e29aca4e925817a10c9529bc311ea8f5437b2566029e5d165e9d79db9e**. source/copy 전후 hash는 Rust fixture가 대조했다. GUI 제품 실행·수동 클릭은 없다. explicit ignored 단일 관측은1pass/20.24초·exit0이다. 이것은 아래 정직한 실패와 정상 종료의 관측 통과이며 카탈로그 기능 통과가 아니다.

WebView2 **155.0.4283.62**가 방영중?page=1의 DOM **101,495B**를 수집했다. SHA **56b3430f96481e8d6106ed8a39ec6f2693f1ad3f8c7bc848ef5523c9d670e109**, metadata ID **fae1aed877f01dc026148158593a8974231a412e213b7319b3bccc1623f568ca**를 보존했다. capture_kind=dom_snapshot/status=null, request/document generation1이다. filter 설치/요청 설정=true, 관측 requested71/blocked0이며 실제 차단 성공을 설치 여부로 대신하지 않는다.

job-81041a4edffe4678b742f0bcc8dbb1a5는 attempt1/revision3/**NeedsInput**이다. `unrecognized_page: Catalog layout not recognized; empty result unproven`, output=null/publication_started=false이며 빈 목록으로 바꾸지 않았다. EOF의 provider_host_stopped는 browser_worker_exited=true, completed/cancelled/interrupted totals0이다. actual-cli/provider-actual 아래 observation.json·requests/stdout/stderr/exit·profile·원문/metadata를 보존했다. 실제 engine 접근·DOM 보존·소유 worker 정상 종료와 parser 실패를 구별한다. parser는 보존 DOM의 오프라인 실증 후 보완하며 추가 네트워크 재시도는 아직 없다.

## parser/rawreader 고정 CLI와 실제 3category

runtime/quality-review/catalog-parser-raw-r1의 제품 snapshot은 fmt0, all-target check0(26.05초), strictClippy0(23.55초), workspace 218pass/12ignored 및 두 bin build0(2.79초)다. 고정 CLI SHA는 `00cf42bd85154d3d3e4081f9d3ffcc311f2d619326cf03457c542b5a3a5e4dea`, GUI SHA는 `56d8ddfcf0d9d60e9aa60f462bde0d36b564e4f724cacf72371b636326d32809`이며 GUI는 실행하지 않았다. source-product/final-bin 보존 뒤 제품 코드 변경 없이 fixture만 강화해 fmt/strictClippy 및 jobs 5pass/3ignored를 확인했다.

이미 빌드된 harness를 직접 실행해 새 컴파일 없이 offline 1pass/7.06초와 실제3category 1failed/86.23초(exit101)를 얻었다. offline은 보존 r2 DOM의 v2 해석 58개와 CLI raw chunk 전체 복원·v1 capture 불변을 확인했다. 실제3category는 기능 실패이며 반복 네트워크 재시도하지 않았다.

| category | 실제 결과 | 보존 원문 |
| --- | --- | --- |
| airing | timeout, output=null, publication_started=false | DOM capture 없음. 시간 초과 원인은 아직 미확정 |
| completed | NeedsInput/unrecognized_page | 221453B, body SHA `af85cbaaeae7cc28baa2d985481fe9ce46e5e117439464a6a892194a348059a2`, metadata `646d64e22463c02d789ac4109b359321cba91c50a2014f312acef1bdf661088b` |
| movies | NeedsInput/unrecognized_page | 205480B, body SHA `2badebea2baeca62aaf8e03412eb50eb51e07b4a8431b1c9ba2a80f7c5801f23`, metadata `4c1900b11d5989033d93212da513dbcc90de8ee908e34f5a1322d0018b9fba97` |

두 DOM은 status=null, WebView2 155.0.4283.62, filter requested/installed=true이며 requested_count=37/35, blocked_count=0이다. 실제 차단 양수 증거는 없다. 보존 DOM에는 .grid-wrap>.grid>.row와 .desc가 있으나 현재 v2가 요구한 section.board-list 조상이 없다. 이는 두 parser 거부의 구조적 원인이며 방영중 timeout의 원인으로 확대하지 않는다.

actual-cli/provider-three-categories 아래 원문·metadata·각 observation.json·CLI 입출력 및 logs를 보존했다. 실패 수집 뒤 원 host의 정상 EOF 종료, 이전 endpoint 접근 실패, 동일 jobs/catalog/profile roots를 새 endpoint의 소유 host로 재획득하고 정상 EOF 종료하는 assertion을 통과했다. 양쪽 stop receipt의 browser_worker_exited=true다. 최종 all-complete assertion에서 실패했으며 parser 성공이나 전체 OS 자식 종료를 뜻하지 않는다. 후속은 보존 DOM의 범주별 구조 회귀와 timeout 시 bounded snapshot 진단 보존이며 네트워크 반복보다 먼저 수행한다.

## 판정 한계

ReadCatalog의 내부 Succeeded는 DOM 캡처이며 parser/게시/파일 검증을 통과한 jobs.Completed와 다르다. DOM은 HTTP status=null이고 exact network body라고 표시하지 않는다. 중간 raw artifact가 남을 수 있으나 최종 output receipt와 구별한다. 저장된 Completed가 현재 파일 가용성을 보장하지 않으며 readback은 참조 raw/metadata/result SHA를 다시 검증한다.

정상 EOF의 worker 종료와 engine 생성 중 강제 취소/종료 고장은 별개다. Wry 생성 자체의 중간 강제 중단·COM 영구 대기·실제 script dialog/download 실패 주입과 전체 WebView2 자식 프로세스 종료 관측은 미검증이다. root의 소유 worker 종료 receipt를 외부 사용자 브라우저 종료나 모든 OS 자식 종료의 증거로 확대하지 않는다.
