---
id: sot:kinekino/hls
title: 관측된 HLS 매니페스트의 회차 해석 결과
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 관측된 HLS 매니페스트의 회차 해석 결과

## 현재 단계

최신 [wire 실제 CLI 인수](../../verification/application/player-manifest-wire-cli-2026-10-19.md)는 r3 전체289pass17ignored·fmt/strict Clippy/두 bin과 actual1pass22.18초다. 실제 ExtraInfo Referer absent, schema2 Completed·새 CLI 재조회·손상 거부를 확인했다. 아래 r1은 보존한 역사적 실패다. 전체 media/native 재생은 아직 미검증이다.

`player-manifest-cli-r1`은 [runtime 관측](../../verification/application/player-runtime-cli-2026-10-19.md) 뒤의 checkpoint다. [실제 검사](../../verification/application/player-manifest-cli-2026-10-19.md)는 fmt/수리 strictClippy/전체283pass17ignored/두bin 뒤 actual1failed21.14초다. 실제 POST628bytes와 HLS master 후보678bytes를 확보했지만 기본 event의 관측 빈 Referer가 source 정책에서 거절돼 게시하지 않았다. ExtraInfo의 wire 부재/빈 값/미관측을 구분하는 수리가 미등록 준비 중이다. 첫 Clippy와 actual 실패·원문·수정 전 source-product 및 parser.rs source-repair를 보존했다. 아래 구현 계약의 Completed·새 reader/tamper 인수는 아직 실제 성공하지 않았다.

## 후속 wire 증거 수리

`player-manifest-cli-r2`는 실제 `Network.requestWillBeSentExtraInfo`를 session과 유일 request ID로 연결한다. 기본 event의 빈 Referer를 wire 증거로 사용하지 않는다. 추가 관측 없음은 `None`, 완전한 headers map에서 부재는 `Absent`, 관측 빈 값은 `Empty`, URL은 `Url`로 구별한다. redirect·중복 event의 모호함과 session별64개 상한 초과는 증거 없음으로 남긴다. Cookie/Auth는 투영하지 않는다. 늦게 도착한 ExtraInfo는 현재 capability에만 보강하며 body를 다시 요청하지 않는다.

새 저장 형식은 schema2다. `episodes.inspect_resolution`은 별도 CLI 프로세스에서도 원문과 typed 증거를 재검증하며 URI를 제외한 요약을 반환한다. schema1은 typed 조회에서 Unsupported이고 원문 조회는 보존한다. r3 전체 테스트와 실제 실행 인수를 통과했다. r1의 실제 실패를 소급해 성공으로 바꾸지 않는다.

## 수집과 완료 조건

native 관측 callback의 수신 Unix ms를 event마다 보존한다. 이는 CDP 자체 monotonic timestamp나 서로 다른 시스템 clock의 정확한 교차 배치가 아니다. synthetic event에는 미관측 시각을 null로 남긴다. request의 Pending/Finished/Failed/Redirected, response·finished byte 수, body 조회 실패 capability를 분리한다. provider 상태와 observation_end(manifest_verified/response_rejected/provider_failed/execution_deadline/cancelled)는 별도 값이다.

body는 실제 현재 session/request/frame/loader·native owner에 결합된 일회 capability로만 읽는다. GET player script, 실제 media-cdn `/player/index.php` POST XHR/Fetch 응답, GET HLS MIME 또는 GET XHR/Fetch probe를 허용한다. MIME·확장자와 무관하게 probe 자체는 미디어가 아니다. 새 GET/POST·query/body 재구성·header 변경·직접 복호화는 없다. 본문·metadata·URI·query는 private catalog에 보존한다.

2MiB/응답·한 work 전체4회 발급 상한은 유지하며 in-flight·조회 실패도4회에 포함한다. owner 교체·취소 뒤 callback은 원문 해석 전에 거부한다. required script/bootstrap/명시 manifest body 조회 실패는 해당 현재 수명과 목적을 기록하며 다른 VTT probe 실패와 구별한다.

8 runtime 표본 확보는 네트워크 관측 전체의 완료 조건이 아니다. 기존 job deadline 안에서 script/bootstrap 성공 본문, 현재 child의8표본, 실제 응답의 HLS 검증을 함께 기다린다. 명백한 필수 응답 실패·검증 거절·provider 실패·취소·deadline은 각각 기록한다. deadline을 늘리거나 pending 요청을 빈 응답/실패로 바꾸지 않는다.

