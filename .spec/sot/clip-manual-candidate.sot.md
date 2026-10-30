---
id: sot:kinekino/clip-manual-candidate
title: 숏폼 수동 렌더 후보 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 숏폼 수동 렌더 후보 계약

현재 R154 출력 계약은 아래 R152의 fragmented pipe 후보를 대체한다. 기존 owned runner의 원 stdout은 create_new read/write 일반 파일이고 reader는 독립 open한다. 고정 fd: -fd1로 그 원 stdout만 사용해 seek 가능한 일반 MP4와 edit list를 기록한다. 최종 출력 경로·새 worker를 만들거나 임의 fd를 요청에서 받지 않는다. 후보 facts는 recipe/project 발행/tool·inventory/경계 SHA를 함께 보존한다. [실제1080×1920/H26430frames·선택AAC·시작0/길이1초/전체decode와 회귀](../../../verification/application/clip-manual-candidate-20261028.md)를 확인했으며, 원 R152의1.066667초와 R153의1.021333초 실패도 보존했다. 이것은 단일 후보의 시간축 확인이며 제품 발행/Jobs/CLI/색·사용자 품질 승인은 아니다.

2026-10-28 R152 중앙 PM. [source](clip-source-admission.md)·[encoder](clip-encoder-admission.md)·[decoded 경계](clip-frame-boundaries.md)를하나의실제후보인코딩에결합한다. 최종output발행·Jobs/CLI지원과구분한다.

manual_clip_plan은media의정확발행document SHA와encoder준비document SHA를대조하고단일segment·고정crop/fit·선택오디오를계획한다. source ID/SHA/clock관측·segment ID/구간/선택track·정수threshold가decoded경계와같아야한다. crop은정규화정수에서source pixel을floor변환하고빈pixel을거절한다. output픽셀예산/최대120fps/단일60초·FFmpeg정수표현범위를검사한다. 여러segment/구간별crop/선택자막/subject합성은명시Unsupported로남기며요구사항에서삭제하거나조용히생략하지않는다.

native encode_candidate는원media source 객체와NativeClipBoundaryLease.source의주소동일성을검사하고원source·probe·encoder guard를물리프로세스종료까지보유한다. source ID도대조하므로동일content SHA의다른alias를원sourceowner로바꾸지않는다. callerJSON의command/filtergraph/path를실행하지않고typed계획의숫자에서고정argv를조립한다. 기존owned-process runner·timeout/취소/receipt/물리종료를재사용하며새worker를만들지않는다.

실행은copyts/고정demuxer/file protocol/MP4 drefs차단·원clock의2초앞seek/noaccurate_seek후정수trim으로반개구간을선택한다. crop exact=1,fit의scale/pad 또는fill의scale/crop,setsar/format/fps를거쳐MP4/libx264 veryfast/crf18/yuv420p와선택AAC192k를만든다. 오디오는공통absolute초구간을trim하고0~100volume/muted를적용한다. 원시간·패킷지연/priming/최종A/V길이의추가검증은필요하며단순목록지원으로승격하지않는다.

source색정보누락을SDR로추정하지않는다. 이prototype은Rust조립의원source SHA에묶인명시DeclaredBt709Limited를요구하고입출력matrix/range를고정한다. source_color_mode=host_declared_bt709_limited_unverified/color_verified=false/quality_approved=false로기록한다. HDR transfer·회전은별도검증adapter전거절한다. 이시험선언은제품기본색승인이나사용자품질승인이아니며후속제품색정책을대체하지않는다.

출력은최대128MiB의fragmented MP4 pipe buffer후보이며최종경로를고르거나덮어쓰지않는다. EncodedClipCandidate는private bytes/facts를보유하고output SHA·bytes와계획을기록한다. candidate를최종산출물로발행하는API는아직없다. 별도검증을거친불변no-overwrite publication·output/receipt/ACK·원Jobs/SDKCLI가남는다. encoder인벤토리는새로필요한setsar/setparams도공통요구기능에포함한다.

실제시험은원Sintel15~16초·저장된native프로젝트·source/encoder/decoded경계를조립하고1080×1920/30fps/H264/AAC후보를만들어원probe로frame수/track를검사하고전체decode한다. 원source/저장발행불변·다른document/색선언거절도확인한다. 선택자막/구간별crop·EOF·다른clock/회전/HDR·최종발행/Jobs/CLI/GUI/사용자품질은미완료다. 기존모든승인요구와보류를유지한다.

기존modootoday QA fixture/코드검증스킬의출처·해시는[source admission](clip-source-admission.md)에기록한것을적용해실제후보인코딩·출력관측·제품인수·사용자품질을구분했다. preimage-r151-clip-render에기존mod/encoder요구기능을보존했다. C-only/heavy1·원WIP/소유권/private overlay·WARP/추론/다운로드/수동GUI/push보류유지.
