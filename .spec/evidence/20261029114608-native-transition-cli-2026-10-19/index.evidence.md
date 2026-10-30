---
id: evidence:kinekino/20261029114608
title: 회차 전환 root 등록과 CLI 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 회차 전환 root 등록과 CLI 검증

## 입력과 범위

중앙에서 Meari·UI 검증 반환 후 SDK 단일 heavy 슬롯과 root 등록을 승인했다. [준비 관측](native-transition-preparation-2026-10-19.md)의 20개 준비 파일을 책임별 경로에 등록하고 기존 파일 다섯 개는 문맥 패치로 연결했다. 등록 전 입력은 `runtime/quality-review/native-transition-cli-r1/source-before`, 준비본은 `pending-source`, 적용 패치는 `registration.patch`에 보존했다. Cargo manifest/lock과 Meari 제품 등록은 이 변경에 포함하지 않았다.

공통 application 전환은 검증된 목록의 기존 EpisodeQueue 순서로 대상 회차를 선택하고 durable episodes.resolve 작업을 접수한다. resolve 완료 후 명시적 commit만 기존 엔진을 종료하고 새 엔진으로 교체한다. host 하나가 browser jobs·catalog·native owner·단일 전환 slot을 함께 소유한다. 실제 재생과 성공 품질을 resolve 완료나 프로세스 생성으로 대신하지 않는다.

## 관측한 검사

모든 receipt는 위 runtime 경로에 있다. 두 논리 CPU·BelowNormal·descendant Job·기존 Cargo cache·offline/locked/-j1·단일 테스트 thread를 사용했다.

| 검사 | 결과와 제한 |
| --- | --- |
| fmt-first | exit1. 신규 등록 포맷 차이를 보존하고 cargo fmt로 정리 |
| check-first | exit0, 33,933ms. 전체 타깃 컴파일. unused 필드/접근자 경고 두 건은 이후 제거 |
| focused-first | exit0, 39,252ms. transition 이름 필터 12pass: 신규8+기존4. startup/출력 검사를 포함한 준비12 전체 집계가 아님 |
| full-first | exit0, 278,828ms. 319pass/0fail/18ignored. startup 실패·복구 두 검사와 diagnostics complete/empty collector 검사 통과. 아래 교체 fault 보강 전 입력 |
| replacement-fault-first | child exit101, 6,972ms(wrapper exit1). 테스트의 unwrap_err가 성공 DTO의 Debug를 요구한 컴파일 오류. 제품 DTO를 바꾸지 않고 테스트에서 err().expect로 수정 |
| replacement-fault-r2 | exit0. 내부 교체 실행 어댑터의 8pass(그중 bounded child probe 1) |
| replacement-fault-r3 | exit0, 22,949ms. 10pass(그중 bounded child probe 1). EOF cleanup 원오류 보존과 실제 소유 프로세스 종료 실패 검사 추가 |
| strict-first | exit0, 23,582ms. 보강한 교체 경로와 테스트를 포함한 all-targets clippy -D warnings |
| fmt-replacement-final | exit0, 1,138ms. 포맷 적용. 최종 fmt --check와 구별 |
| full-final | exit0, 268,717ms. 329pass/0fail/18ignored. 직접 교체 fault 보강 포함 |
| fmt-gate | exit0, 3,898ms. cargo fmt --check |
| jobs-harness-final | exit0, 11,187ms. 전체 검사 뒤 방향별 실패 문구 세 곳과 Previous 호출 서식만 변경하고 실제 harness 재빌드 |
| actual-first | exit101, 104,559ms. 0pass/1fail/15filtered. 두 방향 commit·PNG 이후 play 진행 관측 실패. 전체 성공으로 집계하지 않음 |
| bins-final | exit0, 1,152ms. GUI/CLI build와 아래 실행 파일 pin |
| actual-r2 | exit0, 103,036ms. 본문103.00초, 1pass/0fail/15filtered. 동일 source/bin, fresh actual-cli-r2 |

교체 검사는 production `replace_verified`가 호출하는 동일한 `replace_using` 경로에 내부 launch/close 어댑터를 주입한다. 외부 CLI나 SDK에 명령·오류 주입 옵션을 노출하지 않는다. 고정된 Rust 테스트 실행 파일의 자식 probe를 실제로 소유하고 종료하며, 성공한 child 생성은 미디어 재생 증거가 아니다. fixture 실행 파일 pin은 작은 synthetic 파일이며 실제 mpv 실행 경로 검증을 대체하지 않는다.

## 직접 교체 fault의 경계

