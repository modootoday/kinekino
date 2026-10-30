---
id: evidence:kinekino/20261029114645
title: 실제 decoder audio 관측 custody adapter
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 decoder audio 관측 custody adapter

2026-10-26 중앙 PM. infrastructure/timeline_evidence의 RetainedAudioClock은 신뢰된 local assembly가 지정한 절대 경로와 SHA를 받아 기존 ordinary-root/file guard를 재사용한다. 원 파일과 모든 ancestor guard를 반환 객체가 보유하며 새 파일이나 registry를 만들지 않는다. 사용자 CLI/GUI에 임의 경로·Verified 입력을 추가하지 않는다.

최대12MiB UTF-8 로그, frame50000개, 관련 행4096bytes를 제한한다. FFmpeg Parsed_ashowinfo의 n/pts/nb_samples/fmt/channels/rate를 중복 없이 읽고 mono16k s16·순차 n·정수 PTS 연속성을 요구한다. pts_time은 계산에 쓰지 않는다. 음수 PTS를 유지하고 누락·잘못된 숫자·불연속을 거절한다.

이는 raw observation 내용 고정과 clock 해석이다. 원미디어/decoder 실행 provenance·PCM byte 계보·video capture·ReviewCase 결합을 단독으로 인증하지 않는다. 전체 TimelineEvidenceResolver 구현과 Verified 전이는 계속 미완료다. 해당 권한을 예상 SHA 문자열만으로 부여하지 않는다.

182는 합성 parser 및 기존 admission 시험을 실행 중이다. 후속 actual 시험은 새 decoder/추론 없이 보존된 sprite-audio-legacy-replay-datalab-r1/decode.stderr(5003403bytes,SHA EC1A5907637C54A4DA4A5184FBB9F72B7F9E1D1DACCA229D1B1D3A17A755345C)를 명시적으로 읽는다.27128 frame/10078204 samples 원 clock과 비교하고 custody 중 writer 거절·잘못된 SHA 거절을 확인할 예정이다. 실행 전 성공으로 계상하지 않는다.

## 최종 실행 결과

- 182 종료0: 기존 admission4+새 parser3 =7pass, 명시 actual1ignored. 합성 행은 새 실제 media clock으로 세지 않는다.
- 183 종료0: ignored actual을 명시 exact 실행해1pass/0.65초. 원27128frame·첫PTS0·10078204samples, 동일SHA, 보유 중 write-open 거절과 잘못된SHA 거절을 확인했다. decoder/추론/다운로드/GUI 실행0이다.
- fmt --all -- --check 종료0,184 all-targets strict 종료0(19.27초). 기존 blitz-paint 의존 deprecation은 원로그에 보존했다.
- 원로그 시험 후 SHA EC1A5907… 동일. audio.rs SHA FA57FEB7E7B9B14167B5D5FF9B36E7F7419C52E360C4CA9A3D3B4D83FB3E827E, tests.rs DC8A0DE660CC0CCC0E79B8C83ADBE408BAA3661EF44ADEDDF70A5D25ED6B851D. fmt 이후 시험/strict 동안 코드 변경 없음.

원 로그의 고정과 정수 clock 파싱 단계는 완료했다. video 원관측·PCM byte 계보·source/tool/process provenance의 교차 결합, ReviewCase/media/reference exact binding을 갖는 실제 TimelineEvidenceResolver 및 SDK owner/clock 연결은 미완료다. raw parser 통과를 제품 Verified로 승격하지 않는다.

이전39603654후계190/정상bins는 이 새코드의 통합 결과가 아니다. 새코드 영향/strict 및 실제 읽기 결과를 별도 기록한다. 원WIP/소유권/원로그·heavy1·C캐시 및 추론/다운로드/수동GUI/WARP/push보류를 유지한다.

기존 적용 modoo.today test-fixture-and-mock-build와 code-change-review-verification의 raw/합성·주장범위 분리 적용. 기존 private clock 관측기의 동작 근거를 재사용하되 새 제품 parser에는 중복/누락/숫자/자원 경계 거절을 명시했다. [원 PCM 관측](../ai/sprite-audio-clock-replay-datalab-20261026.md)과 [admission 계약](timeline-evidence-admission-20261026.md)에 연결한다.
