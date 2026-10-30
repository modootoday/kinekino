---
id: sot:kinekino/catalog-http-chain
title: Catalog HTTP 실패의 오류 chain 정적 검토
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Catalog HTTP 실패의 오류 chain 정적 검토

2026-10-19 PM 배정의 읽기 전용 검토다. 제품 source 수정·cargo·새 네트워크·환경변수 값 조회·인증서/시스템 설정 변경은 없다. 기존 catalog-r2/r3/r4 observation과 현 media-catalog source 및 로컬 dependency source를 읽었다. **실제 DNS/TLS/proxy 원인은 여전히 미확정**이다. R4 엔진 보존 완료·AI 실험 유보 상태도 유지한다.

## 관측과 정보 손실

기존 `.spec/sots/verification/application/catalog-http-2026-10-19.md`와 r4 observation은 connect=true, timeout=false, io_kind=Other, os_code=null, HTTP status/body/provenance 부재다. reqwest의 is_connect는 hyper legacy Error의 연결 단계 플래그를 source chain에서 찾는다. DNS·TCP·프록시 CONNECT·TLS 중 어느 하나를 뜻하지 않는다. Other/null도 해당 OS 오류가 없었다는 확증이 아니다.

현재 `packages/media-catalog/src/infrastructure/http.rs::transport_error`에는 다음 손실이 있다.

- source()만 최대8회 순회하고 잘렸는지 기록하지 않는다. std io::Error의 custom payload는 source()가 payload 자체 대신 **payload.source()**를 반환한다. 따라서 `io.get_ref()`로만 보이는 payload 타입을 빠뜨릴 수 있다. 로컬 core/src/io/error.rs:547 구현에서 확인했다.
- io를 만날 때마다 같은 io_kind/os_code를 덮어쓴다. 더 앞의 실제 OS code가 뒤 custom wrapper의 None으로 소실될 수 있고, 여러 io wrapper의 순서·개수도 남지 않는다. 현재 Other가 어느 층인지 복원할 수 없다.
- source Display를 이어 붙여 certificate/unknownissuer/dns/resolve/proxy substring만 분류한다. Display가 typed variant를 완전히 표현하지 않으며, 인증서 이외 TLS 오류는 일반 connect로 남을 수 있다. 반대로 메시지 안 URL·이름의 우연한 단어는 분류 근거가 될 수 없다.
- 로컬 hyper-util의 TunnelError Display에는 `tunnel error: unsuccessful`, `unexpected end of file`, `io error establishing tunnel`처럼 proxy라는 단어가 없는 형태가 있다. 현재 proxy substring 분류가 이를 놓친다. 그러나 이 사실만으로 이번 실패를 proxy라고 판정하지 않는다.
- Client::build와 send 오류를 같은 함수로 바꿔 생성/요청 단계가 구별되지 않는다. 원래 오류를 CatalogError로 바꾸고 나면 새 진단에 필요한 typed chain을 다시 얻을 수 없다.

`io.get_ref()` 누락은 타입 접근의 실제 결함 후보지만, 현재 Display도 수집하므로 모든 rustls 오류 메시지가 사라진다고 주장하지 않는다. 보존된 r4에는 원문 chain이 없어 이번 실패에 어느 손실이 작용했는지는 알 수 없다.

## 실제 고정 dependency 경계

현재 독립 Cargo.lock은 reqwest0.12.28/hyper1.12.0/hyper-util0.1.21/hyper-rustls0.27.10/rustls0.23.46/tokio-rustls0.26.6이다. manifest는 default-features=false,blocking+rustls-tls다. 로컬 reqwest feature 정의상 rustls-tls는 webpki-roots 계열이며 Windows native roots와 같은 설정이 아니다. 이것은 source 설정 사실이고 이번 TLS 신뢰 실패의 증거가 아니다.

reqwest Client 기본 auto_sys_proxy=true는 ProxyMatcher::system을 추가한다. 로컬 hyper-util matcher는 환경변수 ALL/HTTP/HTTPS/NO_PROXY의 대소문자 우선순위를 처리하고 Windows 시스템 proxy fallback은 별도 client-proxy-system feature 조건이다. **reqwest default-features=false라도 환경 proxy가 자동으로 사라지는 것은 아니다.** lock만으로 활성 feature closure나 이번 목적지에 실제 선택된 proxy를 확정할 수 없으며, 현재 환경값이나 proxy 주소·자격증명은 조회하지 않았다.

hyper-rustls connector.rs:120은 TLS io error를 io::Error::other로 감싸며, tokio-rustls client.rs:89도 ClientConnection 생성 오류를 Other에 담는다. handshake processing은 InvalidData를 쓸 수도 있다. 따라서 Other만으로 TLS/설정/터널/일반 IO를 고를 수 없다. hyper ConnectError와 TunnelError는 각각 private 모듈의 타입이며 외부에서 직접 downcast할 수 있다고 가정하면 안 된다(Tunnel만 re-export). rustls::Error는 공개 enum이나 직접 동일 버전 의존이 필요한 선택이다.

