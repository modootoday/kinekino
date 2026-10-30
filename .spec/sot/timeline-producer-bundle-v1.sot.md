---
id: sot:kinekino/timeline-producer-bundle-v1
title: Timeline producer bundle v1
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Timeline producer bundle v1

2026-10-26. 내부 Rust adapter의 관측 교차검증 계약이다. 공개 SDK/CLI wire나 기존 모델 review package 형식을 바꾸지 않는다. 원 package와 item asset들 중에 명시된 작은 JSON manifest 및 관련 관측·실행 기록을 소비한다. 원 모델 결과/판정은 수정하지 않는다.

## 신뢰 입장

제품 조립 코드가 TrustedTimelineAdmission으로 정확한 case/package/item/source-request SHA, media ID/SHA, manifest ID/SHA, audio/probe/capture engine SHA를 지정해야 한다. 이 값은 Deserialize를 제공하지 않으며 호출자 요청에서 자동 생성·검색하지 않는다. 한 resolver는 한 명시 입장만 처리하고 다른 case/media/reference는 파일 조회 전에 거절한다.

입장은 이미 검토한 producer bundle을 지명하는 신뢰 경계다. 임의 JSON의 SHA를 계산하는 행위는 producer 출처의 인증이 아니다. production 조립에서 실제 producer와 원 execution 근거를 인수하기 전에는 새 bundle을 임의로 지명하지 않는다. 합성 fixture의 지명은 시험 전용이며 실제 영상·AI 품질 성공이 아니다.

## manifest와 원자료

manifest(schema_version="1")는 source_request_sha256, media, mapping_revision 및 다음 참조를 갖는다. 모든 참조는 같은 case item의 명시 asset ID/SHA여야 한다. 세션은 package와 전체 asset bytes/SHA를 검증·보유하고, tool/source 원파일은 metadata inspect에서도 이 보유를 유지한다.

- audio: log/full_wav/clip_wav/replay_pcm/engine/execution과 clip_start_sample.
- video: frames/engine/execution.
- origin_capture/start_capture/end_capture: 각각 capture 기록 asset.

manifest·실행/capture JSON은64KiB, raw log/frame JSON은12MiB, PCM/PNG는64MiB 한도다. DTO는 unknown/duplicate 필드를 거절한다. 파일 경로는 입력받지 않으며 embedded source request 경로도 해석하지 않는다.

audio execution은 source/engine/log/replay SHA·필수 completion=observer_accepted와 copy_timestamps=true, mono16000/s16/time_base1/16000/async_correction=false를 요구한다. raw log의 정수 frame 연속성과 합계가 full PCM sample 수와 일치해야 하며 full/replay·clip sample slice를 exact byte로 비교한다. 원 wrapper receipt에 없는 숫자 exit_code를 추가하지 않는다.

audio/video execution에는 원 command/inputs/process asset 참조와 명시 ExecutionContract enum을 추가로 요구한다. 계약에는 Default가 없으며 누락된 contract나 video interval을 채우지 않는다. audio는 AudioCopyTsMono16kS16FirstStream, video는 VideoFirstStreamFrameWindow{interval}만 해당 역할에서 허용한다. 원 명령의 전체 argv를 선택된 계약과 대조하고 source/tool/원PCM SHA 및 원 경로 문자열을 원 INPUT 기록과 연결한다. 경로는 문자열 대조에만 쓰고 실행·파일 탐색하지 않는다. process는 원 observer의 result=Ok(())/terminal=true와 PID/elapsed를 요구한다. 이는 원 wrapper의 accepted 결과이며 기록에 없는 numeric exit field를 새 관측으로 발명하지 않는다. 원 observer 소스에서 status.success일 때만 Ok를 반환한 사실과 함께 해석한다.

복사한 기대값으로 원 명령을 대신할 수 없다. 예를 들어 normalized async_correction=false여도 원 -af가 async=1이면 거절한다. 마찬가지로 원 audio stream 선택·video interval·종료 상태와의 불일치도 거절한다. 실제 bundle 발행은 이 trace 대조와 capture의 원 request/response/process 대조가 모두 준비된 뒤 수행한다. 현재 실제 admission은 아직 발행하지 않는다.

video execution은 source/engine/frames SHA·stream index·필수 completion=observer_accepted가 원 관측과 일치해야 한다. capture 기록은 source/engine/PNG 참조, 요청 us/실제 signed PTS/timebase와 원 process에서 읽은 exit_code/complete를 갖는다. PNG는 기존 bounded decoder로 완전히 검증한다. origin 요청은0, stream timebase와 같아야 하며 원점 숫자를 별도로 받지 않는다. start/end capture는 공통 TimelineObservations의 sample→source us 및 첫 frame at-or-after 규칙과 일치해야 한다.

## 반환과 미완료 경계

capture에는 원 INPUT/request/response/process 참조와 필수 CaptureContract::SdkFirstSourceFrameAtOrAfter를 추가로 요구한다. TrustedTimelineAdmission은 capture producer CLI SHA도 명시해야 한다. v1은 capture의 probe가 video probe pin과 같음을 요구한다. 원 INPUT의 source/tool/probe/CLI SHA, 요청 source/output/시각, 응답 request ID·tick/timebase/us·PNG SHA/bytes/dimensions와 실제 PNG, process의 동일 CLI SHA/FileID·exit0을 모두 대조한다. 경로는 문자열 대조만 하며 열거나 실행하지 않는다.

관측 구조는 필요한 authority 필드에 Default를 제공하지 않는다. legacy CapturedFrame의 선택적 output_sha256이나 unavailable capability의 Default를 이 검증에 재사용하지 않고, 원 capture output SHA와 명시 timeout 등 필요한 값이 없는 기록은 거절한다. 실제 producer CLI의 역사 SHA는 원 기록의 producer 식별이며 현재 CLI 파일이 그 SHA라는 주장이 아니다. INPUT의 file identity와 파생 PNG 일치도 원 source-content SHA와 구별한다. original trace 검증 후에도 실제 묶음 발행/입장과 SDK playback 검증은 별도로 수행한다.

ReviewTimelineEvidence는 TimelineEvidenceResolver를 구현하고, 반환 lease가 원 RetainedReviewEvidence 세션과 계산한 관측을 함께 보유한다. 전체 application admission이 case/media/evidence lease를 추가로 유지하고 snapshot 일치를 재확인한다.

현재 playback_ready=false 및 wire TimelineStatus::Unverified를 유지한다. producer bundle을 실제 관측에 맞게 발행·입장시키는 production 조립, 실제 SDK playback owner/clock·Verified 저장전이·actual9 연결·GUI/사용자 품질은 별도 미완료다. 기존 역사 plain-text receipts를 이 JSON 계약에 맞는 실제 producer 실행으로 자동 변환하거나 소급 인수하지 않는다.
