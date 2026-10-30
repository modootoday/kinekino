---
id: sot:kinekino/legacy
title: 배포·제품 식별·신뢰·legacy 이관 경계
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 배포·제품 식별·신뢰·legacy 이관 경계

2026-10-26 통합 설계. PM은 공개/배포 gate와 index, SDK는 저장/프로세스/IPC, Media는 모델/실표본 provenance, Meari는 독립 framework/assets/native closure, UI는 제품 SVG/표시·입력에 대한 책임을 유지한다. 이 문서는 배포/공개/push/권리 승인 실행을 허가하지 않는다.

## 신뢰 경계

| 입력 | 허용 처리 | 거절·보존 기준 |
|---|---|---|
| browser URL/message/provider DOM | Rust validation과 현재 source/session에 묶인 typed observation | shell/임의 exe·file/javascript/external protocol·credential/서명URL 영속·다른source stale message 거절 |
| local user path | 목적별 root/file guard·명시지원 local disk·canonical content identity·한도 | traversal/reparse/root 교체·원본/출력동일·기존출력 자동덮기 거절. API별 path 허용범위의 차이는 원 handler 유지 |
| IPC/CLI Request | 공통 version/context/unknown field/schema·payload 한도·현재사용자 endpoint | endpoint 문자열만으로 권한 생성 금지, 전달후 unknown 보존·자동재시도/fallback0 |
| project/proposal/model output | 데이터만·strict schema·revision/hash/원 source refs | 실행스크립트/filtergraph·임의명령·모델 README/Python remote code 실행 금지 |
| model/native tool | explicit trusted assembly의 revision/hash/license/runtime/admission | main/latest·미검증pickle·caller 선택프로그램·선언dtype를실측으로승격 금지 |
| UI/plugin/HMR | manifest/grant·typed binding/document epoch | unknown/extra props 묵살·권한token 합성·stale mutation replay·Web에native권한추정 금지 |
| archive/cache/history | 재생성캐시와 소스/모델/미디어/원실패/복구증거 분리 | 캐시명만으로삭제·과거FileID/원절대경로일괄치환·새호스트성공승계 금지 |

민감 payload/토큰/쿠키/개인키/비밀번호/서명URL을 command line·diagnostic·프로젝트·공개자료로 내보내지 않는다. request_id는 공개 correlation이지 secret carrier가 아니다. 사용자가선택한미디어의 재생가능성이 저장/변환/재배포권한을 보장하지 않는다. 실제로 제공된 출처·라이선스·attribution 범위와 불명확한권리를 구분하고 Unknown을 허가로 바꾸지 않는다.

## 제품 식별과 설정 migration

ProductIdentity/PRODUCT가 display_name·stable_product_id·GUI/CLI 파일명·config schema를 소유한다. package/crate 이름과 표시명을 분리하고, 새 코드에서 문자열을복제하지않는다. installer/publisher/support metadata는 같은제품설정의추가필드로관리한다. 명칭변경만으로 API namespace/JobId/ProjectId/pipe identity를 바꾸지 않는다.

stable OS ID/저장위치를 변경해야할 때는 old schema/location 탐지→정확백업→strictdecode/경로·권한검증→versioned 변환→새namespace에no-overwrite publication→readback/상태보고 순서다. 실패시 원본/백업을보존하고 새위치완료로표시하지않는다. 완료marker는 같은 migration input/schema/hash에서만 재사용하고 반복실행은 새revision을중복생성하지않는다. 수동선택파일·원미디어·모델원본은이름변경에맞춰자동이동하지않는다. unknown/newer schema는 명시 Unsupported/NeedsInput이며대화없이reset하지않는다. release 전 GUI title/About/CLI help/version·실제파일명·installer/upgrade·동시실행/endpoint·설정이력을검증한다.

## package·권리 closure