- unsupported preflight: old PID/instance 그대로, old child 생존, launch 호출 0.
- old process close 실패: 같은 live child 보존, launch 0, 명시적 host shutdown으로 종료.
- old close 진단 저장 실패: 정지된 old owner 보존, launch 0, 진단 게시 재시도 후 종료 receipt.
- old 종료 후 new launch 실패: old 종료 receipt, 새 프로세스/owner 없음, 자동 복구 없음.
- startup 정리 미확인: 새 owner로 recovery 전달, 기존 instance와 구분, 명시적 shutdown이 모든 worker 종료를 확인. 시작하지 않은 stdout 때문에 diagnostics complete는 false.
- startup 진단 저장 실패: 이미 정지된 새 owner도 보존해 게시를 재시도.
- launch 중 EOF: 새 child 종료, 영구 종료 뒤 재교체 거부.
- EOF의 새 process close 실패: live child와 원 cleanup error 보존, host 재시도 receipt.
- EOF의 새 진단 게시 실패: 정지된 새 child와 원 cleanup error 보존, 같은 instance로 host 재시도 receipt.

## 첫 실제 실행 실패

exact case는 `native_episode::actual_resolve_and_owned_native_lifetime`이며 `harness-list.txt`에 --list를 보존했다. 첫 실행은 fresh `actual-cli`에서 초기 resolve·다음·이전 회차 resolve/commit·새 영상/시간/오디오·PNG와 옵션 복원까지 진행했다. 그 뒤 고정된 10초 seek 후 play 검사의 기존 5초 기한 안에 +500,000us 진행을 관측하지 못했다. 접수는 accepted/unobserved, instance3/source9d2cec3373e81e64e99dac71b2ecf3fdbe86d2b8a9d4de67d3eb88b799b1b1f5의 connected=true/paused=false/idle=false/error=false/1920×1080/audio2/rate1.0이었으나 position은 정확히10,000,000us에 머물렀다. mpv 보존 출력은 seek 이전 time3초/Cache2.5초, 이후 time10초/Cache0초를 보였다. 명시 HTTP error/timeout/failed segment 문구는 없었다. 캐시 고갈과 일치하지만 네트워크 원인을 확정하거나 제어 성공으로 승격하지 않는다.

첫 실패의 `native-failure-stop.stdout`은 engine3/IPC/stdout/stderr 모두 종료, diagnostics complete=true/truncated=false/read_failed=false, provider browser worker 종료, completed jobs2, pending transition null을 기록했다. `native-failure-cleanup.json`은 forced=false/wait=true다. runner111248/harness113296 종료와 해당 fixture 실행 파일·프로필·mpv 프로세스 부재는 `owned-processes-after-first.json`의 []로 남겼다. 첫 실패 디렉터리와 네 PNG를 보존했다. Next PNG는 성과 일본어 화면 글자, Previous PNG는 불타는 성의 영상 프레임으로 시각 확인했다. 이는 엔진 subtitle overlay 증거가 아니다.

동일한 고정 바이너리와 조건으로 fresh `actual-cli-r2`에서 재관측한 실행은 전체 통과했다. 기한·PNG 해상도·전체 decode 조건을 완화하지 않았고 제품 코드나 테스트의 완료 조건도 바꾸지 않았다. 이는 첫 실행 seek 후 정지의 원인 해결이나 신뢰성 보장을 뜻하지 않는다.

## 두 번째 실제 실행 관측

목록·raw evidence 재검증 뒤 같은 combined host에서 Next와 Previous를 차례로 접수하고 서로 다른 두 회차의 실제 대상 descriptor를 대조했다. 중복 시작은 RevisionConflict이며 전후 완전한 jobs ID 목록이 같았다. resolve 중 이전 engine/artifact를 유지했고 명시적 commit에서 각각 이전1→새2, 이전2→새3으로 교체했다. 두 old receipt 모두 engine/IPC/stdout/stderr 종료 true, diagnostics complete=true/truncated=false/read_failed=false다.

| 방향 | target descriptor SHA256 | 새 resolution artifact |
| --- | --- | --- |
| Next | 4a3bd001b2c701236d07524989ceae60c3d1c9b87809992be93028547937cebb | 42d13ca31aab4a860e195a6fdfd228e13f95359703abcb6906b4df3caad17e26 |
| Previous | 13fcce4f093f940c7ea7f2fafa47c35fe115ebb5213122cb643d0239f437bb5b | c56e22e25f8a42dc8c0888a597b2bf40ad682eb90832b1c494b99e62c0afeedc |

각 새 엔진에서 connected/idle=false/1920×1080/audio2를 관측하고 해당 준비 완료 표본 대비 1초 이상 time 증가를 확인했다(최종 Next1,172,567us·Previous1,214,278us). 전체 PNG hash/bytes/instance/artifact/크기/픽셀·버퍼 상한과 독립 decoder next_frame+finish를 검사했다. 기존 4초 publication deadline은 그대로다. exact PTS/color parity는 false, subtitles_included=false, window_capture=false다.

