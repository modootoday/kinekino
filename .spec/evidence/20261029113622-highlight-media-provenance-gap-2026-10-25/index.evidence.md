---
id: evidence:kinekino/20261029113622
title: 실제 하이라이트와 미디어 구간 provenance 공백 — 2026-10-25
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 하이라이트와 미디어 구간 provenance 공백 — 2026-10-25

결론: 조회한 기존 인덱스/정본에서 **실제 AI 하이라이트 출력 + 같은 원미디어 identity + 검증된 영상 PTS + 현재 공통 SDK 구간 재생 연결**을 함께 충족한 표본은 확인하지 못했다. actual9 range9null은 유지한다. 실제 출력과 로컬 미디어가 연결되는 더 좁은 후보 및 개발자 제작 클립은 아래처럼 존재하지만 인수표7 충족으로 승격하지 않는다.

이번 조사는 AI 정본 인덱스, 텍스트/독립 애니메이션/제작 정본과 그 문서가 지정한 작은 manifest·snapshot index·원 요청만 읽었다. 새 추론/다운로드/미디어 실행/FFprobe/재귀 미디어 스캔/대형 asset 해시·복사/Cargo/GUI/사용자 판정 쓰기0. 작은 JSON manifest3개만 pin을 확인했다. 미디어 hash는 기존 저장값이며 현재 asset 내용을 재검증하지 않았다.

## 기존 후보를 혼동하지 않는 분류

### 1. Sprite Fright: 실제 ASR/음악 출력 ↔ 같은 PCM 클립은 연결됨

근거 [입력](animation-independent-input.md), [실제 ASR/음악](animation-asr-music.md). `runtime/ai-spike/results/sprite-asr-music-summary-v1.json` 현재 작은 manifest SHA `56C9C031F672BE2116A43196B66514DDC4F561B8746D95215CA6AEA1E8D80BD2`.

- 원 `runtime/ai-spike/source/sprite-fright-sprite-source-r1/source.mp4`:102392846B, 기존 SHA `7cfc8b8c6eeb19685048a289aca208a01356f5082b470fb9f4d933fe4b2087ef`.
- 원 english.vtt:5202B, 기존 SHA `c0bbee8acc29db5cbd5bbe2975bec81fe186ead6a4ec30d4cd3c718b1d57eb9c`.
- 기존 제작사 선언 CC-BY-1.0, stream API licence Unknown; `(CC) Blender Foundation | studio.blender.org`. 이 기록은 현재 재배포 권리 심사/허가가 아니다.
- latin-17 PCM: `runtime/ai-spike/assets/sprite-clips-r1/latin-17.wav`,272044B,SHA `d12ec627660e5b3de88579296d45b6f3cf368c7fe6e2419ea3acbeb643c33270`; start_sample272000/sample_count136000/16kHz, 디코딩 PCM상17–25.5초.
- 같은 summary 행의 실제 ASR label `qwen-onnx-sprite-latin-int4-r1`, 원 stdout SHA `01ce881f5b67d81710636c501ca417a3eb3a7aa430780465ec8990e986c9c0cb`, measurement SHA `185fdf660b942f2fe6d5aabdeceb2140053ccc578d8da35420310e8558b1ffb2`가 바로 이 audio SHA를 참조한다. 실제 출력 텍스트/오류는 보존돼 있고 quality_approved=false다.
- 같은 행의 YAMNet stdout SHA `0379b277f0b8554ab4094226e1ae6e7f184dd2ae5b8c0a49f02a9885609119d4`도 존재한다. 이것은 음악 frame score이며 AI 하이라이트 생성 결과가 아니다.

따라서 같은-source 실제 모델 출력 provenance가 있는 ASR 청취 후보는 이미 있지만 실제 하이라이트 표본은 아니다. 영상 PTS origin·오디오 priming/offset·PCM sample→영상 frame PTS의 검증은 없다. VTT 경계와 PCM 초를 그대로 영상 시간으로 선언하지 않는다. 최소 후속은 기존 `runtime/development/sprite-pts-sample-plan-r1/PLAN.md`의 단일latin 표본 bounded metadata/PTS 관측 및 기존 SDK seek/clock 연결이다. 중앙 별도 media 자원 배정이 필요하며, 성공해도 ASR/기술 표본일 뿐 하이라이트 인수는 남는다.

### 2. Sintel 개발자 제작 클립: 실제 영상/자막 변환 근거는 있지만 AI 선택이 아님

[제작 정본](production.md), [수동 출력](manual-export.md), `.spec/artifacts/spikes/2026-10-18/README.md`의 기존 출처 선언은 Sintel/Blender Foundation CC-BY-3.0이다. 번역 자막은 원음 ASR gold가 아니며 별도 권리/최종 배포 검토를 대신하지 않는다.

