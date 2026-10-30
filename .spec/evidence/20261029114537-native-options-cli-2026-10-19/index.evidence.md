---
id: evidence:kinekino/20261029114537
title: native 속도·트랙 선택 CLI 검증 이력
status: active
created: 2026-10-29
updated: 2026-10-29
---
# native 속도·트랙 선택 CLI 검증 이력

## r1 고정본과 최초 진단

`runtime/quality-review/native-options-cli-r1/source-product`는 domain playback_options, application 옵션 포트/DTO·typed client·schema, mpv bounded 관측·인스턴스/목록 검사 및 기존 실제 harness의 확장을 포함한다. 첫 fmt는 nested 테스트 모듈 경로 오류였고 첫 strict7794ms101은 private_interfaces다. field를 mpv 내부 가시성으로 좁히고 명시 path를 추가했다. 수리 strict15196ms101의 helper8인자는 공통 SelectEpisodeTrack DTO로 묶었다. source-repair의3파일 및 최초 로그를 보존했다.

최종 fmt0·workspace/all-targets strict15332ms0·전체307pass18ignored(root214pass1ignored)/315732ms0·두 bin948ms0다. root Cargo의 실제 jobs harness와 --list exact case를 대조했다. UI web_ui_adapter는 root presentation/mod.rs 미등록, Meari는 Cargo exclude이므로 이 WIP를 실제 root compile 입력 불변 근거에 합치지 않는다. snapshot은 전체 외부 컴파일 입력 봉인이 아니다.

| r1 파일 | SHA256 |
| --- | --- |
| player-cli.exe | d9a4ac4e02f0d38a5a035be732d41ee186b10b81787e2607c596a75f654626d9 |
| player.exe | 37c02d5b6796ff24c1696b10cc19cb8b6faeca797e870476db7698f66a976b3c |
| jobs-harness-final.exe | 241125cc32b58781b3438b3c2adcaaf678e73a2b466d050fd683f6412f94d647 |

실행은 기존 runner의 affinity3·두CPU·BelowNormal·자식 상속과 Cargo offline/locked/-j1/RUST_TEST_THREADS1을 유지했다. [기존 native-r3 인수](native-episode-playback-2026-10-19.md)의 실행파일·성과와 별도다.

## 실제 r1 전체 실패

actual은51862ms/launcher41412/child115952/terminaltrue/exit101,0pass1failed/51.69초/15filtered다. 처음 실패한 assertion은 options-restored-frame이0 대신10을 반환한 것이다. 메시지는 `Export deadline expired before output publication`이며 기존4초 deadline을 넘은 게시를 거부했다. 전체 actual 실패를 성공으로 바꾸지 않는다.

그 전에 새 resolve/schema2 결과 `89edea182c1134169b193fb1020da734b6dbfe5ab2e5d19e633f1dec179f30aa`와 기존 fresh CLI/tamper/legacy 경계 및 초기 native 프레임을 통과했다. 초기 PNG는1920x1080/4970502bytes/SHA `44e7bac6981cce234f345d00a1688b59796aa98318559d797cd3ea060bd97a5f`, 시각 범위1506233–2632367us다.

실제 options.rate1.25와 재생 시간 진행 후1.0 복원을 관측했다. 초기 catalog revision3에 h264/video1·aac/audio1이 있었으며 id는 종류별로만 유일했다. subtitle entry는 없고 실제 subtitle selection은 disabled다. 없는 subtitle을 만든 것이 아니다. audio disable/revision4/selected=false 및 실제 aid disabled, audio 복원/revision5, video disable/revision6와 복원/revision7을 확인했다. 이전 revision 요청과 잘못된 종류·없는 ID는 거부됐다. 이 선택 속성 관측은 청취·모든 트랙의 디코드 품질을 의미하지 않는다.

복원된 영상의 staging PNG는 `.native-frame-110224-1/frame.png`,7103727bytes/SHA `dcbd5b2fd833ccb055555ffc11df9705e98faf9fcc0533e8b5bea5bda8f2f020`다. 제품의 전체 PNG 검증 뒤 publication 단계에서 deadline 오류가 났으므로 최종 출력은 게시하지 않았다. 원 staging은 보존했다. 파일 작성 시각은10:55:05–10:55:06UTC, RPC 응답 파일은10:55:09UTC이나 정밀한 단조 stage 시간 측정은 아니므로 이 차이만으로 어느 단계가 병목인지 확정하지 않는다.