| PNG | 크기/bytes | SHA256 | 관측 bracket(us) |
| --- | --- | --- | --- |
| next-native-frame.png | 1920×1080 / 9,132,799 | dbf17753264a47af6fe576b1759ac135b5b65a4deb55c4ae27bb1c9191be0098 | 1,297,700→2,006,733 |
| previous-native-frame.png | 1920×1080 / 5,831,078 | 5ce9d53cdac06fa268a61b681caf999951e094a80357f109940a89dd81de83f3 | 1,255,989→1,631,367 |

두 PNG를 직접 시각 확인했다. Next는 밤의 성과 화면 일본어 글자, Previous는 불타는 성의 프레임이며 검은/잘린 이미지가 아니었다. 번인 글자를 엔진 subtitle overlay로 해석하지 않는다. 소비한 transition handle 재조회/재commit은 NotFound, old engine control은 RevisionConflict였다. 이후 rate1.25→1.0·audio/video disable/복원·복원 PNG·pause/seek10초/volume13.25→0/play를 검사했고 play time10,556,944us까지 진행했다. 닫기3→재열기4→late3 거부→EOF4 종료 및 host 종료 후 open Cancelled까지 통과했다.

`native-stop.stdout`은 EOF engine4 및 모든 worker 종료 true, 빈 stdout/stderr의 실제 EOF를 관측한 diagnostics complete=true, provider browser worker 종료 true, completed jobs2/cancelled0/interrupted0, pending transition null을 기록한다. runner101008/harness119032 종료와 관련 mpv/fixture exe/profile 프로세스 부재 []는 `owned-processes-after-r2.json`에 보존했다. 실행 후 CLI/GUI/harness/lock pin도 동일했다.

이 checkpoint의 예약된 두 방향 실제 인수는 위 범위에서 통과했다. 첫 실행의 seek 후 캐시0 진행 정지는 미해결 관측으로 남긴다. 전체 회차/사이트 완전성·청취 품질·자막 양성·정확 PTS/색상·GUI 시각 일치·전체 제품 완료는 검증하지 않았다. [진행 계획](../../../plans/player/native-episode-transition.md)과 [옵션 r3 기준](native-options-cli-2026-10-19.md)을 함께 본다.

## 사용자 관측 후속: 커서와 무음

이 실행에서 volume0은 테스트의 명시 설정이며 청취를 검증하지 않았다. 그러나 이를 모든 GUI 무음이 정상이라는 근거로 확대하지 않는다. 소스 감사에서 legacy `src/infrastructure/mpv/mod.rs`의 고정 `--mute=yes`도 발견했다. GUI state 기본volume80과 별개로 강제 mute가 적용될 수 있는 제품 결함이며, owned/network_plan 경로에는 해당 고정 mute가 없다.

두 launch 경로 모두 당시 `--input-cursor=no`를 사용했고 cursor-autohide를 명시하지 않았다. 고정 mpv의 `--no-config --list-options`는 자동 숨김 기본1000ms/fs-only=no/input-cursor=yes를 보고했다(`mpv-options-receipt.json`189ms/0). 공식 [mpv manual](https://mpv.io/manual/stable/)에서 input-cursor는 pointer 이벤트 수신, cursor-autohide=no는 엔진 자동 숨김 비활성이다. 이 근거만으로 사용자 hover 즉시 숨김·이동 후 미복귀의 원인을 확정하지 않는다. 사용자가 커서를 수정 대상으로 확정했으므로 정상 동작이나 단순 미검증으로 종결하지 않으며 [후속 수리 계획](../../../plans/player/native-pointer-and-audio.md)에서 별도 수정·실행한다. 현재 전환 r2 pin의 성공은 이 후속 수리 결과를 포함하지 않는다.

## 고정 바이너리

`final-pins.txt`의 SHA256은 아래와 같다. source-final은 등록된 root 입력과 당시 미등록 UI 파일을 함께 복사한 보존본이며 독립 Meari 작업 전체를 봉인했다는 뜻은 아니다. full-final 뒤 차이는 방향별 실패 문구 세 곳과 Previous 호출 포맷뿐이며 새 harness에 반영했다.

| 입력 | SHA256 |
| --- | --- |
| player.exe | 45f14ac9cd0167e9881b33c3b807692ffee335d5fd9a896c144873a80c92f6b9 |
| player-cli.exe | 49a0404263a0b83396891a2316362da333aca40605e8bc78a789b8dd345d5e36 |
| jobs harness | 9a242b118431c37889f2098caed0c400335d035bd53df39dc83b6ade8f9c8732 |
| resource-runner | 692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463 |
| mpv.exe | f8bf7e8fd8d6696e35441c55091498060e4594f724a132fd6c46579341bc0790 |
| Cargo.lock | 50323b23fb1bbf496e3ad42be71b4ccbdc3e1f38de1de5a5154a1ba5d78da5f7 |