- `.spec/evidence/ai/model-spikes-2026-10-18/results/production-export-76968/manifest.json` SHA `468BDE4B3B287A702B42F5926910929E8F7A0CF61605705A020D4342BACB32E8`.
- 저장된 원 영상 SHA `f12c070e295b38cfc94ebd61ac3357c3bac82d1015985f1e8da93a4c6496c46d`, 원 자막 SHA `241f05979875655eaadf7ed98f5ae98cbb712845d33c80bc2fc540b349a61391`.
- 정확 파생 asset: `.spec/artifacts/spikes/2026-10-18/results/production-export-76968/portrait.mp4`,932830B,SHA `0fd9b2de74774b4d2dcea9f9b66466899b44aff1300b121d548e10af4c386483`. 원 producer path는 `runtime/ai-spike/results/production-export-76968/portrait.mp4`로 기존 snapshot index에 기록된다.
- developer 선택 source107.250–119.250초→출력0–12초/24fps·288프레임/H264 AAC, 전체 decode 성공의 기존 관측. cue1–3 원점 이동과 끝 cue 자름이 기록됐고 subtitle_readability_accepted=false다. 별도 자막 왕복 gate의26 cue PTS/duration/payload 일치는 자막 stream 변환의 증거이지 AI 구간 선택 또는 원영상 전체 timestamp origin의 증거가 아니다.

이 클립은 다운로드·추론 없이 기술 재생 fixture로 재사용할 후보이지만 개발자 선택을 모델 하이라이트 출력으로 계상하면 안 된다. 원 SDK playback source identity/seek·관측 및 해당 실제 AI 출력 provenance가 자동으로 생기는 것도 아니다. 기존 prototype 출력과 현재 정상 제품 재생 연결은 별도다.

### 3. 실제 텍스트 highlights 출력은 존재하나 현재 패키지 range는 null

[실제9 package](text-review-sdk-actual.md), [내용 감사](text-review-content-audit.md). package SHA `659b3cb6757259d8e770f6d3c96c4a77dafbab43c6ea540bcc7e0c58e4eb0197`의 모든9 range=null. Qwen 실제 실행 manifest `.spec/evidence/ai/model-spikes-2026-10-18/results/text-structured-qwen06-smoke-r1/manifest.json` SHA `8A31FE724074EBF0A84EA2266C98F4932D47B14229AEFF9F29F2CEFF59B8F844`는 기존 `request-highlights-balanced.json` SHA `0045fb997ea058536303df337d8e644d99f0dcabf69cbaedfaf94e61ebd461fb` 및 cues-ko SHA `dad7609aede6c4dd4eba6a0e868663c10b975f5e2dbb7c21151876caef482b23`를 고정한다. prompt의107.25초 등 cue 시각은 원 요청 선언값이다.

Sintel 기반 자막·텍스트 출력의 출처 근거가 있다는 사실만으로 위 수동 portrait 클립이나 Sprite를 actual9에 붙일 수 없다. 원 출력의 cue IDs/구조 오류·원 source hash·원 자막 참조·영상 timebase/PTS를 모두 대조하고 원 SDK 유스케이스에 연결한 실행 근거는 이번 범위에서 발견되지 않았다. 기존 원 package를 수정하지 않는다.

## 최소 후속 작업과 인수 기준

새 추론 없이 가능한 준비 범위는 기존 실제 highlights raw의 정확 model/output/request/cue identity를 보존한 채, 같은 원영상/원자막 identity와 timestamp transform을 읽기 계약으로 대조하는 것이다. 의미 없는 새 시간을 작성하거나 구조 실패를 자동 복구하지 않는다. 검증된 원 cue→미디어 PTS 연결이 실제로 확보된 경우에만, 원 package와 분리한 명시적 파생 근거/SDK 입력 계약을 검토한다. developer clip을 새 모델 출력으로 만들지 않는다.

실행은 중앙의 별도 제한된 media/SDK 자원 gate에서 원 content identity와 bounded PTS 관측→기존 Rust SDK source/seek/clock/error/cancel/close→정상 GUI 원본/후보 및 필요한 구간 표시 순으로 인수해야 한다. 현재 실제 AI 하이라이트 구간을 재생하고 사용자가 판정할 표본은 **미완료**다. 다음 입력/구간/권리 검토가 끝나기 전 신규 추론/다운로드/미디어 실행을 자동 재개하지 않는다.

## 정상 빌드 placeholder와 실제 인수는 별개

| 단계 | 필요한 증거 | 대신할 수 없는 것 |
|---|---|---|
| 정상 바이너리 준비 | SDK 고정 player 절대경로·SHA·source/producer 및 필수 asset custody | placeholder 문자열 또는 과거 libtest |
| 실행 자원/권한 | PM의 해당 정상 GUI/media owner 배정·원 종료/복구 책임 | 파일 존재·명령 문서·Ready snapshot |
| 원자료 권리 | 기존 선언의 범위·attribution·미디어/자막/모델 조건·최종 사용 권한 검토 | SHA 동일성·공개 URL·stream Unknown의 임의 승격 |
| 시간 정합성 | 같은 content·원 stream/timebase/PTS·PCM/자막 변환·SDK 실제 seek 관측 | 요청 cue 시각·decoded PCM초·다른작품/수동 클립 |
| 사용자 품질 | 실제 GUI의 해당 원본/후보/필요 구간을 본 사용자 명시 verdict/이력 | headless19·실제9CLI·단순재생/변환 성공 |

위 단계는 각각 독립이며 하나를 완료해 나머지를 완료로 표시하지 않는다. 사용자9 proposal/decisions namespace에 개발자 기록0을 유지했다. 적용 스킬은 기존 읽은 modootoday/software-qa/code-change-review-verification+verdict-table 및 test-fixture-and-mock-build: 상속된 원 근거/새 관측/미확인을 구별하고 실제/합성 및 개발자 fixture/모델 출력의 경계를 유지한다.
