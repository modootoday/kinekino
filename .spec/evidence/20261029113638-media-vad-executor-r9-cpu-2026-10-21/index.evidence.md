---
id: evidence:kinekino/20261029113638
title: VAD executor r9 합성 CPU 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# VAD executor r9 합성 CPU 검증

2026-10-21 중앙 PM 단독 슬롯. root `2fd3e94eea4d32a4189b2dfdb331f8af05d0bf7f`, Meari `0b79d57d7a59b3146b7ec72d02e2651b21b166fd`. 고유 actual46=library31+product15, 실패0/ignored0. 실제 모델·worker 실행·추론·다운로드·공개 operation 등록·GUI 검증은0이며 공개 지원/품질 승인 근거가 아니다.

## 입력과 실행

근거 디렉터리: `runtime/verification/media-vad-executor-r9/`. source143은 채택 root94/guard1+r9 원25+P0r4 원10+protocolr2 원13이다. 최종 상속143 manifest `composition-final143.json` SHA256 `071E84F0EF84E745875F9F34030B629D399411BB40E13531641EC5EA504ACDC5`. 초기 manifest는 coordination의 composition-before-sdk.json에 보존했다. 원 후보25 BE3B631A는 변경하지 않았다. 수리 후 최종25/raw는 `final-source25.json`/`final-source25/`, manifest SHA256 `32924663B3D42407016C91D95585BBF507B56E9F134AB1077A4BE1960D5D5EF0`이다.

진입 RAM30.74GiB/VRAM5726MiB/C171.64GiB, canonical runner EEA9/conservative2/mask3/BelowNormal·offline/-j1/thread1. 기존 model-spike target을 사용하고 test-support는 비활성이다. 원 lock6209C8C5와 생성 lock `54B957271C6662EDCF349B9D94BF6F3EA02EFEEBAEBFA7830E0929C46CE3E51D`를 보존했다. 생성 registry33의 name/version/source/checksum 모두 원681에 존재하고 drift0이다. pruning과 로컬 P0/protocol 추가만 허용했다.

최종 fmt0/strict0, fresh initial no-run29435ms0, 제품 수리 no-run1649ms0. 원 list는 library31/product15, 최종 제품 list15다. library31 실제74ms0, 최종 제품15 실제184ms0이며 벽시계 성능 벤치마크가 아니다. 목록과 원 stdout에서 고유 이름을 확인한다. P0/protocol/SDK 전체 시험을 중복 실행하지 않았다.

## 최초 실패와 최소 수리

최초 strict101/18221ms는 native/session 중첩 if 경고2, repair-r1은 단락 평가를 보존한 합성이다. 다음 strict101/1415ms는 제품 fixture의 fs import 누락15사용처이며 repair-r2 import1로 strict0. 그 import 정렬의 fmt1도 보존하고 정렬 후 fmt0이다.

최초 product는13pass/2fail/0ignored였다. 생산 EvidenceRoot가 canonicalize의 VerbatimDisk 경로를 저장하고 child admission에 재사용해 기존 Disk-only 검증에서 거절됐다. repair-r3는 기존 root(path)?의 Disk·reparse·ancestor 검증과 원 열린 handles를 유지하면서 검증된 원 절대 경로 spelling을 저장한다. shared guard 변경·허용 prefix 확대·fixture 우회는0이다. 실제 두 attempt/lease 회귀에 도달해 최종15가 통과했다. 최종 변경은4파일, 나머지139 입력은 불변이다.

## 바이너리와 반환

- library 고정 BIN SHA256 `ECEA6BEC5CEED6C5969E6DEA36FFAA9AFECC0C5DC10826B22597891348B83917`.
- 최초 제품 실패 BIN 고정사본 `87F4FF12371FCC26478E960D68A0DBF5B465E8E72037C7A4AC91DEE107C21261`.
- 최종 제품 BIN `93C1A2E4CB6F80AD7D48C5339AFE273C5AE9CB60090880CA5221C04C282737C1`.

최종 library/제품 compiler 원본과 고정사본 SHA가 일치한다. 최초 제품 compiler 경로는 재빌드됐으므로 최초 bytes는 고정사본으로 보존한다. 원 receipts18 모두 terminal이며 실패4도 보존했다. 기록 PID36은 현재 부재이고 원 exec handle은 모두 회수했다. PID 부재를 실제 worker cleanup 증명으로 사용하지 않는다. root 상속95 SHA차이0/indexempty이며 root tracked status는 기존 Meari dirty 표시만 있었다. 전체 root 파일의 시작 전 snapshot을 취했다고 확대하지 않는다. PM에 heavy 명시 반환 후 문서를 작성했다.

스킬 적용: 기존에 읽은 `modootoday/software-qa/code-change-review-verification`, 참조 `references/verdict-table.md` SHA256 `9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9`. 작성/실등록/통과/지원의 구별, 최초 실패 보존, 실제 소비 경로에서의 원인 수리에 적용했다. 후속은 공용 request/release resolver·실제 worker wire/native 배포·취소/저장/재열기·품질 인수이며 별도 배정이 필요하다.
