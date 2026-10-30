---
id: evidence:kinekino/20261029113915
title: 제품 자막 플러그인 codec 검사 r1
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 자막 플러그인 codec 검사 r1

2026-10-19 PM의 단일 검사 슬롯에서 packages/player-ui-plugins 독립 workspace를 검사했다. Meari plugin API는 작성 중이므로 이번 dependency에 넣지 않았다. registry를 제품에 복제하지 않았고 root Cargo/lock/SDK 파일을 수정하지 않았다.

| 명령 | 결과 | 범위 |
| --- | --- | --- |
| cargo fmt --manifest-path packages/player-ui-plugins/Cargo.toml --check | exit0 | 새 Rust 패키지 |
| cargo test --manifest-path packages/player-ui-plugins/Cargo.toml --offline -j1 | exit0, 계약 시험3/3 | 최초 독립 lock14개 생성, compile2.64초 |
| cargo clippy --manifest-path packages/player-ui-plugins/Cargo.toml --offline --locked --all-targets -j1 -- -D warnings | exit0 | 0.64초, 전체 target |

BelowNormal/-j1/기존 공유 target을 사용하고 정상 검사 중단·중복 cargo 없이 종료 후 슬롯을 반환했다. 증거는 runtime/quality-review/player-ui-plugins-r1/{logs,source-at-audit,source-hashes.json}에 보존한다. unit0/doc0을 추가 동작 시험 수로 세지 않는다.

첫 시험은 canonical `media_subtitles::application::convert::execute`를 in-memory SubtitleStorage와 연결한 host closure를 어댑터에 주입한다. VTT voice 정보의 loss가 있으면 명시 동의 전 NeedsInput/출력 없음, allow_loss 뒤 새 SRT 경로·overwrite=false·cue_count1·일본어 본문 보존을 확인했다. 이 메모리 저장소 시험은 실제 파일 publication이나 ApplicationClient 실행 증거가 아니다. production codec은 parser/storage를 직접 호출하지 않는다.

둘째 시험은 SDK RevisionConflict의 code·message 전달과 details/request_id/remediation/retryable 보존 경로를 검사한다(명시 assertion은 code/details/request_id/remediation/retryable). 셋째는 unknown field가 canonical DTO 경계에서 InvalidArgument가 되어 SDK 호출에 도달하지 않는 것과, SDK transport 없는 adapter의 Unsupported를 검사했다. 입력 schema 오류와 SDK usecase 오류를 같은 지원 판정으로 혼동하지 않는다.

Meari PluginAdapter impl/manifest 등록·permission 협상·cancel/event/lifetime·source binding, Native/Web 실제 소비, SDK root host closure assembly, 실제 OS 파일과 Web 권한, 화면 렌더·폰트·SVG·시각 인수는 미검증/미연결이다. 다음 gate는 본체 공개 plugin API freeze를 받아 같은 canonical codec에 연결하는 것이다. 전체13화면군·picker/playback/window 후속을 이 첫3시험으로 완료 처리하지 않는다.
