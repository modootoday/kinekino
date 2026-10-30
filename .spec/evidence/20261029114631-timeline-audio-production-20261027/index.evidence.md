---
id: evidence:kinekino/20261029114631
title: 영상·오디오 공통 producer 수명과 실제 clock 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 영상·오디오 공통 producer 수명과 실제 clock 관측

2026-10-27 중앙 PM. 기존 영상 실행 수명을 `production/process.rs`로 분리하고 영상·오디오가 같은 원 Job/child·deadline·permit·pending cleanup을 사용하도록 연결했다. 명시적 ffprobe/ffmpeg 도구 wrapper가 파일명·SHA·원 파일 guard를 유지한다. 전체 issuer·PCM 계보·GUI/AI 승인은 아니다.

## 고정 입력과 실행

SOURCE `CAFFCBBA32E67C5AA683B45053F8E2C30DBEB4AC5CB86EC8F15700C7C048CD84`, inventory `9498EF5389AF4095EE01E6CC38F0FAE6D964424842F400C728E44D1A203804CB`,2206파일. `runtime/verification/source375-timeline-audio-input-r1` 소유. 기존 C27 입력/190/정상bins는 별도 checkpoint로 보존한다.

- 446 최초 unit은18통과/5ignored. 신규 unused import2와 기존 RoutePause 경고를 구별하고 신규 import만 제거했다. 최초 로그와 preimage는 보존했다.
- 447 strict(native-proposal-window/all-targets/-D warnings) 통과. 기존 blitz-paint dependency deprecation 경고 소유권은 변경하지 않았다.
- 449 실제 시험 no-run 통과.
- 451 최종 정책18통과/5ignored,452 fmt 통과,453 verify-current exit0/41453ms·2206파일 content/identity 전후 동일·stage 없음.
- 450 실제1 통과/6.86초. 동일 source의 영상240프레임 PTS196608~318976/time_base1/12288, stdout SHA `BA26BD11B6BEE0FEF8FCB5A461F4B9601BEEE05ABCF26E275BB935D2DF954A34`를 유지했다.
- 오디오 원 decode는27128 ashowninfo 프레임·mono16k/s16 연속clock·firstPTS0/end_sample10078204, 원 childPID35948/creation134375268715395893/exit0/비강제. stderr SHA `DBE71E2E786A6160B5CB881D6B589A8D86EFBBF2DB440D9CA3D36B807D41D097`.
- 영상/오디오 각각 원 child 생성 뒤 취소를 실행했다. 두 최초 오류는 Cancelled, child exit124/forced=true/Job empty/cleanup_error=null, observation_published=false다. 강제 취소를 child 정상 성공으로 계상하지 않는다. 슬롯 독점과 종료 후 재예약·ABANDONED 부재도 확인했다.

도구는 기존 FFprobe SHA A6618E99…와 기존 추출 FFmpeg6.1 SHA `71414777922D05F2639A5CF4A0F9A405F169809CF459000A6F976C9ADD61F060`을 사용했다. 새 설치·PATH변경·모델 실행·다운로드·수동 GUI 없음. 원본 Sprite의 case/media 및 nomination guard를 유지하며 실제9 Sintel과 결합하지 않았다.

원 증거는 `runtime/verification/timeline-audio-production-actual-datalab-r1/RETURN.json`, `CANCEL.json`, `AUDIO-CANCEL.json`, 각 component의 INPUT/STARTED/PROCESS/RETURN 및 `source375-datalab-b0-r1-logs/446`~`453`이다. test 이름은 이전 video 명칭을 유지하지만450에서는 영상과 오디오 양쪽을 실행했다.

## 미완료 범위

오디오 clock은 source를 copyts로 디코드한 첫 오디오 스트림의 16k/s16/mono 관측이다. 출력은 null이며 PCM 원본·재생·클립 파일과 sample별 일치의 근거가 아니다. 영상 time_base를 audio sample clock과 혼합하지 않는다. 전체 PCM 계보·decoded origin/끝점 capture·결합 opaque proof/custody 등록·제품 admission/startup은 미완료다. 새 CAFF 입력의 전체190/정상bins도 미실행이다. 기존 C27 성공을 승계하지 않는다.

modoo.today `modootoday-free/ai-quality-plus/agent-run-postcondition-and-replay-check` 본문 SHA61843099b7d54c3d7ac226218a5ee1b6ccdbe36ef84268762dbc33a68fbb58b5 및 worked-example SHA80c9fe8a8a79dd8e08d84b36483d7742f273e55fb2769d2e6156a1111fdd5c11의 원 실행/후조건/주장 분리를 적용했다. 정책 시험·실제 process·품질 승인을 구별한다. 소유권·WIP·보류·원 실패는 유지한다.