## 소유 세션에 제안하는 최소 Rust 진단

1. **추가 요청 없이 기존 실패 변환 직전에** phase(client_build/request_send/body_read), reqwest flags(builder/request/connect/timeout/body/decode/redirect), bounded error node 수·truncated·unknown 여부를 고정 enum/숫자로 기록한다. HTTP body를 못 받았으면 계속 raw/provenance=null을 유지한다.
2. chain을 bounded graph로 읽되 각 io::Error의 get_ref() payload와 source() 양쪽을 방문하고 중복·cycle/한도를 처리한다. 노드별로 io kind·raw_os_error를 보존하며 기존 단일 code를 유지해야 한다면 첫 non-None을 뒤 None으로 지우지 않는다. 순회 실패를 임의 원인으로 채우지 않는다.
3. 가장 작은 1차 수정은 std io 구조와 reqwest flags까지다. DNS/TCP는 고정 dependency의 `dns error`/`tcp connect error` 같은 **정확한 allowlisted connector label**을 hint로 기록하고, private 타입을 downcast하려고 dependency를 무작정 추가하지 않는다. Tunnel Display의 알려진 정적 prefix/label은 proxy_tunnel_hint로 분리하고 상태/자격증명/endpoint 원문은 저장하지 않는다. Hint는 실제 단계 계측과 구별한다.
4. typed TLS 구분이 필요하면 infra/진단 한정으로 동일 rustls 버전을 직접 연결해 get_ref/source의 rustls::Error를 downcast하고 InvalidCertificate의 안전한 variant, alert 종류, protocol/config/other 같은 **고정 enum**만 추출한다. Debug/Display 전체·certificate DER·서버 이름·Other 내부 문자열은 출력하지 않는다. 같은 버전 타입 일치와 feature 영향을 담당자가 검토해야 하며 이번에는 구현/컴파일하지 않았다.
5. client 설정의 정적 profile(roots backend, proxy auto/explicit/disabled, timeout)과 proxy env **설정 존재 여부만** 기록할 수 있다. env 존재는 proxy 적용 증거가 아니다. 값·호스트·사용자명·NO_PROXY 목록·RUST_LOG hyper trace/connection_verbose를 원문으로 출력하지 않는다. 불명확하면 unknown을 남긴다.

먼저 네트워크 없는 Rust synthetic 오류 graph로 nested io Other→rustls leaf, 앞 OS code→뒤 None, 8단계 초과·cycle/중복, tunnel의 proxy 단어 없는 label, 알 수 없는 leaf, sentinel credential/URL 미노출을 검증한다. 기존 fmt/test/strict Clippy는 변경 담당의 필요한 범위에서 수행한다. 새 실사이트 요청은 이번 조사에 없으며, 실제 원인 확인이 필요하면 중앙 조율 후 **단일 요청의 진단**으로 진행하고 세 카테고리를 다시 반복하지 않는다.

DNS 성공/실패를 확정해야 하는 후속 단계에서는 reqwest resolver wrapper로 실제 요청에 사용된 resolve 결과의 성공·주소 수·안전한 오류 종류만 관측하는 선택이 있다. 별도 DNS probe나 직접연결/no_proxy A/B는 현재 요청 경로를 바꾸고 추가 네트워크이므로 이번 최소 수정에 포함하지 않는다. proxy를 쓰면 resolve 대상도 origin이 아닐 수 있다. connector/handshake 단계가 public hook에서 식별되지 않으면 추정 대신 unknown을 유지한다.

TLS roots 교체·인증서 검증 해제·시스템 proxy/인증서/방화벽 변경은 **현재 근거로 제안하지 않는다**. 이 문서는 오류 정보 보존과 다음 안전한 진단 설계이며 사이트 장애 원인 규명·HTTP 성공·제품 CLI 지원 완료가 아니다.

## 사용자 브라우저 성공 관측과 CLI browser provider

사용자는 기존 실제 브라우저에서 source 접근이 정상이며 직접 GET의 봇 탐지 가능성을 지적했다. 과거 정본 `.spec/sots/player/infrastructure/browser/sot-001-browser-research.md`도 2026-10-18 내장 브라우저의 세 목록·작품·회차·외부 iframe 관측과 최초 방영중 Chrome 확인을 기록한다. HAR/응답 헤더/원문은 당시 수집하지 않았다. `.spec/sots/verification/playback/sot-005-playback-validation.md`에는 별도로 앱 WebView2 155.0.4283.60의 광고차단 ON 세 표본 재생 근거가 있다. 이는 제한된 과거 표본이며 현재 독립 CLI provider 전체 지원 증거가 아니다.