NativeOwner 실패 cleanup은 forced_owned_child_stop=false/wait_succeeded=true다. EOF는 instance1 engine/IPC/stdout/stderr 종료를 모두 true로 반환했다. diagnostic artifact `dff05af940dcef14bc83c81d5b6a769ef6a37632ebab617bcc5312cfa54d7ab6`의 wrapper9731bytes/body SHA `a983a267d92d4e3565e0a5379b146dac86d39a246bbcc081bb8746bfd62f4c87`, stdout6813bytes/SHA `688a4ba4af6e3af36f542b0f65c9ccfb7e7cc69544a28e61bc849c85377660d4`, stderr0bytes다. complete=true/truncated=false/read_failed=false다. runner/harness 부재를 확인했다. 실패 뒤 나머지 기존 제어·reopen·EOF 정상 경로 assertion에는 도달하지 않았으므로 전체 회귀를 통과했다고 주장하지 않는다.

## r3 실제 인수 통과

`runtime/quality-review/native-options-cli-r3`에서 fmt3149ms0·strict18246ms0·전체307pass18ignored234247ms0·두bin1493ms0 뒤 동일 actual이1pass0fail/38.96초, runner39963ms0으로 종료했다. launcher69952/harness24040은 종료 후 부재를 확인했다. 두 logical CPU/BelowNormal/-j1/단일 테스트 thread와 기존4초 게시deadline을 유지했다.

고정 SHA256: CLI `aa6ad17b81e8f4b019a5c7a332b3e7a187a91527b37dd631202d0ec3e2446d9c`, GUI `d58941cd6878eda542841b38f26b1a532b2f322edc1f3c43105dfb8b881faffa`, harness `733f0b89ec01c69e127021b997418589db29b67f1f22b1412a3ae664e6a21690`. runner/lock/mpv pin은 기존과 같고 `final-binary-sha256.txt` 및 원 receipt에 보존했다.

새 resolve artifact `70a072bc594f6ccad22f41e079c2c2dcdaa27b00bda43e2a48e957d23e48869c`를 사용했다. 초기 PNG1920×1080/6242752bytes/SHA `8c6baeaad1712115da49e8d27048dcb1f40ec6c04cbf5bef82f239bd4e739533`, 시간 bracket1381111–1714778us다. audio/video disable·복원 뒤 PNG1920×1080/7377161bytes/SHA `ed874ad3642a4d19dfda134bad6d6f2e1f1df0f4725048fc87d2c75bd8d95922`, bracket3883611–3883611us를 실제 게시하고 두 파일을 열어 성/불 장면과 복원 장면을 직접 확인했다. 원 미디어에 들어있는 문자는 보이지만 엔진 subtitle overlay 포함을 증명하지 않는다. frame_time_exact/color_parity_verified는false다.

rate1.25에서3174567→3675067us 진행 후1.0복원, catalog revision3의 video1/h264·audio1/aac를 관측했다. audio disable4/복원5, video disable6/복원7과 실제 선택 속성을 확인했다. 이전 revision·없는 ID·잘못된 종류·잘못된 instance·범위 밖 rate를 거부했다. subtitle entry는 없고 disabled이며 subtitle 양성 인수로 확대하지 않는다. 원래 pause/seek10000000us/volume13.25→0/play10598656us와 시간 진행도 통과했다. 복원 직후 paused snapshot의 audio_channels는null이었고 후속 seek/play에서는2다. 실제 청취는 미검증이다.

일반 close(instance1)에서 engine/IPC/stdout/stderr 종료true, after-close NotFound, 새 instance2 뒤 이전ID close RevisionConflict, EOF2의 영구 host 종료 및 모든 worker 종료true, 종료 뒤 SDK open Cancelled assertion을 통과했다. instance1 diagnostic artifact `00715e1d20dff20e73eded985fbca44005fe14ce6b09bc309dcfff44c3dac2d8`, wrapper10567bytes/body SHA `a541ef27058cf718c03b4775c3d8c22f895f12ea7fe9784082be8f15c269c90c`, stdout7439bytes/SHA `ece5e815ea8e903187aabdb4a44bceace92ac252df082a0d2f633c874ea010fa`, stderr0, complete=true/truncated=false/read_failed=false다. EOF2 wrapper644bytes/body SHA `34ed9f4addea6626bc8638dfd757b2ef31676f13497cd16876df38d2a34370a2`는 빠른 종료로 양 stream0이다.

