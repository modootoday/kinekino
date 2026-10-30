---
id: evidence:kinekino/20261029113727
title: 독립 애니메이션 입력과 자막 참조 계약
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 독립 애니메이션 입력과 자막 참조 계약

2026-10-18. 기존 Sintel 및 FLEURS와 다른 작품인 Sprite Fright를 제작사 원본 영상과 영문 자막으로 확보했다. 이 문서는 입력 준비를 담당한다. 후속 [ASR4회·음악4회 관측](animation-asr-music.md)은 별도 기록이며 품질 승인 결과가 아니다. 일본어 애니메이션 검증은 별도로 필요하다.

## 출처와 권리 선언

- 공식 API: `https://video.blender.org/api/v1/videos/a69d68a5-a0e0-4a80-9d66-49f093c97aaf`
- [제작사 설명](https://studio.blender.org/projects/sprite-fright/pages/about/)은 CC BY 1.0을 선언한다. 스트림 API licence는 Unknown이다. CC BY 4.0으로 바꾸어 표기하지 않는다. 상표·별도 표시 자료의 예외는 유지한다.
- 크레딧: `(CC) Blender Foundation | studio.blender.org`.
- 원본 영상 102392846바이트, SHA256 `7cfc8b8c6eeb19685048a289aca208a01356f5082b470fb9f4d933fe4b2087ef`. upstream digest는 없으며 로컬 수집 파일의 동일성만 증명한다.
- 영문 VTT는 automaticallyGenerated=false, SHA256 `c0bbee8acc29db5cbd5bbe2975bec81fe186ead6a4ec30d4cd3c718b1d57eb9c`다. 비자동 메타데이터는 사람이 음성을 대조한 ASR 정답을 의미하지 않는다.

## 고정 클립 계획

전체 오디오를 mono PCM16 16kHz로 한 번 디코딩한 뒤 Rust에서 샘플 범위를 추출한다. 별도 seek/resampling을 하지 않는다. 시간은 디코딩 PCM 기준이며 인간 word timestamp 정답이 아니다.

| ID | 시작 샘플 | 샘플 수 | 초 구간 | 목적 |
|---|---:|---:|---|---|
| latin-17 | 272000 | 136000 | 17–25.5 | 영어와 라틴어 고유명사 |
| dialogue-28 | 448000 | 260000 | 28–44.25 | 연속 대화와 혼합 원음 |
| tea-205500 | 3288000 | 232000 | 205.5–220 | 다른 장면의 대화 |
| caption-gap-0 | 0 | 192000 | 0–12 | 자막 없는 구간 출력 관찰 |

자막 경계를 가르거나 예상 자막 범위가 바뀌면 중단한다. 자막 부재는 검증된 무음·비발화 라벨이 아니다. 자막 비교는 proxy CER로 구분하고 음성 정확도·환각은 청취 및 인간 라벨로 확인한다. 음악 점수도 OP/ED 확률이 아니다.

## 수집 증거와 다음 게이트

`runtime/ai-spike/acquisitions/sprite-source-r1`에 요청·사전 관측·결과를 보존했다. CPU 16.13%, 여유 RAM 37398716416바이트에서 수집했으며 기존 앱은 종료하지 않았다. ready=false는 실행 중 player가 있는 원래 preflight 상태다. 별도 자원 admission 통과를 성능 비교용 격리로 해석하지 않는다.

manifest는 `runtime/ai-spike/results/sprite-fright-source-v1.json`이다. 수집 EXE `7d9d0c7dd73743a6b661634dd818c9c5aa6e6fee88f60d2455834a92fbd10b0d`와 컴파일 입력91파일을 실행 전에 동결했다. bundle 총92개는 EXE1개를 포함한 수다. 영상·자막·API·라이선스 원문 파일 해시를 기록했다.

## 실제 PCM 추출

`sprite-clips-r1` 실행이 완료됐다. FFmpeg 실행은 infrastructure 어댑터로 분리했다. EXE `32d974e47414f4dc20b6c47e0b5bc7624304d21a024131a193806e5b26a588fb`와 컴파일 입력93파일을 선보존했다. bundle 총94개는 EXE1개를 포함한다. fmt/test36/clippy 검사를 통과했다.

전체 PCM은 10078204샘플, SHA256 `872987bc1c6296b33241e833f5a315e8bd656b2d3ed05d14034231cad4ae51a4`다. FFmpeg 실제 바이너리 SHA256은 `71414777922d05f2639a5cf4a0f9a405f169809cf459000a6f976c9add61f060`이며 버전 원문과 stderr를 저장했다. admission은 CPU11.77%, 여유RAM37407629312바이트였다.

| 클립 | WAV SHA256 |
|---|---|
| latin-17 | d12ec627660e5b3de88579296d45b6f3cf368c7fe6e2419ea3acbeb643c33270 |
| dialogue-28 | 68be4cc605d179cf61ab0f31e707ac87ae87352e6b76746eb4c16f337de14bdb |
| tea-205500 | 9fec411a24d466a5ab5cec3f226b6b93895f5dcce1e5aa57342109fddbc46ef4 |
| caption-gap-0 | 01bd542dffe88388b54532e0a9e9809529859b28de83328825b2d13b6a0fa8bd |

manifest는 `runtime/ai-spike/results/sprite-clips-r1.json`이며 참조 자막·샘플 좌표·원본 manifest 해시를 포함한다. 후속 worker 계약과 proxy 평가기 및 실제 관측은 [결과 정본](animation-asr-music.md)에 있다. 사람 검토 등 [잔여 품질 게이트](../../../plans/player/ai-quality-gates.md)는 유지한다.
