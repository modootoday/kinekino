---
id: sot:kinekino/source-admission
title: 숏폼 source admission
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 숏폼 source admission

R146 결과: [합성포트4시험·기본strict/no-run/fmt·2402사후custody](../../../verification/application/clip-source-admission-20261028.md) 통과. 실제native source probe와encoder는아직없다.

2026-10-28 R146 중앙 PM. 저장된 [ClipDocument](clip-document.md)의 미디어 선언을 실제 검증 권한으로 취급하지 않는다. 이 계층은 native probe/encoder 연결에 앞선 공통 application 정책이다.

SavedClipProjectPin(document ID·발행revision·발행SHA)으로 원 ClipProjectStore의 명시 revision을 읽는다. 발행schema/ID/revision/parent형식·저장SHA와 재계산SHA·유효문서를 확인한 뒤에만 source를 연다. 과거 revision도 정확히 고정되면 사용 가능하며 head로 암묵 갱신하지 않는다. native store의 전체이력 검증 책임은 그대로 유지한다.

Rust 조립이 ClipSourceAdmission과 구체 ClipSourceLease를 제공한다. 요청 JSON은 lease·경로·URL·관측성공을 제공하지 않는다. 실제 adapter는 원파일/조상 guard를 유지하며 bounded hash/probe로 SHA·크기·기간·음수가능한clockorigin/timebase·관측SHA·컨테이너별 track index/kind/codec을 독립 관측해야 한다. 기존 frame probe는 v:0/unsigned 시작시간을 가정하므로 그대로 전체숏폼관측으로 사용하지 않는다. 파일 guard·pinned tool guard·owned process runner는 기존기반을 재사용한다.

application prepare_media는 각 선언과 lease 관측을 모두 대조한다. track 열거순서만 정규화하고 track값·clock·SHA·크기·duration 차이는 SourceExpired다. 모든 source lease를 PreparedClipMedia 수명에 보유하고 Serialize/Clone을 제공하지 않는다. 중간 거절·취소 시 미완성 lease를 회수하고 원오류를 전파한다. 원저장소에 발행하거나 원문서를 변경하지 않는다. source 여는 단계 전후와 각 대조 후에 공통 MutationControl checkpoint를 사용하며 실제 adapter의 hash/probe도 같은 control을 따라야 한다.

이 단계는 media custody 대조 계약이며 encoder/subtitle·font/subject asset/output·Jobs 실행/발행 권한이 아니다. 새 RPC·CLI 지원이나 production.render_short 지원을 표시하지 않는다. 실제native source adapter·관측SHA 산출 계약·encoder pin/라이선스·정확컷/crop/subtitle·취소/검증/발행은 남는다. 처음 단일clip 범위 뒤 다중clip/추적crop을 후속으로 두는 승인설계는 유지하며 필수선택자막을 범위에서 빼지 않는다.

native 연결 전에는 clock 기준stream 선택과 duration의정확한의미(스트림duration_ts/컨테이너기간,음수origin/시간기준),관측SHA의canonical bytes와tool pin귀속을고정해야한다. 기존값모델을근거로이항목을임의추정하지않는다. 기존timeline producer는PinnedTimelineProbe/원media lease/원deadline·pending owner/실제VideoFrame창을가지므로재사용가능성과Send/Sync소유권경계를우선대조한다. 원reviewmedia lease의non-Send소유권을강제로Send로전환하지않는다. [FFprobe 공식 문서](https://www.ffmpeg.org/ffprobe.html)의show_entries·stream선택/원관측출력계약을참고하며공식문서조회만으로실제host동작을입증하지않는다.

기존 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용해 합성포트와 실제파일/encoder 성공을 구분했다. 기존소유권/WIP/private overlay·C-only/heavy1·WARP/추론/다운로드/수동GUI/push보류 유지.
