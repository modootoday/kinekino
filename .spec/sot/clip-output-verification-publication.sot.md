---
id: sot:kinekino/clip-output-verification-publication
title: 숏폼 출력 검증과 불변 검토 산출물 발행
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 숏폼 출력 검증과 불변 검토 산출물 발행

2026-10-28 중앙 PM. [수동 후보](clip-manual-candidate.md)를 최종 산출물로 바로 노출하지 않는다. 현재 구현은 검토용 파일 묶음이며 원 Jobs/CLI 연결과 사용자 품질 승인은 별도다.

ClipOutputVerifier는 고정 probe/decoder SHA와 기존 tool·조상 guard, 명시 C-cache tmp root, 하나의 deadline을 사용한다. 후보 bytes/크기/SHA와 원 encoder SHA를 먼저 대조한다. 원 MP4 bytes에서 ftyp·유일한 moov/mvhd·movie timescale/duration을 읽고 fragmented moof·잘못된 box·누락을 거절한다. 새 private tmp namespace에 원핸들 create-only로 복사한 뒤 파일/조상 guard와 SHA를 고정해 probe와 전체 decode를 수행한다. 실패/성공 spool과 원 owned process 증거는 보존하며 자동 cleanup을 완료로 표시하지 않는다.

출력 probe는 H264/yuv420p·SAR1:1·해상도·fps·decoded frame 수, 시작PTS0, 영상/선택AAC 길이·sample rate/channels, 불필요한 track 부재, 선언한 BT709 태그를 검사한다. 요구 프레임 수는 출력 fps의 frame grid로 ceil하며 요청 길이와 관측 길이를 모두 기록한다. video clock 1tick·audio 1sample·실제 mvhd movie 1tick의 양자화 범위를 넘는 차이는 거절한다. 원 R152/R153의66.667ms/21.333ms drift는 성공으로 취급하지 않는다. 전체 decode는 -xerror/-err_detect explode와 원 deadline/control을 사용한다. metadata 태그 일치는 source 색 정확도·청감/음량·PCM padding·사용자 품질 승인이 아니다.

VerifiedClipCandidate는 생성자가 공개되지 않은 non-Serialize/non-Clone 타입이며 원 후보를 borrow하고 검증 파일·조상 guard를 소유한다. 출력·검증도구·raw probe/decode SHA와 실제 metadata를 보존한다. 이 객체만 ClipArtifactStore.publish에 전달할 수 있다. 검증되지 않은 bytes나 caller JSON으로 발행 권한을 만들지 않는다.

ClipArtifactStore는 고정 root와 유효 artifact ID만 받는다. 원 writer lock 아래 새 clip-ID 디렉터리에 clip.mp4/render.json/verification.json을 각각 원핸들 create-only로 쓰고 읽기 guard·SHA/크기를 확인한다. manifest.json의 완전한 create-only 발행이 유일한 commit point다. 디렉터리 전체가 한 번에 숨김 해제되는 방식은 아니며 commit 전 파일은 완료 산출물이 아니다. inspect는 반드시 외부에 보존된 manifest SHA pin을 받아 전체 파일/lineage를 재검증하고 원 guard를 반환한다. 기존 target/partial을 덮어쓰거나 자동 삭제·재시도하지 않는다.

최종 gate는 기존 ExecutionControl.reserve_publication을 사용하고 새 cancellation arbitration을 만들지 않는다. commit 전 취소는 manifest를 만들지 않고 partial을 보존한다. commit 뒤 취소는 출력을 없다고 바꾸지 않는다. commit 뒤에도 원 deadline은 유지하며 Timeout 등 실패에는 publication_committed/artifact ID/manifest SHA를 남겨 별도 inspect로 조정한다. 실제 Jobs control이 공급한 attempt identity만 manifest.execution에 기록하며 없으면 null이다. 시험용 control 성공을 원 worker/Jobs 인수로 계상하지 않는다.

발행 결과는 review_artifact_only=true/quality_approved=false다. 선택자막·구간별crop·preset catalog·source 색/회전/HDR·EOF/다른clock·전체 취소/디스크/재시작/ACK·원Jobs/SDKCLI·GUI·AI 사용자 품질 인수는 남는다. `.spec` 원본/실패와 WIP·소유권·private overlay, C-only/heavy1·WARP/추론/다운로드/수동GUI/push 보류를 유지한다.

기존에 읽은 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100)와 code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용해 parser fixture·실제 파일 I/O·발행 control fixture·원 Jobs 검증을 구분한다.
