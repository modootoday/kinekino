---
id: sot:kinekino/clip-preset-admission
title: 숏폼 출력 프리셋의 정의와 입력 검증
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 숏폼 출력 프리셋의 정의와 입력 검증

R184 검증 완료 범위: 인코더·원 Jobs capture/실행/발행·SDK capabilities/typed clip_preset_catalog·실제 CLI에 필수 검증을 연결했다. 앱 기본 proposal과 fixed host_manual 파일을 각각 실제 CLI/SDK로 조회·선택·저장해 원 Jobs2렌더씩 확인했고,같은 pin의 width 변경 프로젝트는 Jobs 생성 전에 SourceExpired로 거절했다. source snapshot과 render facts의 preset_definition을 대조했다. 양구성 strict·기능부재거절·프리셋/인코더 정책·실제파일guard/경계pixel·지정190/정상2bins·2449파일 사후custody가 통과했다. 아래 R182/R183의 WIP/미연결 문장은 당시 이력이다. 전체 오류/취소/cleanup/복구/ACK·모든 profile/분수fps/최대길이·공식 플랫폼/사용자 품질·GUI 인수는 남는다.

R183 통합 WIP: R182 정책3/실제Windows파일1·strict/lib-no-run/fmt·2449사후검사를 통과한 owner를 인코더의 필수 입력으로 연결했다. 원 Jobs capture/실행 재검증·발행 scope 및 optional host preset_files,SDK capability/typed catalog도 연결했으나 이 통합의 실제시험은 아직 없다. 이하 R182의 ‘미연결’ 문장은 당시 상태다. `preset_files`는 ID→고정 local JSON 파일 map이며 생략 시 기본 proposal2만 사용한다. 카탈로그 경로 binding과 전체 host SHA를 capture에 묶고 실제 정의 증거를 source snapshot/render facts에 남긴다. capabilities의 정의 조회는 encoder 가용성·플랫폼/품질 승인이 아니다. 기존 publication은 계속 읽되 알려지지 않은 과거 선언 pin으로 렌더를 조용히 허용하지 않는다.

2026-10-28 중앙 PM, R182 private overlay WIP. R181의 실제 SDK/CLI/190/정상 bins를 보존한다. 이 문서는 남아 있는 preset 출처·설정 일치 공백의 계약이며 새 프리셋을 플랫폼 공식 규격이나 실제 인코더 가용성으로 표시하지 않는다.

`ClipPresetDefinition`은 clip-render-preset/1, stable preset_id/revision, label, origin, 출력 profile(width/height/기약분수 fps/container/video/audio/color), 총 출력 max_duration_us를 가진 strict JSON이다. 원 Pin의 ID/revision/SHA와 전체 profile을 `ClipOutputPreset`에 대조한다. 한도가 다른 정의를 같은 핀으로 선택하거나 width만 바꾸는 것을 SourceExpired로 거절한다. ClipDocument 자체의 저장/역사 형식은 변경하지 않는다.

기본 정의 2종은 application_proposal인 1080×1920/30fps/MP4/H264/SDR와 AAC 또는 무음이다. 최대60초는 이 앱 제안의 값이며 서비스 최대나 공식 업로드 규격이 아니다. typed definition의 고정 serde JSON 바이트 SHA를 catalog pin으로 사용한다. 본문이 바뀌면 새 revision/핀을 발행해야 한다. 모델·폰트·엔진/라이선스·실제 재생 검증을 이 데이터로 대체하지 않는다.

로컬 확장은 host가 고정한 ID→absolute ordinary local-Disk 파일만 사용한다. 파일은 host_manual이어야 하고 앱 proposal ID를 shadow할 수 없다. 원 raw JSON SHA/길이·ID/revision을 확인하고 조상/file read guard를 `PreparedClipPreset`의 non-Serialize owner가 보유한다. 원 바이트가 같은 객체의 authority이며 DTO/catalog 응답만으로 실행 lease를 재구성하지 않는다. unknown ID는 NeedsInput,원 핀/설정 불일치는 SourceExpired,strict schema/출처 위장은 InvalidArgument다. 기간 초과는 NeedsInput,관측 취소는 원 Cancelled를 유지한다.

정의는64KiB,고정 파일64개,profile은8192축/기존 MAX_FRAME_PIXELS,정의 길이는180초까지의 저장 한도를 둔다. 실제 렌더의60초·단일clip·codec·메모리/도구 한도는 별도이며 정의를 조회했다고 전체 renderer 지원으로 표시하지 않는다. `engine_availability_verified`/`platform_requirements_verified`/`quality_approved`는false다.

R182는 application/native 입력 검증과 목록 관측을 구현했지만 기존 prepare_encoder·원 Jobs·SDK capabilities/typed catalog·실제CLI에는 아직 연결하지 않았다. 해당 공백이 닫힐 때까지 기존 fake/manual catalog pin을 검증된 preset 출처로 계상하지 않는다. 다음 필수 통합은 encoder가 definition owner를 도구/출력 발행 종료까지 유지하고 원 host binding/capture와 SDK/CLI에 같은 catalog를 사용하는 것이다. 기존 저장 publication을 덮어쓰지 않고 새로운 publication에 해소한 preset pin을 명시한다.

기존 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용한다. 합성 문서의 입력 정책과 실제 Windows 원파일/SHA/쓰기·rename 차단을 구별한다. [검증](../../../verification/application/clip-preset-admission-20261028.md).
