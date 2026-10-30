---
id: plan:kinekino/20261029113641
title: 실제 ffprobe와 원 provider 연결 체크포인트
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 ffprobe와 원 provider 연결 체크포인트

2026-10-23. root JOB 채택 d7c8ea26166ace75342be85b213329d687ef1155 이후의 후속이다. 실행 권한은 execution-leases.md가 유일하다. 이 문서는 실행·배포 승인이나 지원 표시가 아니다. 아래 초기 입장/제안의 미실행 표기는 작성 당시 이력이며 현재 실제 결과는 [원 fd 부분 검증 정본](../../../sots/verification/production/download-original-fd-checkpoint-2026-10-23.md)을 따른다.

## 실제 provider host 설계 전 추가 확인

최신 취소 수리는 private effective8 `4B6E2EED3AD68C14CD66529D13890C9BDDCD747221B289FB0AFF72F264E41842`이며 작성·독립 읽기·leaf fmt/check만 완료했다. 실제 sourcecancel33의 실패를 해결했다고 아직 표시하지 않는다. HTTP host 구현과 생산 등록은 후속 별도 배정이다.

읽기 전용으로 다음 경계를 추가 확인했다. private `media-download-provider-r1/src/infrastructure/download/http_stream.rs`185~202의 response.chunk는 frame이64KiB보다 크면 거절하고 그 이하만 pending에 복사한다. 이는 caller에64KiB씩 반환하는 상한과 다르다. 해당 frame 상한이 실제 provider 응답에 적합한지는 미검증이며, 전체 정상 media 지원이나 엄격한 wire 메모리 상한의 증거가 아니다. 원 response를 host가 소유하는 시점을 post-send 검증 이전으로 잡는 문제와 함께 설계·실제 fixture에서 다룬다. 후속 계약은 input frame buffer 상한·caller 분할·backpressure·oversize 오류와 미지원 범위를 각각 명시한다. 현재 우선순위는 typedcancel 실제 인수 뒤 제품 executor/CLI 연결이며 HTTP 수정·새 worker·네트워크 범위를 먼저 확대하지 않는다.

현재 생산 `browser_provider/protocol_observer.rs`는 Network.getResponseBody의 bounded JSON capture 경로다. takeResponseBodyAsStream/IO.read 구현은 없다. `browser_provider/media_observer.rs`는 원 STA IStream의4KiB prefix 관측이며 전체 media stream 어댑터가 아니다. legacy `infrastructure/browser/responses.rs`와도 책임·등록 상태가 다르므로 해당 callback 관측을 원 provider byte-stream 지원으로 승계하지 않는다. 원 response 재사용이 필수인 auth/session과 명시 새GET이 허용되는 public response를 구별한 뒤 host 방식을 선택한다. 새 구현/네트워크/실행은 이번 읽기에서0이다.

## 로컬 후보의 직접 관측

`C:/ProgramData/chocolatey/lib/ffmpeg/tools/ffmpeg/bin/ffprobe.exe`는 82620928 bytes, SHA256 `EFB014C8F39091C90F3616154FE20AB8470F5F943F6A48EC4FD30C479C7DD697`다. 이번에는 파일 읽기만 수행했고 실행하지 않았다. 설치 nuspec은6.1.0, 동봉 README는6.1-essentials_build-www.gyan.dev/GPL v3/소스 d4ff0020b4를 명시한다. README SHA `7DD81893C6532888CB9F37D2F894B17456F51B88E8A1D36FB1C15DE9AA02DBEF`, LICENSE SHA `8CEB4B9EE5ADEDDE47B31E975C1D90C73AD27B6B165A1DCD80C7C545EB65B903`다. 이는 설치 파일의 관측이며 대응 소스·외부 의존성·배포 요건 검증 완료가 아니다. 제품 배포 핀으로 자동 승격하지 않는다.

