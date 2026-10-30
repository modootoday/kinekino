---
id: evidence:kinekino/20261029114512
title: Video frame 원자료 custody adapter
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Video frame 원자료 custody adapter

2026-10-26 중앙 PM. infrastructure/timeline_evidence에 RetainedVideoFrames를 추가하고 원 파일/ancestor 보유·SHA·12MiB 제한·UTF-8 읽기를 raw 책임으로 분리했다. RetainedAudioClock도 같은 raw 보유를 소비한다. 기존 guard의 reparse/일반 파일·디렉터리 제약과 writer 거절을 유지한다.

video 계약은 고정 ffprobe 관측 필드 frames{pts,best_effort_timestamp,duration,key_frame}만 허용한다. 중복/누락/float/unknown 필드, 빈 창/50000초과, zero duration, best-effort와PTS 불일치, 연속 frame PTS 불일치를 거절한다. signed PTS는 그대로 보존한다. timebase/source/origin/decoder 실행 권한을 frame JSON에서 추정하지 않는다. unsupported 다른 관측 포맷을 조용히 변환하지 않는다.

185는 기존 admission/audio 및 새 video parser 영향 시험 진행 중이다. 다음 actual은 보존 frames.stdout.json의 SHA BA26BD11B6BEE0FEF8FCB5A461F4B9601BEEE05ABCF26E275BB935D2DF954A34·38211bytes와 원 audio EC1A5907…·5003403bytes를 읽는다. 새 decoder/추론/다운로드/GUI 실행은 없다. 보유 중 write-open/잘못된SHA 거절과 240 frame/first196608/last318976 및 capture tick208896/313344 membership을 확인한다. 완료 전 성공으로 계상하지 않는다.

첫 실행:185 영향8pass/actual2ignored.186 actual은audio1pass/video1fail로보존했다.실제파일에는frames외에빈programs/stream_groups와stream index/time_base/disposition/tags가있어최초strict DTO가거절했다.원frame·SHA를바꾸지않았다.실패video.rs(18F2171AAA7E0F4CE8782D032BC302A06F6AB927250AE6E45DD6589E0803693C)/tests사본은runtime/verification/timeline-video-parser-first-failure-r1에보존했다.

후계는정확히1개stream의정수index/양의유리time_base를명시해반환한다.programs/stream_groups는빈배열만,disposition은0/1·tags는bounded문자열metadata로검사하며원점/권한으로쓰지않는다.누락/중복/ambiguous stream을거절한다.187영향시험진행중이며186실패를소급성공으로바꾸지않는다.

## 최종 후계 결과

187 영향8pass/actual2ignored.188은명시actual2pass(0.69초):audio27128frame/10078204samples재확인,video240frame/first196608/last318976/tick208896·313344 membership,stream index0/timebase1/12288확인.두파일의보유중write-open·wrongSHA거절통과.원video/audio SHA는각각BA26BD11…/EC1A5907…로불변이다.

fmt --all -- --check 종료0,189 all-targets strict 종료0/26030ms.후계코드는시험/strict동안동일했다.raw.rs SHA BA2814B173F3468240F9F2111CC8F376CDA3DA3D9EFC2D24F1CBB12F06E74549,audio.rs7CB13C6CAA35CDC4E2D196A8A5992CAB508A7CBD8D1CA58614ED1FD9159B6455,video.rs8B325309B81102F8C09B2B694CBC4DAF52F8B14051E77BEDED81F93D296D0B9D,video/tests.rsEA2CC609E2AE8270DB980451D3B574C11EEA90144A0E51195C446169C7CDC2BD.원186실패와소스사본유지.

audio/video 원자료의보유·정수해석은완료했다.PCM byte계보와source/tool/process·capture·ReviewCase를결합한실제resolver,SDK재생clock·Verified는미완료다.임의caller facts를실측으로대체하지않는다.

source/tool/process·PCM byte 계보·capture와 ReviewCase/media/reference 결합, 실제 TimelineEvidenceResolver·SDK 재생owner/clock·Verified는 미완료다. 이 후계는 이전396기준190/정상bins 이후 변경이며 해당 성공을 자동 승계하지 않는다. 원WIP·소유권·로그·single heavy/C NVMe 및 보류 유지.

기존 modoo.today test-fixture-and-mock-build/code-change-review-verification의 합성/원관측/권한 분리 적용. [audio 정본](timeline-audio-custody-20261026.md), [원 frame 관측](../ai/sprite-stream-frame-pts-datalab-20261026.md), [admission](timeline-evidence-admission-20261026.md)에 연결한다.
