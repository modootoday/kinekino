---
id: sot:kinekino/encoder-inventory
title: 숏폼 encoder inventory와 준비 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 숏폼 encoder inventory와 준비 계약

R150 검증완료범위: [실제inventory1/공통정책4/parser1·기본strict/fmt/no-run·지정190/정상bins·2411사후custody](../../../verification/application/clip-encoder-admission-20261028.md). 설치FFmpeg8.1.1의원6단계와필요기능·재열기SHA·원toolguard를확인했다. R149첫2자리filter표실패도보존했다. 실제encode/재생/발행/Jobs/품질은아직아니다.

2026-10-28 R149 중앙 PM. source admission에이어encoder의실제가용성을확인한다. 이계층은실제인코딩·재생호환·출력발행승인이아니다.

Rust조립의FixedClipEncoder는절대경로/SHA·최대30초inventory deadline을고정한다. 기존tool guard와owned-process runner를재사용한다. 원파일/조상guard를PinnedClipEncoder수명에유지하고version/buildconf/license/encoders/filters/muxers의6단계를같은deadline/control로직렬관측한다. 각stdout최대512KiB,기존stderr상한/물리종료/receipt정책을유지한다. rawstdout/stderr를단계별EncoderToolObservation에보존하며PATH탐색·다운로드·실패pin fallback은없다.

inventory SHA는{contract:"clip-encoder-inventory/1",tool_sha256,observations}의고정Rust field순서JSON bytes SHA다. build_configuration_sha256/license_sha256는각단계의stage/stdout/stderr record bytes SHA다. 목록의flag열과name/alias를구분하고legend/설명문을기능으로인정하지않는다. version명시누락·빈build/license관측·빈/잘못된table은NeedsInput이다. 라이선스원문보존은재배포허가나제품법률승인이아니며배포는목표범위밖이다.

공통application encoder_requirements는유효ClipDocument에서요구기능을도출한다. 초기exact reencode프로필은MP4/libx264 H264·yuv420p의짝수치수·선택AAC이며다른container/codec/fast-keyframe경로는명시Unsupported로남긴다. 이를승인요구에서삭제하지않고추가프로필/별도cut검증으로추적한다. 기본trim/setpts/crop/scale/pad/format/fps,선택오디오의atrim/asetpts/volume,선택자막의subtitles를요구한다. 선택자막기능은필수목표로유지하되filter존재를자막/폰트asset검증이나실제합성성공으로대체하지않는다.

prepare_encoder는명시inventory의tool/record/build/license SHA·version과요구encoder/filter/muxer를대조하고누락기능을NeedsInput details에남긴다. 실패/취소시원오류를전파하고임시lease를회수한다. PreparedClipEncoder는non-Serialize/non-Clone이며정확한전체ClipDocument SHA·preset·요구기능과원lease를보유한다. 실제renderer는media발행document와이SHA를재대조하고source/encoder양쪽guard를물리종료까지유지해야한다.

미완료: 실제MP4인코딩·프레임경계/PTS·source색/회전·구간별crop·자막/폰트·소리·출력검증·no-overwrite발행·원Jobs/SDK/CLI등록·재생호환. 현재production.render_short미지원. 기능목록·합성포트통과를그인수로계상하지않는다. 기존WIP/소유권/private overlay·C-only/heavy1·WARP/추론/다운로드/수동GUI/push보류유지.

modoo도메인검색을실행했으나frontend-first/agent replay/학습·생성영상검사결과는이번nativeencoder책임에맞지않아채택하지않았다. 기존에본문을읽은modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을적용해관측/준비/인코딩/품질승인을구분했다.
