---
id: sot:kinekino/provider
title: 화면 없는 브라우저 provider 공통 포트
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 화면 없는 브라우저 provider 공통 포트

상태: 공통 포트/순수 상태·infra 첫 DOM·jobs/CLI 접합 소스를 작성하고 all-target check와 strictClippy를 통과했다. 최종 회귀와 실제 CLI 수집/종료는 진행 중이다. 이 문서는 브라우저 성공/CLI 지원 완료 근거가 아니다. [infra 재사용 조사](../infrastructure/browser-provider.md), [CLI 이관 계획](../../../plans/player/presentation/cli-migration-gaps.md)을 따른다.

## 책임과 실행

application/browser_provider가 검증된 요청·ticket·snapshot·cancel 및 순수 상태 규칙을 소유한다. infrastructure/browser_provider는 전용 STA의 WebView2/숨김 native 부모/message pump와 bounded channel adapter만 소유한다. root CLI/jobs host와 SDK 조립은 SDK 담당, 새 infra 구현과 기존 browser 최소 공통 추출은 UI 담당이다. presentation은 호출하지 않는다. COM/Rc/창/stream 객체를 Send+Sync나 unsafe Send로 넘기지 않는다.

BrowserProviderPort는 try_start(validated_request), poll(ticket), cancel(ticket), navigate(ticket,expected_document_generation,url), release(ticket)를 노출한다. 네트워크·engine 준비 완료를 기다리지 않는 짧은 admission/snapshot 작업이다. 장기 실행과 cancel/deadline·결과 보존은 기존 jobs executor 측에서 연결한다. 기존 동기 CatalogFetchPort에 브라우저 대기를 그대로 넣지 않고, catalog 완료 결과를 job 접수 receipt로 묵시 변경하지 않는다. 명시 --serve-provider의 jobs.submit에서 catalog.search/episodes.list를 실행하고 같은 카탈로그 명령의 result_artifact_sha256 입력으로 저장 결과를 읽는다. episodes.resolve/native 실행은 아직 이 executor에 등록하지 않았다.

## identity와 순서

ticket의 session ID/request ID/request generation은 runtime 작업 identity다. 실제 navigation document generation과 분리한다. 각 callback은 ticket과 발생 시점 document generation을 고정한다. 같은 ticket/current document generation만 상태·원문·미디어 결과를 갱신한다. 새로운 작업/문서/취소·종료 뒤 옛 callback은 무효이며 성공으로 채택하지 않는다. ResolveEpisode의 중간 DocumentReady는 Running/document을 보존하고 SDK가 Rust 추출 후 명시 navigate를 요청할 수 있다. ReadCatalog의 Succeeded는 캡처 완료이며 Rust parser/제품 작업 성공을 의미하지 않는다.

취소 접수는 Cancelling이며 worker 완료 Cancelled와 다르다. terminal capture가 먼저 수락되면 보존하고 already_terminal을 반환한다. 취소가 먼저 수락되면 늦은 capture는 채택하지 않는다. terminal 실패는 명시 오류이고 빈 목록이 아니다. timeout은 terminal Timeout이며 callback을 무효화한다. 소유 host 종료는 새 접수 차단→활성 작업 취소→worker 회수 관측 순서이며 Drop에서 dispatcher를 무기한 기다리게 하지 않는다. 외부 앱·브라우저·사용자 프로필은 종료하지 않는다.

## 캡처와 예산

원문은 complete network response와 DOM serialization을 구별한다. network capture만 수신 HTTP status/headers를 주장하며 DOM에 HTTP 원문 status를 만들어 넣지 않는다. bytes는 owned bounded buffer로 전달하고 실제 SHA/UTF8·HTML parse·파일 게시는 callback 밖의 SDK executor가 처리한다. 원문·쿠키·서명 URL·오류 Debug를 진단에 출력하지 않는다.

초기 범위는 동시 활성1, 명령 queue16, 보존 ticket snapshot32, navigation 관측32, terminal 원문2MiB다. 제한 초과는 resource_exhausted 또는 명시 transport failure로 종료하며 terminal event를 조용히 버리지 않는다. adblock 요청 설정과 filter 실제 설치/차단 관측은 구별하며 기존 공통 엔진/요청 kind·method·referer 의미를 재사용한다. 현재 DOM 우선이며 추가 media observer 소스는 미등록·미검증이다.

job source blob은 정규화된 operation/parameters/source_kind=request_descriptor JSON이다. 이 SHA는 요청을 고정하며 서버 본문을 고정하지 않는다. browser jobs에는 expected_source_sha256를 허용하지 않는다. DOM은 capture_kind=dom_snapshot/status=null이며 engine/ticket/document 세대·navigation/filter 관측과 exact DOM UTF8 SHA를 보존한다. 최종 results/<sha>.json의 SHA/bytes/source descriptor identity와 참조 raw body/metadata를 읽을 때마다 확인한다. 취소 대 최종 결과 게시 gate는 기존 ExecutionControl을 재사용한다. 중간 캡처 artifact가 남을 수 있으나 이것은 Completed output receipt가 아니다.

## 인수 순서

network 없는 상태·queue full·stale generation·cancel 우선순위·종료 회귀를 먼저 수행한다. 이후 자기 소유 CLI host로 카테고리 단일 수집과 EOF 종료부터 실제 관측한다. provider engine 접근, 원문 확보, Rust parser 성공, CLI 제품 지원은 각각 구별한다. 사용자의 기존 브라우저 접근 성공과 직접 HTTP connect 실패만으로 봇 탐지/TLS/DNS/proxy 원인을 확정하지 않는다. GUI 자동입력·새 모델 추론·인증서 검증 해제는 이 경로에 없다.

현재 [v3 실제3category](../../verification/application/catalog-v3-2026-10-19.md)는 방영중58/종영24/극장판24·CLI fresh readback·원문 전체 대조·정상 EOF/재획득을 통과했다. 전체 pagination·회차·resolve/native 재생의 증거가 아니다. 새 회차 Rust fixture는 불변 카테고리 결과를 검증한 후 첫 series를 선택하고 실제 CLI episodes.list와 실패 원문을 보존한다. 제품 v3 고정본을 사용하며 아직 실행 전이다.

미디어 관측은 [소유 문서 수명 ADR](../../../adrs/player/browser-media-document-lifetime.md)과 [응답 귀속 감사](../infrastructure/browser-provider-media-observation.md)를 따른다. 선택된 문서별 WebView 수명은 구현/실제 호환성 검증 전이며 request URL-only/response 시점 current-page 매핑을 지원 근거로 사용하지 않는다. 실제 회차 DOM 확보를 먼저 수행하고 redirect/cookie/POST/iframe/worker 호환성 관측에 따라 선택을 다시 검토한다.
