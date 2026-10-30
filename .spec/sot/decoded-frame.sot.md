---
id: sot:kinekino/decoded-frame
title: 숏폼 decoded-frame 경계 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 숏폼 decoded-frame 경계 계약

R151 검증완료범위: [공통3/parser1/실제Sintel decoded1·strict/fmt/no-run·2415사후custody](../../../verification/application/clip-frame-boundaries-20261028.md). 실제15~16초는24frames/첫포함15000·마지막15958·첫제외16000tick으로확인했다. 이번전체190/정상bins는미실행이며최신전체gateR150을유지한다. 실제음수표본/EOF/다른clock/렌더/품질승인으로확대하지않는다.

2026-10-28 R151 중앙 PM. [native source probe](clip-native-probe.md)의보고기간만으로프레임경계를승인하지않는다. 실제source/tool원lease를보유한NativeClipSourceLease.observe_segment가고정probe를실행하고원decoded PTS창을대조한다. 새로운worker/소유권규칙·추론·다운로드·수동GUI는추가하지않는다.

공통frame_thresholds는source/segment/선택영상track과유효구간을검증하고기존TimelineTransform의ceil변환으로[start,end)의signed integerPTS임계값을계산한다. 실제포함기준은start이상/end미만이며키프레임seek지점으로대체하지않는다. evaluate_frame_boundaries는8192개이내의엄격증가decoded PTS와start이전또는같은관측·end이상첫제외프레임을요구한다. 첫/마지막포함PTS·첫제외PTS·원구간/임계값·포함수량을기록한다. 관측부족/중복·역순/빈구간은NeedsInput이다. EOF에제외프레임이없는경로는별도증명전미지원으로남기며마지막프레임을꾸며내지않는다.

native관측은원source/frameclock의기준video stream을선택하고원파일·probeguard를빌린NativeClipBoundaryLease를반환한다. rawframe JSON과source SHA/source관측SHA/probe SHA/rawframe SHA·정확interval·경계사실을연결한다. lease는원source를borrow하므로물리decoder종료까지원guard수명을유지하며Serialize/Clone권한객체를만들지않는다. 다른video stream의clock정규화는후속이며다른source/잘못된구간은process시작전공통정책에서거절한다.

실제seek interval은clockorigin의absolute microseconds floor에구간앞뒤2초관측margin을추가해구성한다. 음수origin은부호를보존하며0으로추정하지않는다. 단일window최대60초/8192frames/stdout2MiB/최대120초deadline·기존stderr/프로세스종료정책과file protocol/고정demuxer·MP4 drefs차단/thread1을사용한다. seek가앞뒤로달라져도실제관측coverage가부족하면성공하지않는다. pts가없으면best_effort만으로채우지않고둘이서로다르면거절한다. 원streamindex/time_base도source관측과일치해야한다.

[FFprobe 공식 문서](https://www.ffmpeg.org/ffprobe.html)의read_intervals seek비정확성/show_frames계약을확인했다. 합성값의음수/VFR정책통과를실제음수표본검증으로계상하지않는다. 실제시험은원Sintel의15~16초1구간을decoder로관측하고반복원출력SHA·외부source거절을확인하도록작성한다. 인코딩·색/회전·선택자막/음성·출력발행·원Jobs/SDK·CLI는후속이다. production.render_short는미지원유지.

기존modootoday QA fixture/코드검증스킬의정확출처·해시는[source admission](clip-source-admission.md)에기록한것을적용했다. 값검증/실제decoded관측/렌더·품질승인을구분하며원소유권/WIP/private overlay·C-only/heavy1·보류를유지한다.