공개범위는 root .gitignore의deny-default와책임별허용을유지한다. .spec/.agents/runtime/model/media/cache/운영자료는private다. Cargo include는별도배포경계이므로gitignore만으로안전하다고판정하지않는다. source package inventory와실제배포bundle을각각검사한다. Meari현재manifest의publish=false는publish승인이아니며include패턴이있어도모든내용/권리가검증된것은아니다.

closure manifest는 source HEAD+dirty selected hash/lock/features/target/toolchain·EXE/DLL/runtime·모델/토크나이저/전처리·공식icon/font·template/SVG·license texts/NOTICE/출처·원producer를연결한다. library+target-feature의transitive/license·native runtime/VC/ORT/DirectML 등은별도실제bundle범위로확인한다. font TTF parse/hash만으로glyph/shaping/fallback·Web/native 렌더완료가아니다. 공식upstream/공식Rust·React배포우선Meari카탈로그를사용하고필수font5파일등은대체검증전삭제하지않는다.

source375폰트closure와Media실제9·모델·Sintel/Sprite의증거는서로다른범위다. 개발자선택clip을실제모델highlight결과로꾸미지않는다. 플랫폼preset은조회일/버전있는제작제안이며현재외부서비스규격이나업로드성공을보장하지않는다. 배포전외부규격/권리검토가필요한항목은그때공식근거를갱신하며이번로컬설계에서새최신성주장을만들지않는다.

## 기존 비Rust와플랫폼직접호출 이관

| 현재자산/경로 | 유지할책임 / Rust이관경계 | 삭제·교체조건 |
|---|---|---|
| assets/clean-page.js | 고정cosmetic DOM adapter, Rust network/navigation차단정책과분리 | 기존광고slot/iframe/video보존·popup/redirect no-bypass를같은입력에서검증한대체후. remote scriptlet확장0 |
| assets/media-control.js | browser media의최소명시typed command adapter | play/pause/track/seek 의미·stale/generation·실엔진관측을Rust usecase/host와동등검증후 |
| assets/playback-observer.js | browser의실제미디어관측을Rust port로전달 | 추정progress금지·bounded message/schema·원source/session·정리·관측오류동등검증후 |
| assets/episode-observer.js | 실제href/canonical회차관측 | 제목URL합성금지·회차ID/stale/deeplink·provider동등검증후 |
| src/infrastructure/browser/mod.rs 초기화주입 | 위4고정자산의명시등록/OS기술adapter | presentation에정책/OS호출을늘리지않고application port로조립. DOM에필수인최소JS는명시adapter로한정 |
| 기존PowerShell/Python운영recipe | 역사절차/일회작업근거, 새실행도구의언어예외아님 | 기존동작·원복/custody검증된Rust도구채택후별도정리. 원실행recipe/실패증거보존 |

React/TSX UI선언및Meari변환입출력예외를JS/TS정책·SDK·검증도구로확대하지않는다. WebViewDOM자체를Rust로직접조작할수있다고가정해필수adapter를지우지않는다. VM/DPI/사용자앱/네트워크설정은이관범위밖이다.

## 인수 gate

RE01 package/bundle 명시inventory·private/secret부재·필수자산존재. RE02 모델/코드/native/font/media별revision/hash/license/NOTICE closure·Unknown분리. RE03 install/config schema·migration/retry/rollback/reopen·원본보존. RE04 stableidentity·GUI/CLI표시·파일명·pipe/동시실행호환. RE05 path/URL/message/plugin/IPC negative boundary·로그redaction. RE06 legacy대체의동일입력/오류/취소/정리동등성과삭제조건. RE07 offline/bare-source/package build·실output검증·호스트도구체인재현성한계. RE08 기능·OS·시각·사용자품질·배포권한의독립판정. 모든gate는계획이며제품새실행/권리승인을의미하지않는다.

연결: [아키텍처](sot-002-product-architecture.md), [identity](../domain/product/identity.md), [adblock](../infrastructure/adblock/sot-003-adblocking.md), [실행 순서](../../../plans/player/design-completion/implementation-order.md).
