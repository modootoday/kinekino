---
id: evidence:kinekino/20261029113950
title: 제품 TSX의 실제 Meari graph 검사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 TSX의 실제 Meari graph 검사

## r2: 실제 ModuleProgram 투영

PM 배정 슬롯에서 `project` 명령을 추가한 consumer의 fmt check, offline/locked `cargo test -j1`(0 tests,39.21초), strict Clippy all-targets(5.57초), build(6.87초), 실제 고정 실행파일 project가 모두 exit0이었다. test는 컴파일 확인이며 자동 동작 회귀 수를 부풀리지 않는다. BelowNormal/-j1/기존 공유 target을 유지했고 건강한 빌드를 중단하지 않았다. 현재 슬롯은 반환했다.

실제 명령은 `player-ui-consumer.exe project src/presentation/web_ui src/presentation/web_ui/viewing/source-graph.json runtime/quality-review/player-ui-consumer-r2/mount-request.json`이다. mount 요청은 document_id=`player_viewing_consumer_r2`,epoch=1과 `productName`/`connectionLabel`/`timeLabel`의 실제 String 값 세 개다. productName은 이번 관측 당시 src/domain/product.rs PRODUCT.display_name의 snapshot이며 새 제품 식별 설정이 아니다.

원본4파일의 graph를 ModuleProgram::mount에 전달해 `root`, `root@0/0`, `root@0/1`, `root@0/2`의 별도 instance와 source origin을 투영했다. state revision은 모두0이고 hook state는 없는 입력이다. disabled 버튼의 실제 SDK 동작·event dispatch·props update를 검증한 결과가 아니다. 결과는 stage=`module_program_projection`,renderer_verified=false,assets_verified=false,visual_verified=false다. renderer/document mount, Native raster, Web DOM, 폰트/SVG/CSS 공급, builder bundle, SDK intent 및 시각 인수는 남아 있다.

같은 고정 binary로 추가 컴파일 없이 epoch0과 timeLabel 누락 요청을 실행했다. 둘 다 exit1이며 각각 viewing/player.tsx `unsupported`/span0..0/`module document scope/epoch required`, span306..558/`exact declared bindings required`를 원문 JSON으로 보존했다. 정상 입력과 오류 입력 모두 Meari의 module mount 경로를 사용하며 caller 오류를 임의 기본값으로 숨기지 않는다. 로그와 두 요청은 동일 r2 checkpoint에 있다.

로그·mount request·고정 binary·consumer/web_ui/Meari core/native source snapshot은 `runtime/quality-review/player-ui-consumer-r2/`에 있다. binary SHA256은 `10344ad407e4d168d631cb175976ef127bea526df06b8b6138b7b466430ba074`다. `meari-wrapper-hashes.json`은 이번 freeze에서 대조한 application.rs/module_app_bundle.rs/Native source_modules.rs/Web source_modules.rs 네 파일 hash다. 네 파일의 현 mtime은 UTC04:28:13이며 담당/PM은 이전31검사 반환 후 이번 배정 전에 추가했다고 정정했다. 이번 test 로그에 core/native/builder 재컴파일이 있으나 wrapper 기능 자체는 호출하지 않았다. Web cfg 실행이나 builder graph bundle 지원으로 확대 해석하지 않는다.

## r1: 원본 graph 파싱

2026-10-19 PM 배정 슬롯에서 tools/player-ui 독립 Rust consumer를 검사했다. Core9 공개 parse_module_graph API를 사용했으며 원본 TSX를 flatten하거나 별도 HTML로 대체하지 않았다.

| 검사 | 실제 결과 | 범위 |
|---|---|---|
| cargo fmt --check |종료0|consumer Rust workspace |
| cargo test --offline -j1 |종료0, 0 tests|빌드 검사이며 동작 회귀 통과로 세지 않음 |
| cargo clippy --all-targets -- -D warnings |종료0|consumer 전체 target |
| 실제 viewing source graph |종료0|원본4파일의 모듈 파싱/조합 |

entry는 viewing/player.tsx의 ViewingPlayer이며 window-chrome/transport-bar/start-screen을 실제 local import로 해석했다. 결과는 stage=module_graph_parse/runtime_verified=false/visual_verified=false다. generator 실행·Web/Native document·builder bundle·asset delivery·SDK intent·브라우저 렌더·시각 일치는 이 검사로 증명하지 않는다.

로그/고정 실행파일/source snapshot은 runtime/quality-review/player-ui-consumer-r1/에 보존한다. 고정 player-ui-consumer.exe SHA256은 `cfe73c26f64bf369c18da66743921f70e126d31bdc0b21260140b9abf509a0cf`다. 첫 offline lock/공유 target 캐시를 만들었고 정상 실행을 중단하거나 중복 cargo를 시작하지 않았다. root SDK/Cargo/lock은 변경하지 않았다.

## 같은 고정 도구로 보존한 실패 재현

추가 컴파일 없이 각 명시 inventory를 같은 실행파일로 검사했다. 입력을 통과시키려고 원본을 단순화하지 않았다.

| 입력 | 결과 | 원본 byte span | 진단 |
|---|---|---|---|
| verification/keyed-jobs.tsx |exit1|92..131|prop type must be string/boolean |
| studio/queue-footer.tsx |exit1|667..676|only string/bool literals and props.name supported; hooks/effects/expressions unsupported |
| studio/frame.tsx |exit1|0..39|plain source-owned component imports required |

각 source-graph.json과 stderr 원문은 같은 checkpoint의 keyed/queue/frame 로그에 있다. 실패는 각각 typed collection, attribute expression, ReactNode/type-import 슬롯 요구를 좁히는 근거이며 전체 기능 불가능을 뜻하지 않는다. Meari 담당에 원본·파일/span·기대 동작·인수 기준을 인계했다.

다음 단계는 Web graph document와 Builder graph/bundle의 실제 연결 및 CSS/assets·typed host bridge다. 제품 시나리오는 자막 원문/수정/명시 채택을 공통 Rust SDK에 연결한다. 실제 기능 검증은 SDK/CLI, UI 상태/입력/접근성/시각은 별도 중앙 슬롯에서 검증한다. 현재 이 다음 단계는 미완료다.
