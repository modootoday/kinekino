---
id: sot:kinekino/clipdocument
title: 공통 ClipDocument와 수동 숏폼의 입력 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 공통 ClipDocument와 수동 숏폼의 입력 계약

R139고정입력:2283snapshot종료0,2388파일/inventory8373A89D56544742646B8089790D85471F98B6B7295036F580D9C73FF4A7A162,SOURCE.md1CC00F2C2D43672C9284C08C1D07444B32C21AC678BD2169C389DFBE5B1BEAFE.2284기본lib/Jobs/CLI no-run진행중이며새4시험/strict/후속gate는아직미완료다. 최신완료기준선R138·기존보류유지.

R139 구현 중이다. [승인 숏폼 유스케이스](shorts.md)와 [CLIP 요소 계약](../../presentation/studio/shorts.md)의 공통 프로젝트를 Rust domain/clip_document에 추가했다. domain은 값 검증만 하며 OS경로/engine/GUI에 의존하지 않는다. source/admission·원프로세스·encoder 가용성·파일 발행은 후속 application/infrastructure 책임이다. production.render_short는 여전히 미지원이며 문서형식 통과를 실제 렌더 지원으로 표시하지 않는다.

| 승인 요구 | 공통 값 |
|---|---|
| CLIP-01/02/03/12 | source pin 배열·순서 있는 segment 배열·half-open source us 구간·document ID/revision |
| CLIP-07/08/09 | 정수 백만분율 crop key·fit/fill·선택된 source-bound subject ref |
| CLIP-10 | source-bound 자막 문서 pin·고유 cue ID·공통 style/font 자산 pin |
| CLIP-11 | 원 container stream index·0..100 volume·mute/disabled의 명시 구별 |
| CLIP-13/14 | preset catalog pin·크기·기약분수 fps·container/codec/color 선언·정확 재인코딩/빠른컷 구별 |

미디어 SHA·관측 SHA·트랙 목록/길이/크기는 선언 값이다. 문자열/참조 검증이 원본 검증이나 custody/Verified 권위를 만들지 않는다. 후속 preflight는 실제 source lease/hash·도구/clock/stream 관측을 비교하고 지원하지 않는 조합을 명시 거절해야 한다. 재생 엔진의 native TrackId와 컨테이너 stream index는 다른 식별자다. source clock은 signed decimal origin ticks와 유리수 time base를 보유하며 기존 TimelineTransform으로 정수 변환한다. range us는 그 명시 origin에 상대적이다.

crop은0..1,000,000 정수 범위이며 덧셈 overflow·영면적·구간끝 key·역행을 거절한다. 첫key는구간시작에있어야한다. key 사이의 실제 렌더 전환/지원은 후속 encoder 계약에서 고정하며 임의 추적/보간으로 승격하지 않는다. 구간은비어있지않고source길이안에있어야하며출력누적길이overflow를거절한다. JSON 시간/수정번호는 기존 DecimalU64, origin은 canonical i64 문자열로 보존한다.

프리셋은후보 선언이며H.264/AAC·다른enum의존재가실제encoder·권리·재생호환승인을뜻하지않는다. SDR만현형식에표현하고HDR/추적/다중클립합성등실제지원은별도admission판정이다. sources/segments각128·crop keys128·선택cues4096·document1MiB는저장한도이며플랫폼제작규정이나성능보장이아니다. 편집/undo·불변프로젝트저장·preflight·encoder adapter·job/SDK/CLI/실제VFR/정확컷·취소/발행인수는아직남는다.

preimage-r138-clip-document에domain mod 원본보존,2282fmt완료/2283snapshot진행중. 새4시험은정수정밀도/serde·원시간축변환·구간/누적overflow·crop/stream/foreign자막참조경계를다루며실제미디어/렌더시험이아니다. modoo catalog에서새domain intent를탐색했으나frontend-first/추론훈련 등검색결과는이작업에불일치하여채택하지않았다. 기존읽은modootoday/software-qa/code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B),test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100)을적용해선언검증과실제source/encoder검증을구분한다. 기존소유권/WIP·private overlay·AI승인0/9·보류유지.