filter0 변경 뒤 이 actual은 통과했으나 r2와 동일 픽셀의 통제 성능 비교나 성공 단계별 시간은 측정하지 않았다. 따라서 필터만의 정밀 가속 배율이나 모든 장면의4초 성공을 주장하지 않는다. r1/r2 actual 실패는 그대로 유지한다. 실제 회차 전환·GUI 조립·제작·AI 및 전체 제품 goal은 남는다.

## r2 제한 유지 성능 수리

r2 최종 관측: 전체307pass18ignored/252640ms0, bins1247ms0 뒤 실제 gate는45781ms101·0pass1fail/45.74초다. 같은 options-restored-frame publication timeout이며 monotonic screenshot153ms/read+verify3791ms/publication350ms/total4295ms였다. compression1만으로 해결되지 않았다. read+verify가 주시간이라는 관측과 구체적인 PNG 내부 병목 가설을 구별한다. 원문은 `runtime/quality-review/native-options-cli-r2`에 보존했다.

r2 초기 출력PNG5618598bytes/SHA `7e2dc77dfd8ca23a5c705e202c67642a77bbab386013aba9122b9d1884f9a054`, 복원 staging8857210bytes/SHA `f816884d783868a0f921636abdc985cccd18ac8b4e62c8a0baa252461a107b3c`를 보존했다. 실패 cleanup은 forced_owned_child_stop=false/wait_succeeded=true, instance1 engine/IPC/stdout/stderr 종료 모두true다. diagnostic artifact `446948f2867ba3cfa81eecb929834a2b1d4c3a2e2e7e4f7fecb05a0c9f282b80`는 complete=true/truncated=false/read_failed=false다. launcher47016/harness42224의 종료 후 부재를 확인했다. 이후 기존 제어/reopen 인수에 도달하지 않았다.

`png_validation::validate`는 read_info/상한 검사/next_frame/finish로 전체 디코드를 한 번 수행한다. finish의 끝 chunk 검증은 중복으로 삭제하지 않는다. 게시의 write_all/sync_all/기한 검사/hard_link는 PNG 재디코드가 아니며 SHA는 게시 뒤다. 4초는 게시 deadline이고 전체 응답 시간을 4초 이하로 보장한다는 계약은 아니다.

r3는 고정 `--screenshot-png-filter=0` 한 줄을 추가했다. 공식 옵션의 필터 없음이며 PNG 원픽셀·해상도·전체 디코드 검증·4초 deadline을 유지했다. r3 검사와 실제 결과는 위 통과 항목을 따른다. 필터만의 비용 감소량은 미측정이다. r1/r2 전체 actual 실패를 후속 성공으로 소급하지 않는다.

`native-options-cli-r2/source-repair`는 network_plan.rs의 고정 `--screenshot-png-compression=1`과 capture.rs의 publication 오류 stage 시간만 수리했다. 원1920x1080/PNG 픽셀 검증·4초 deadline·전체 actual assertion은 유지했다. PNG는 무손실이며 압축 정도의 변경은 해상도나 픽셀 수를 낮추는 수리가 아니다.

[공식 mpv screenshot 옵션](https://mpv.io/manual/stable/#taking-screenshots)은 기본 compression7 및 압축 정도가 쓰기 시간·파일 크기에 영향을 준다고 설명한다. 로컬 r1에서 기본값을 조회하거나 정밀 stage를 측정하지 않았으므로7이 단독 원인이라는 확정은 보류한다. 수리 후 failure details는 screenshot/read+verify/publication/total의 단조 소요 시간을 기록한다. r2 fmt0·strict26269ms0 뒤 전체 tests는 통과했고 실제 r2는 위와 같이 실패했다.

[옵션 계획](../../../plans/player/native-options.md)과 [native application 계약](../../player/application/native-episode-playback.md)의 전체 범위를 유지한다. rate/tracks·기존 frame/control/lifetime의 강한 actual 재실행 뒤 결과를 확정하며 실제 회차 전환·GUI 이관·제작·AI 등 전체 goal은 계속 남는다.
