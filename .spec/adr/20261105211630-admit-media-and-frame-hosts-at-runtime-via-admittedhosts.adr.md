---
id: adr:kinekino/20261105211630
title: Admit media and frame hosts at runtime via AdmittedHosts
status: active
created: 2026-11-05T21:16:30Z
updated: 2026-11-05T21:16:30Z
rel:
  governs: [crates/kinekino-player/src/domain/navigation.rs, crates/kinekino-player/src/domain/player_document.rs, crates/kinekino-player/src/domain/admitted_hosts.rs]
  depends_on: [adr:kinekino/20261030200000]
---

# Admit media and frame hosts at runtime via AdmittedHosts

## Context

로드맵 M3-1(PG-5)이다. 플레이어 domain의 `navigation`·`player_document`이 미디어·프레임 호스트를 컴파일 타임 문자열 매치로 허용하고 있었고, 그중 실 CDN 호스트가 기본 허용 목록에 남아 호스트·플러그인 경계(ADR 0011)와 어긋났다. 카탈로그 쪽은 이미 프로세스 전역 `LocationRules`(`install_rules`)로 플러그인 document 절에서 설치한다. 미디어·프레임도 같은 패턴이 필요하다. `SourceManifest::allows_media_host`는 스트림 resolve에서만 쓰이며, 네비게이션·플레이어 문서 허용에는 연결되어 있지 않았다.

이번 조각(M3-1a)은 domain 정책 객체와 설치 API·테스트만 도입한다. 플러그인 매니페스트 배선(M3-1b)과 episodes/playback/projects/episode_queue/response_purpose 등 남은 하드코드 치환(M3-1c)은 후속이다.

권고안(루틴 진행):
1. `AdmittedHosts` 값 객체 — `allows_media_host`와 같이 정확·대소문자 무시 호스트 매치.
2. navigation 옆 프로세스 전역 설치 API(`OnceLock`), `install_rules`와 동일 패턴.
3. `navigation_allowed` / `player_page` / `PlayerDocumentLocation::admitted_response_origin`이 설치·활성 호스트를 참고하며, 실 CDN을 컴파일 타임 기본으로 두지 않는다.
4. 테스트 픽스처 호스트는 `catalog.example.test`·`frame.example.test`·`media.example.test`만.
5. `cfg(test)`이고 미설치면 픽스처로 폴백해 기존 application 테스트를 유지. 비테스트·미설치는 거부 기본(deny-by-default).
6. 추가 호스트 설치 시에만 그 호스트의 `/video/<32 hex>` 문서 모양이 허용되고, 미설치·lookalike는 거부됨을 새 테스트로 증명.
7. M3-1b: 플러그인 open 시 `media_hosts`(필요 시 document.host)로 설치. M3-1c: 남은 하드코드 치환.

skill: 조회 결과 무관.

## Decision

1. **`AdmittedHosts` 값 객체를 domain에 둔다.** 호스트 목록은 정확 일치·ASCII 대소문자 무시. `admits`는 네비게이션 허용, `admits_player`는 네이티브 플레이어 활성화·플레이어 문서 origin에 쓴다. 생산 경로에서 `AdmittedHosts::new(hosts)`는 두 목록을 같게 둔다(미디어 호스트 설치). 테스트 픽스처만 `with_player_hosts`로 catalog를 네비게이션에만 넣고 frame·media를 플레이어 목록에 둔다.
2. **`install_admitted_hosts` + `OnceLock`.** `kinekino_core::source::document::install_rules`와 같이 프로세스당 한 번. 이미 설치돼 있으면 `false`.
3. **활성 호스트 조회.** 설치분이 있으면 그것. 없고 `cfg(test)`이면 `AdmittedHosts::fixture()`. 비테스트·미설치면 빈 목록(전부 거부).
4. **호출부.** `navigation_allowed`는 활성 `admits`. `player_page`·`admitted_response_origin`은 활성 `admits_player`. 공개 함수 시그니처 `navigation_allowed(&str)` / `player_page(&str)`는 유지한다.
5. **문서 전용 origin.** `admits_player`에만 있고 `admits`에는 없는 호스트는 `PlayerDocumentLocation` 모양(`/video/<32 hex>`)으로만 네비게이션·player_page에 들어온다. 일반 경로(루트·lookalike)는 거부한다.
6. **테스트 격리.** 단위 테스트에서 설치 효과를 증명할 때는 프로세스 전역 `OnceLock`을 오염시키지 않도록 `cfg(test)` 스레드 로컬 오버라이드(`with_admitted_hosts`)를 쓴다. 생산 API는 여전히 `install_admitted_hosts`다.
7. **범위.** 이 ADR은 M3-1a만. 매니페스트 배선·다른 파일 하드코드는 각각 M3-1b·M3-1c.

## Rationale

- 실 CDN을 호스트 바이너리에 기본 허용으로 박는 것은 플러그인 경계 위반이며, 선언되지 않은 출처를 deny-by-default로 두는 편이 안전하다.
- 카탈로그 `LocationRules`와 대칭이면 M3-1b에서 플러그인 open 경로에 붙이기 쉽다.
- 픽스처 폴백은 기존 application·브라우저 테스트가 이 조각에서 한꺼번에 깨지지 않게 한다.
- `admits`/`admits_player` 분리는 “카탈로그 페이지는 늦은 미디어 콜백으로 네이티브 플레이어를 켜지 않는다”는 기존 `player_page` 불변식을 픽스처에서도 지킨다.

## Consequences

- domain에 실 CDN 컴파일 타임 기본 허용이 없다. 추가 CDN은 설치(또는 테스트 오버라이드) 후에만 플레이어 문서 모양으로 허용된다.
- 비테스트 빌드는 설치 전에 네비게이션·플레이어 문서를 모두 거부한다. M3-1b가 플러그인 open 시 설치하기 전까지 생산 경로에서 명시 설치가 필요하다.
- M3-1b: 플러그인 매니페스트 `media_hosts`(및 필요 시 document.host를 네비게이션에 합류)로 `install_admitted_hosts` 호출.
- M3-1c: `playback.rs`·`episodes.rs`·`projects/validation.rs`·`application/episode_queue.rs`·`response_purpose.rs` 등의 남은 예시 호스트 하드코드를 이 API로 치환.
- 테스트는 픽스처 호스트만 기본으로 쓰며, 실 CDN 이름은 설치·오버라이드 증명 테스트에만 등장할 수 있다.