## hls-bounded-v1

정본은 `domain/hls`이며 네트워크·UI·player에 의존하지 않는다. [RFC8216](https://www.rfc-editor.org/rfc/rfc8216.html)의 header/태그군/URI 및 기본 어휘를 참고한다. 추가 parser dependency 없이 지원 부분집합을 구현했다. UTF8·2MiB·16384줄·줄8192bytes·URI4096개를 제한하고 exact EXTM3U, BOM 금지, VERSION 중복, master/media 태그 혼합, 대기 URI 누락, 양수 유한 EXTINF·target duration, bandwidth와 기본 attribute 어휘/중복·인용 형태를 검사한다. 매체 tag의 BYTERANGE와 key IV도 경계를 검사한다.

master의 variant 또는 media의 segment 구조가 필요하며 prefix·HTML·파일명·MIME만으로 통과하지 않는다. 선언된 URI만 원 응답 URL을 기준으로 해석해 HTTPS/기본443/credential 없음 정책을 검사한다. 상대 URI의 표준 해석은 새 media URL을 추정해 요청하는 동작이 아니다. 키 URI를 읽거나 복호화하지 않는다.

AES-128 metadata 인식은 재생 지원/권한/키 확보 증거가 아니다. SAMPLE-AES/비identity key format, 날짜·범위 등 미구현 semantics와 알 수 없는 EXT-X 기능은 Unsupported로 구분한다. 이 제품 부분집합을 전체 RFC/LL-HLS 적합성 검사라고 표시하지 않는다. Master 통과도 자식 playlist/segment 응답·decode·재생을 입증하지 않는다. 구체 연구와 source 시점 한계는 `runtime/coordination/hls-validation-research/findings.md`에 보존했다.

검증 객체는 Deserialize할 수 없는 domain 값이다. 새 경로의 MediaSource는 이 객체의 실제 응답 URL과 관측된 실제 Referer로만 만든다. 없는 Referer를 현재 페이지로 보충하거나 legacy prefix 검사를 정본으로 중복 적용하지 않는다.

## typed 저장과 CLI

공통 `EpisodeResolutionResults` port의 `StoredEpisodeResolution`은 request descriptor SHA, episode/player 문서, 검증 profile/summary, 실제 manifest capture와 보존 raw 참조를 결합한다. `CatalogFiles` adapter는 참조마다 body/metadata SHA와 bytes를 재검증하고 실제 GET·purpose·session/frame/loader·응답 URL/status·Referer와 body의 HLS summary를 다시 대조한다. 과거 JSON/DTO를 live response capability로 승격하지 않는다.

공통 job publication/CAS gate 뒤 `episode_resolution` kind의 private JSON artifact를 저장하고 새 reader로 검증한 결과만 Completed로 게시한다. 결과 metadata의 HTTP status는 null이며 실제 manifest HTTP status는 원 browser_response_body metadata에만 남긴다. 기존 metadata의 새 request method/resource type/purpose는 optional로 읽되 새 manifest 결과에는 실제 필드를 요구한다.

CLI는 명시 provider host의 experimental jobs.submit operation을 사용한다. 직접 episodes.resolve 명령은 별도 미지원 상태를 유지한다. jobs.get의 output.receipt.resolution_artifact를 catalog.read_raw로 전체 재읽기할 수 있다. 같은 application 결과 port를 SDK 및 후속 native consumer가 사용한다. capability의 can_complete는 해당 실행 host의 operation 제공 여부에 조건부이며 engine/site 성공을 보장하지 않는다.

Completed의 범위는 manifest_verified=true/profile=hls-bounded-v1이다. referenced_resources_verified/media_verified/media_decoded/native_playback/live_session_capability/url_expiration_verified는 모두 false다. 이는 전체 media resolve·native 재생·전체 제품 goal 완료가 아니다. 실제 native handoff에는 source·세션·전송·수명과 load/metadata/time/control/종료를 추가 인수해야 한다.

강화 actual gate는 같은 회차·기존30초 deadline에서 Completed 및 manifest 결과를 요구한다. raw 전부 CLI 재읽기와 typed 새 reader 재개방, private 복사본 manifest 한 byte 변이 거부 및 원본 재읽기도 요구한다. 실패할 경우 원문·timeout·원래 assertion을 보존하며 성공으로 재분류하지 않는다.
