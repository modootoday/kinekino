---
id: sot:kinekino/review-content-classification
title: 검토 입력의 컨테이너 헤더 분류
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 검토 입력의 컨테이너 헤더 분류

2026-10-28 중앙 PM. 기존 full-content verified ReviewCaseEvidence와 file/root guard를 유지하고, 최대64KiB prefix를 독립 분류한다. 분류는 코덱/track·decode·시간축·engine admission·품질 증명이 아니다.

`ReviewContentClass`는 기존 iso_bmff_header와 additive matroska_header다. `LocalReviewContentClassifier`는 EBML magic의 입력을 Matroska 분류기로 보내고 그 외에는 기존 ISO BMFF 분류기를 사용한다. EBML 분류 실패를 ISO 성공으로 재시도하지 않는다. 신뢰된 제품 preparation과 session launcher가 이 공통 분류기를 소비한다.

application은 video/mp4↔IsoBmffHeader, video/x-matroska↔MatroskaHeader 대응만 허용한다. MIME만 맞거나 raw header만 맞는 경우는 runtime 요청 전에 Unsupported다. WebM/다른 EBML DocType을 Matroska로 바꾸지 않는다. 원 caller의 MIME/path가 원 package asset custody를 대체하지 않는다.

Matroska 구현의 표준 근거는 [RFC8794 §§4,8.1,11.2](https://www.rfc-editor.org/rfc/rfc8794.html)와 [RFC9559 §4.3](https://www.rfc-editor.org/rfc/rfc9559.html#section-4.3)다. 유한 VINT·header 경계·DocType과 길이 제약을 파싱하며, 문자열 검색으로 임의 payload의 matroska를 인정하지 않는다. 기존 ISO의 ftyp brand 검사는 유지한다.

현재 지원 경계: prefix64KiB, header child32개, header size VINT8bytes/child size VINT4bytes/ID4bytes 이하다. 모든 slicing/length 합은 경계 검사한다. unknown size·잘림·중복 정의·잘못된 필드/버전·잘못된 DocType은 거절한다. EBML version/read-version1, max ID4/max size1~8, DocType정확matroska, document read-version1~4 및 version≥read-version을 검사한다. 기본값이 있는 미기재 scalar는 표준 기본값을 사용한다. finite Void와4byte CRC 필드는 건너뛰지만 CRC checksum 검증을 완료한 것으로 표시하지 않는다. DocTypeExtension·알 수 없는 header 필드와 이 경계 밖 header는 현재 Unsupported다. body·codec·모든 Matroska 문서의 완전 호환을 주장하지 않는다.

기존 source identity에 새 class가 기록되며 producer/publication의 원 source identity 대조도 이를 그대로 사용한다. 새 class를 저장했다고 기존 실제9 historical clock이 검증되거나 association이 Verified로 바뀌지 않는다. 준비된 MKV의 fresh producer·published resolver·association·engine 재생은 별도 실제 gate다. 원9 range=None·품질0/9와 보류는 유지한다.

## 실제 producer 후속의 source format 계약

R74 첫 실제 probe는 raw Matroska 분류 후에도 기존 live execution 인자의 `-f mov`를 사용해 실패했다. 이 실패를 header 분류 실패나 새 source 성공으로 바꾸지 않는다. 후계에서는 공통 live contract가 원 retained source class에서만 demuxer를 선택한다. ISO는 기존 mov/enable_drefs0, Matroska는 matroska이며 네트워크 protocol은 계속file만 허용한다. video/audio/capture와 발행 reader의 argv 재구성이 같은 함수를 사용한다. raw PCM replay는 s16le 계약을 그대로 유지한다. request나 raw trace의 임의 demuxer 문자열을 권위로 받지 않는다.

발행 reader도 기존 ftyp 존재 검사만으로 source를 다시 판정하지 않고, full-content 보유한 source의 bounded raw 분류와 원 token의 class/MIME를 대조한다. 과거 archive v1의 MOV 전용 original_execution 계약은 수정하지 않는다. 이 후계 구현만으로 실제 producer 성공을 주장하지 않으며 원 R74 실패·R75 후속 실행을 별도 보존한다.