브라우저와 reqwest는 TLS 구현/ClientHello·ALPN/HTTP 협상, proxy 선택·신뢰 저장소, cookie/session/storage, JS 실행·하위 리소스·요청 순서가 다를 수 있다. 이번 요청의 실제 차이를 계측한 것은 아니다. connect 실패는 HTTP status/body가 없어 HTTP403/429·challenge 응답이라고 판정할 수 없고, 서버/중간 경로의 연결 종료와 자동화 탐지 가설도 배제하거나 확정할 수 없다. UA 변경만으로 해결됐다고 하거나 browser 성공을 DNS/TLS/proxy 실패의 반증으로 삼지 않는다.

| 경로 | 적용 범위 | 현재 한계·다음 최소 인수 |
|---|---|---|
| Rust reqwest provider | bounded HTTP raw 수집·명시 redirect 검증·Rust HTML 파싱 | 오류 정보 보존을 먼저 보완. JS/session이 필요한 페이지와 동일 기능이라고 가정하지 않음 |
| WebView2/Chromium infrastructure provider | 기존 document/iframe/요청·응답·회차 관측과 필요한 페이지 실행을 명시 포트로 제공 | GUI presentation 없이 소유 STA/message loop·종료·정책·typed 결과를 분리하고 실제 CLI로 인수. 현재 CLI 구현 완료나 완전 headless 지원으로 표시하지 않음 |

CLI 우선은 HTTP-only 제약이 아니다. 공통 application은 catalog/episode/resolve 요청과 job/ticket·deadline/cancel·source revision을 소유한다. infrastructure의 browser actor가 WebView2/COM을 소유 STA에서 실행하고 typed 메시지를 반환한다. CLI는 공통 SDK로 호출하며 앱 GUI의 버튼·주소창·스크린샷·사용자 입력 자동화에 의존하지 않는다. 별도 숨김 host/message window가 기술적으로 필요할 수 있지만 그것은 infrastructure 수명 책임이고 제품 GUI 인수 선행 조건이 아니다.

현 Browser::new는 parent window handle과 build_as_child를 요구하고 Rc/Cell/RefCell·presentation 크기·visible 설정을 포함한다. about:blank의 초기 hidden 옵션만으로 독립 CLI runtime이 이미 완성됐다고 해석하지 않는다. 같은 actor thread의 동기 호출이 그 thread의 callback을 기다리지 않도록 메시지/완료 경계를 설계한다. ApplicationClient에 COM/Rc를 넣거나 unsafe Send로 감싸지 않는다. CLI가 닫힐 때 소유 view/actor/child와 outstanding 요청을 정리하고, timeout/cancel 후 늦은 callback·다른 generation은 결과로 채택하지 않는다.

기존 open_episode_player의 watch-now DOM action은 raw GUI 클릭 자동화와 다르지만, 현재 메서드를 단순 노출한 것만으로 resolver 계약이 완성되지 않는다. 원래 episode/ticket·popup 목적지·frame/navigation generation·응답/manifest·완료 조건을 결합한 provider 작업으로 분리해야 한다. 기존 주입 JS는 이관/최소화 대상으로 유지하고 새 application 정책은 Rust에 둔다. Loaded 이벤트는 목록/resolve 완료가 아니다. 검색 selector·빈 목록·challenge/인증 요구는 각각 명시 결과/needs_input·미지원으로 구분하며 challenge를 조용히 우회하지 않는다.

browser provider는 기존 광고차단 request-source-kinds ALL·cosmetic/observer 정책과 예외/활성 revision을 명시적으로 적용하고 실패 시 조용히 해제하지 않는다. cookie/storage는 provider 전용 session/profile 수명으로 관리하고 사용자 Chrome profile·cookie·자격증명을 임의 복사하지 않는다. signed media URL/headers는 안전한 source lease로 전달하며 로그/프로젝트에 노출하지 않는다. mpv/다운로드의 후속 요청은 WebView filter를 자동 상속하지 않으므로 별도 media-fetch 정책이 필요하다.

담당 세션은 먼저 기존 관측 fixture와 synthetic 메시지로 allowlist·빈 목록/미관측·generation·정책·cancel/timeout·정리를 검증한다. 그 뒤 중앙 조율된 **CLI provider 요청 하나**의 시작→결과/실패→종료·원문/안전한 provenance를 인수한다. 외부 프로그램 종료·수동 GUI·브라우저 재접속·카테고리 반복 요청은 이번 검토에 없다. HTTP 진단과 browser provider는 같은 포트의 명시 capability/backend로 비교하며, 실패 시 다른 engine으로 조용히 fallback하지 않는다.

로컬 source 기준: reqwest src/error.rs:116/147,connect.rs:840/940,async_impl/client.rs:419/694,Cargo.toml:148; hyper-util src/client/legacy/connect/http.rs:665/718,connect/proxy/tunnel.rs:31/235/251,client/proxy/matcher.rs:228/238; hyper-rustls src/connector.rs:115; tokio-rustls src/client.rs:80와common/mod.rs:115; 설치 Rust core/src/io/error.rs:359/547. 경로·버전은 위 고정 registry 기준이며 runtime 동일성 전체를 별도 인증한 것은 아니다.