동봉 doc/ffmpeg-protocols.html에도 fd 섹션과 `fd: -fd file_descriptor`가 있다. 공식 현재 [protocols.texi](https://raw.githubusercontent.com/FFmpeg/FFmpeg/master/doc/protocols.texi)는 정규 파일 descriptor의 seek, URL 내 descriptor 지정 금지, 실제 -protocols 조회를 설명한다. 현재 master는 설치6.1의 revision 근거로 쓰지 않는다. README의 단축 revision을 사용한 원문 조회는 cache miss로 실패했으며 대응 소스 확인은 남는다. [공식 라이선스 설명](https://ffmpeg.org/legal.html)은 빌드 구성별 GPL 적용 가능성을 명시한다. 동봉 GPLv3를 LGPL로 바꿔 표시하지 않는다.

## 기존 경로와 실제 공백

- src/infrastructure/frame_capture/mod.rs의 발견은 현재 exe 옆/chocolatey/PATH 후보와 크기 조건이다. probe.rs는 pathname/file protocol 기반 별도 처리다. 버전 조회 경험을 owned fd 검증으로 승계하지 않는다.
- 채택된 download/media_probe.rs의 ProbeToolPin은 경로/hash/bytes이며 attempt.rs가 원 tool/ancestor guard와 원 input lease를 보유한다. capabilities/inspect는 원 Supervisor·하나의 deadline을 사용한다. 실제 도구 입장 근거는 외부 호스트 책임이며 임의 JSON URL/도구 경로를 권한으로 취급하지 않는다.
- media-download-provider-r1의 HttpTransport::open은 BrowserOriginalRequest와 외부에서 계속 구동되는 Tokio Handle을 요구한다. 원 browser admission은 일회성이며 실제 provider/header/auth/응답 재검증이 남는다. DownloadOwnedExecutor는 이미 존재하지만 구체 SourceOpener와 DownloadPreparation의 제품 조립이 없다. poll_close에서 단순 request drop을 runtime/DNS 물리 정리로 표시해서는 안 된다.

## 다음 직렬 구현과 실제 검증 제안

1. 먼저 새 Rust 실제 매체 fixture를 tests/media_download_owned.rs의 기존 core/retaining-host 흐름으로 연결한다. 새 검증 지원 책임은 tests/support/download_real_probe.rs로 분리하고 기존 부모에는 명시 module/case 등록만 제안한다. 매체는 재배포 가능한 로컬 고정 원본을 hash/bytes/출처/권한과 함께 선택한다. 아직 매체를 선택하거나 생성하지 않았다. 원본을 staged output으로 흘려 원 lease의 regular stdin을 실제 ffprobe가 읽게 하고, 기존 합성 child로 대체하지 않는다.
2. 별도 배정 후 동일 고정 실행 파일의 -version/-buildconf/-L/-protocols와 raw stdout/stderr·원 close receipt를 보존한다. fd 미지원이면 명시 Unsupported로 남기며 pipe/path 우회 성공을 owned fd 지원으로 표시하지 않는다. 원 매체의 container/tracks/packet count/duration·최종 hash/한 번 CAS를 검증한다. 성공·잘못된 매체·원 취소·client drop/unwind의 실제 owner/lease 종료를 구분한다. capabilities 성공은 매체 성공이 아니다.
3. 그 다음 provider stage에 새 infrastructure/download/http_source_opener.rs를 제안한다. 기존 SourceOpener를 구현하고 원 BrowserOriginalRequest/HttpTransport/부분 open 실패 소유자를 유지한다. host runtime의 소유·구동·종료 계약을 먼저 확정하며 새 전역 registry나 blocking Drop을 만들지 않는다. 단계2가 production 도구 신뢰를 대신하지 않는다.
4. 기존파일 변경 제안은 root Cargo.toml의 이미 locked reqwest 직접 의존성, application/download/mod.rs 및 infrastructure/download/mod.rs의 책임별 등록, 기존 browser metadata7이다. 정확한 preimage/manifest와 PM 배정 전에 이들을 편집하거나 stage를 합성하지 않는다. 제품 DownloadPreparation/operation registry/CLI 명령의 정확 경로는 원 실제 host 계약이 확정된 뒤 별도 목록으로 제안한다. URL을 영속 job payload로 복사해 원 admission을 재발급하는 구현은 금지한다.
5. 실제 CLI 원 provider 성공·오류·취소·저장까지 인수하고 나서 지원을 표시한다. HTTP200 direct 경로는 HLS VOD variant/audio/init/order/discontinuity/remux, 동일 객체 Range 재개, 정식 재리졸브·재시작 복구, modes/batch의 완료 근거가 아니다. 이 전체 잔여를 유지한다.

## 적용 근거

modoo.today의 test-fixture-and-mock-build를 다시 읽었다: `modootoday/software-qa/test-fixture-and-mock-build`, SKILL SHA `a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100`. 실제 입력/계약에서 fixture를 만들고 고립 evidence·원 cleanup/recovery 및 합성이 입증하지 못하는 실제 동작을 별도 인수하는 규칙을 적용했다. 이번 의미 검색의 agent replay/model benchmark 결과는 이 작업과 맞지 않아 채택하지 않았다. 컴파일·실행·성능 측정은 이번 체크포인트에서 수행하지 않았다.
