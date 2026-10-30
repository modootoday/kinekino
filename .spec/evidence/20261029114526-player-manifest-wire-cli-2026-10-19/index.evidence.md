---
id: evidence:kinekino/20261029114526
title: 실제 wire 헤더와 저장된 매니페스트의 CLI 재검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 wire 헤더와 저장된 매니페스트의 CLI 재검증

## 검증본

`runtime/quality-review/player-manifest-cli-r2/source-product`는 ExtraInfo 연결·schema2·`episodes.inspect_resolution`을 포함한 제품 source다. fmt와 workspace/all-targets strict Clippy41.051초, 전체289pass17ignored(root196pass1ignored)/340.463초, 두 bin1.141초가 통과했다. 실제 실행은28.08초 뒤 마지막 synthetic schema fixture 디렉터리 누락에서 실패했다. 그 전에 실제 Completed, 새 CLI 조회, 손상 거부와 원본 재조회가 통과했지만 전체 actual 성공으로 표시하지 않는다. r2 원로그·원문을 보존한다.

`player-manifest-cli-r3/source-repair/resolve_cli.rs`는 해당 테스트 디렉터리 생성 한 줄만 수리한 고정본이다. fmt0, strict Clippy1.568초0, 전체289pass17ignored/113.361초0, 두 bin921ms0이다. 실제 강화 CLI는1pass/22.18초, runner23.459초/exit0/launcher120640/child82896이다. EOF의 browser_worker_exited=true/completed_total=1과 소유 두 PID 부재를 확인했다. 실행은 affinity3·두 CPU·BelowNormal 및 자식 상속으로 제한했다. 기존30초 job deadline과 원 assertion을 유지했다.

| 파일 | SHA256 |
| --- | --- |
| player-cli.exe | 2f93910ca77de269eddef1a82bfae017e1e0b0b2543ee7ac5e31fe35622d5fe8 |
| player.exe | 940db26b1b2a20198b3a11baf0b72bf9330844bf3184947dc2c260de56b9cab0 |
| r3 jobs-harness-final.exe | 782a53d7c6fa67d6d322be7e7327b22c7ab20d6d24b07a9cc948b820ba5d667d |
| resource-runner.exe | 692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463 |
| Cargo.lock | 50323b23fb1bbf496e3ad42be71b4ccbdc3e1f38de1de5a5154a1ba5d78da5f7 |

제품 실행파일은 r2와 동일하며 r3에서 테스트 harness만 바뀌었다. 실제 Cargo jobs harness 경로는 test.log에 기록돼 있다. 모델 추론·다운로드·GUI 시각 검사는 수행하지 않았다.

## 실제 관측과 재조회

private 근거는 `runtime/quality-review/player-manifest-cli-r3/actual-cli/resolve-watch-cli`다. 같은 실제 회차의 자연발생 응답을 기존 browser session/frame/loader/native owner에 결합해 읽었다. manifest는 GET/XHR200/text/plain678bytes, body SHA256 `43bca27fe8cb6632effdae25cc6839921e3d625fa818ae9e54a33df5817f860f`, artifact `76cb79c2d5cf8c50da780001b34b150382bcc0c2749508629661260aa69d50d5`다. 실제 ExtraInfo의 Referer는 absent이고 수신1792402645244 Unix ms가 보존됐다. 빈 기본 event를 부재로 추정한 결과가 아니다. Cookie/Auth는 보존하지 않는다.

공통 job이 schema2 결과 artifact `97c2ede10c64eba06edc37a448719bf345782bb763b8c8c852ae41e320870dee`를 Completed로 게시했다. source descriptor SHA는 `13fcce4f093f940c7ea7f2fafa47c35fe115ebb5213122cb643d0239f437bb5b`다. provider 종료 후 별도 CLI 프로세스의 `episodes.inspect_resolution`이 원문 해시·요청 증거·HLS를 재검증했다. master/variant1/reference1/segment0/encryptedfalse/endlistfalse다. URI를 출력하지 않는 요약이다.

private 복사본의 manifest 첫 byte 변이는 fresh CLI에서 source_expired/exit8로 거부했고 원본 재조회는 exit0이다. 별도의 명시 synthetic schema1 fixture는 typed 조회 Unsupported/exit3, catalog.read_raw 원문 조회 exit0이다. 이는 실제 과거 유효 schema1 결과의 마이그레이션 증거가 아니다. 강화 actual의 raw 전체 재읽기·시각/소유 증거·결과 일치 assertion도 통과했다.

## 경계

각 session의64개 request 헤더 상한, redirect/중복 모호함 거부, 미관측 대기, work 전체4회 body 발급 상한을 유지했다. 직접 HTTP·헤더 재작성·URL 추정·복호화·숨김 상태 변경은 없다. [r1 실제 실패](player-manifest-cli-2026-10-19.md)를 보존한다.

`manifest_verified=true`/`hls-bounded-v1`만 검증됐다. referenced_resources_verified/media_decoded/native_playback/live_session_capability/url_expiration_verified는 false다. master의 참조 playlist·segment·키를 읽거나 재생하지 않았다. 다음 단계는 [공통 결과 계약](../../player/application/episode-resolution-result.md)의 실제 전송·수명·native/mpv 인수이며 전체 SDK/CLI·자막·제작·AI 골은 완료되지 않았다.
