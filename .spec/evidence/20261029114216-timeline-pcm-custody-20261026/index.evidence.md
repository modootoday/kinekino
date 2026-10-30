---
id: evidence:kinekino/20261029114216
title: PCM byte 계보 제품 adapter
status: active
created: 2026-10-29
updated: 2026-10-29
---
# PCM byte 계보 제품 adapter

2026-10-26 중앙 PM. RetainedPcmLineage는 trusted assembly의 기존 full WAV/clip WAV/replay s16le path+SHA와 명시 clip start sample을 받아 세 파일/ancestor를 보유한다. wire 입력·임의 registry·모델/decoder 실행을 추가하지 않는다.

WAV는 기존 media_vad::application::wav_pcm의 strict mono16k PCM16 RIFF parser를 재사용한다. 원 WAV data와 replay 전체 bytes, 명시 시작 sample×2에서의 원 slice와 clip data 전체 bytes가 일치해야 한다. 차이를 허용하거나 offset을 탐색/보정하지 않는다. 음성 품질이나 source media/decoder/process provenance를 이 byte검사로 대신하지 않는다.

공통 raw 보유는 기존 text관측12MiB를 유지하며 PCM만64MiB 한도로 연다. read_bytes와 read UTF-8 책임을 분리했고 caller는 파일을 쓰지 않는다. 결과는 full/clip 샘플 수·clip start·각PCM data SHA다. 반환 객체가 세 원 file guard를 유지한다.

190은 기존 admission/audio/video 및 새PCM합성 영향시험 진행중이다. 후속실제파일은 기존 full SHA872987BC…/clipD12EC627…/재현raw74F30535…만 읽는다. 실제 sample10078204/start272000/count136000 및 보유중writer거절을 확인할 예정이다. 새decode/추론/다운로드/수동GUI0,실행전통과계상0이다.

## 확정 결과

190은 영향9pass/actual3ignored로 종료0. 실제 replay pin을 동일 SHA의 lowercase 리터럴로 정리한 후191에서 include-ignored를 명시 실행했고 영향9+actual3=12pass/0fail(시험1.15초)로 종료0했다. 실제 audio/video 회귀와 함께PCM full10078204/clip start272000/count136000·정확byte일치 및 세파일보유중write-open거절을 확인했다.원full/clip/replay SHA는각872987BC1C6296B33241E833F5A315E8BD656B2D3ED05D14034231CAD4AE51A4 / D12EC627660E5B3DE88579296D45B6F3CF368C7FE6E2419EA3ACBEB643C33270 / 74F30535A9EF581F3A048917941DE6F41DAD1C40101149C1EBE84352A1300D8B로시험후에도동일하다.

fmt --all -- --check 종료0,192 all-targets strict 종료0/26873ms.기존blitz-paint deprecation은의존원로그보존.후계pcm.rs SHA9392EC2F80614BEC2ED57D6EE18BB4E845A290B8ADFC791CAF2B243157265711,raw.rs5BF1AA0AC92DC5DBE44D330C106940509EDBB408D6FB961F68BE6EE1B2161292,pcm/tests.rsF4E8A571ABE5A8F8EE9A9DE722904CD4D4FE8FA02B8D6DEF60FEF662F1E92C1B.191/strict동안소스변경없음.

PCM 계보 단계는 완료했다.다음은개별관측을실제source/tool/process와기존case/media/reference custody에결합하는resolver다.원자료의숫자·hash만있는caller요약을권한으로쓰지않는다.이Sprite기술표본은actual9에연결하지않는다.

원 source/tool/process·video/capture·ReviewCase/media/reference를 결합하는 실제 resolver와 SDKclock/Verified·actual9/AI/GUI는 미완료다. 이전396기준190/정상bins를 이 새 후계의 검증으로 승계하지 않는다. 기존 WIP·소유권·원증거·heavy1/C캐시/보류 유지.

기존 적용 modoo.today test-fixture-and-mock-build/code-change-review-verification의 실제/합성·바이트계보/권한 분리를 적용했다. [audio](timeline-audio-custody-20261026.md), [video](timeline-video-custody-20261026.md), [원 PCM 계보](../ai/sprite-pcm-lineage-datalab-20261026.md)와 연결한다.
