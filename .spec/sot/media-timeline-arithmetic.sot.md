---
id: sot:kinekino/media-timeline-arithmetic
title: 공통 미디어 시간 산술 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 공통 미디어 시간 산술 계약

2026-10-26. src/domain/media_timeline은 정수 유리수 timebase와 명시 origin의 순수 계산을 소유한다. 계산값은 원미디어 custody·실측 origin·검증된 PTS·AI 품질 근거가 아니다. [실행 근거](../../verification/application/media-timeline-arithmetic-20261026.md)를 따른다.

TimeBase는 양의 u128 numerator/denominator를 최대공약수로 정규화한다. 문자열은 부호/공백 없는 십진수 n/d다. 0·잘못된 표현·범위 초과는 거절한다. checked 정수 곱셈·나눗셈만 사용하며 f64·wrap·saturating 보정으로 시간을 만들지 않는다.

TimelineTransform은 timebase와 signed i64 origin_pts를 명시적으로 받는다. 기본 origin은 없다. PTS→상대microseconds는 origin 이전이면 BeforeOrigin, 표현 범위 밖이면 Overflow다. 반대 변환도 signed PTS 범위를 검사한다. 호출마다 Rounding::Down 또는 Up을 선택하며 Conversion.exact는 나눗셈의 정수 일치 여부만 뜻한다. 실측 정확도나 Verified를 뜻하지 않는다. source/stream/PCM origin의 선택과 priming/offset·불연속 증명은 후속 evidence adapter 책임이다.

기존 frame_capture::seek::TimeBase는 공통 TimeBase로 위임하되 FFmpeg double-expression 경계의 ticks≤2^53 제한과 다음 tick 선택을 유지한다. 기존 probe의 negative/unknown timing 지원 범위를 이 산술 추가로 확장하지 않는다. ReviewSource TimelineStatus는 계속Unverified이며 새산술은 wire Verified 생성 API를 제공하지 않는다.
