---
id: evidence:kinekino/20261029114316
title: 원본 PCM producer 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원본 PCM producer 관측

2026-10-27 중앙 PM. 기존 source·tool·slot·원 Job/deadline/pending 수명을 유지한 채 FFmpeg의 raw mono16k/s16le stdout을 보유한다. `ProducedAudioPcm`은 같은 실행의 RetainedAudioClock과 원 RawFile의 SHA/바이트 수를 검증한다. WAV 헤더를 합성하거나 기록을 Verified 권위로 승격하지 않는다.

## 입력과 실제 결과

SOURCE `C06BD8DFC6C9F97779D01087CF444ABB035AE10C8D9288829C0DBDF664A109DF`/2207파일, inventory `6DC1413A5A26192731461D32B30A875090392E80BDF9D874BF05C8AD14241191`, `runtime/verification/source375-timeline-pcm-input-r1` 소유. 이전 CAFF와 C27은 별도 epoch다.

| 로그 | 결과 |
|---|---|
|454|native-proposal-window/all-targets strict 통과; 기존 dependency warning 소유권 유지|
|455|정책19통과/5ignored. PCM 빈/홀수/잘린 바이트·clock sample 수 불일치 거절, 음수PTS 보존 포함|
|457|실제 시험 no-run 통과|
|458|실제1/8.36초 통과: 영상240프레임·audio clock27128프레임 회귀와 원본 PCM 관측/취소|
|459|fmt --check 통과|
|460|verify-current exit0/39368ms·전후2207파일 content/identity 동일·stage 없음|

원본 PCM은 10078204 samples/20156408 bytes, firstPTS0/endPTS10078204, SHA256 `74F30535A9EF581F3A048917941DE6F41DAD1C40101149C1EBE84352A1300D8B`. 같은 실행 clock의 samples 합/연속 PTS 범위와 정확히 일치하며, 별도 clock-only 실행의 frame 배열과도 일치했다. 원 PCM childPID34372/creation134375273241918658/exit0/비강제다.

PCM child 생성 후 취소는 원 Cancelled·PID28764/creation134375273263139449/exit124/forced=true/Job empty/cleanup_error=null을 보존했고 관측을 발행하지 않았다. 영상·clock 취소 및 슬롯 재예약도 회귀했다. 취소를 정상 child 성공으로 계상하지 않는다.

원 증거는 `runtime/verification/timeline-pcm-production-actual-datalab-r1/RETURN.json`, PCM-CANCEL.json, component별 INPUT/STARTED/PROCESS/RETURN과 `source375-datalab-b0-r1-logs/454`~`460`이다. 출력은 private verification 증거이며 공개 자산으로 추가하지 않았다. preimage는 timeline-pcm-production-preimage-r1에 보존했다.

## 경계와 다음 작업

원본 PCM 보유만 검증했으며 재생 입력 decode·정확한 clip 계보·origin/capture·결합 proof/custody·제품 admission/startup은 미완료다. 현재 자원 설정의 raw 출력 상한은1~64MiB, clock parser의 원 한도는12MiB다. polling/최종 확인 상한이며 디스크 hard quota가 아니다. source/도구/기존 C 캐시 정책·중앙 무거운 작업1개·locked/offline/jobs1을 유지했다. 실제9 Sintel의 AI 품질이나 GUI 인수에 연결하지 않는다. 최신 전체190/정상bins는 변경 전 C27 checkpoint이며 현재 C06에서는 미실행이다.

modoo.today `modootoday-free/ai-quality-plus/agent-run-postcondition-and-replay-check` 본문 SHA61843099b7d54c3d7ac226218a5ee1b6ccdbe36ef84268762dbc33a68fbb58b5 및 worked-example SHA80c9fe8a8a79dd8e08d84b36483d7742f273e55fb2769d2e6156a1111fdd5c11의 원 실행·후조건·주장 분리를 적용했다. 원본 증거·소유권·WIP·보류를 유지하며 새 설치·모델 실행·다운로드·수동GUI·WARP변경·push는 없다.
