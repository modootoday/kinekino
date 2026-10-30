---
id: sot:kinekino/native-source-probe
title: 숏폼 native source probe 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 숏폼 native source probe 계약

R148 구현·검증완료범위: [parser3/실제Sintel source probe1·strict/fmt/no-run·지정190/정상bins·2406사후custody](../../../verification/application/clip-native-probe-20261028.md). 실제관측pin을source admission과대조했으며SDK/CLI노출·encoder/프레임경계/렌더는후속이다. R147첫컴파일가시성오류도원증거로보존했다.

2026-10-28 R147 중앙 PM. [source admission](clip-source-admission.md)의 실제 파일 관측 어댑터를 추가한다. 기존소유권/WIP·private overlay·보류를유지한다.

FfmpegClipSources는Rust조립의고정media ID→로컬경로/MP4또는Matroska카탈로그와probe절대경로/SHA·최대source bytes·단일hash/probe deadline을받는다. 요청JSON은경로/도구/관측성공을주지않는다. inspect_source는원파일과조상guard·pinned probe guard를NativeClipSourceLease에유지한다. source해시를동일원파일에서checkpoint와함께계산하고헤더/컨테이너일치를확인한다. probe는기존frame_capture의owned-process runner·tool guard를crate내재사용한다. 새worker·프로세스소유권규칙을만들거나non-Send reviewmedia lease를강제전환하지않는다.

clock기준은attached picture가아닌가장낮은container video stream index다. start_pts를정수i64로읽어음수까지보존하며누락을0으로추정하지않는다. 기준stream의time_base와크기를사용한다. video/audio/subtitle의전체컨테이너index/kind/codec을정렬보존하고중복index·unknown codec·stream수64초과·픽셀예산초과를거절한다. codec미지정보/관측누락은NeedsInput이다. attached picture나다른video stream의선택을인코더지원으로승격하지않는다.

duration은기준stream의양수duration_ts×time_base를정수연산으로마이크로초아래쪽으로변환한다. 없으면format.duration의정확한양수소수값(최대6자리)을ContainerReportedSpan으로기록한다. 이fallback은프레임존재나정확한video종료시각의증명이아니다. 음수origin을임의로duration에서빼거나container기간을실제경계프레임으로간주하지않는다. 실제렌더준비단계가선택구간양끝의decoded frame/PTS와지원stream·색/회전정책을추가검증해야한다. basis를숨겨같은의미로표시하지않는다.

observation_sha256는Rust struct의고정field순서JSON bytes인{contract:"clip-source-probe/1",source_sha256,probe_sha256,raw_probe_sha256,reference_stream_index,duration_basis}의SHA256이다. raw_probe_sha256는bounded원FFprobestdout bytes의SHA256이다. filename/임의태그를관측에넣지않고show_entries로필요stream/disposition/rotation/format기간만요청한다. 동일source라도tool/raw관측/기준stream/basis변경은새관측pin이며조용히갱신하지않는다. 원stdout은lease에서보존/열람할수있다.

probe에는file protocol·고정demuxer·MP4 drefs차단·단일thread·max_alloc/probe bytes/analyze기간·stdout256KiB와기존stderr상한을적용한다. hash와probe에같은deadline/control을적용한다. 원owned runner의물리종료/receipt/cleanup책임을유지하며timeout관측만으로프로세스를재시작하지않는다.

이관측은미디어pin후보와source custody다. encoder가용성·라이선스·프레임경계·HDR/회전·자막/폰트·output/Jobs·렌더/품질승인을의미하지않으며production.render_short는미지원이다. 실제표본시험은기존Sintel원본에대해probe만자동실행하며추론·다운로드·수동GUI를재개하지않는다.

[FFprobe 공식 문서](https://www.ffmpeg.org/ffprobe.html)의show_entries와stream/disposition출력계약을참고했다. modootoday QA fixture/코드검증 스킬 출처·해시는[source admission](clip-source-admission.md)에기록한것을적용해parser합성시험과실제probe/원guard시험을구분한다. 제3자tool코드나새의존성은추가하지않았다.
