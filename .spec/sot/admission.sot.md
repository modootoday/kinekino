---
id: sot:kinekino/admission
title: 숏폼 선택 자막 admission과 렌더 연결
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 숏폼 선택 자막 admission과 렌더 연결

R181 현재 검증: optional host JSON catalog를 실제 제품 CLI와 typed SDK가 소비하는 source관측→프로젝트작성/수정저장→두crop+선택자막 원 Jobs 렌더를 통과했다. 선택 자막14CLI/typed source·submit 두렌더,기존13CLI 회귀,기본strict/no-run/fmt·지정190/정상2bins·2445파일 사후content/identity 동일을 확인했다. 아래 이전 ‘CLI 미검증’ 문장은 당시 상태다. 전체 기능 인수·preset 실체/모든키/실제VFR/색/HDR/회전/EOF·모든오류/취소/cleanup/복구/ACK·GUI/사용자품질은 남는다. caption은 manual fixture이며 원 대사 provenance나 AI 품질 승인이 아니다.

R180 검증 보정: 원 Jobs의 실제 선택 자막 및 기존 자막 없는 경로 각1시험,기본strict/lib-no-run/fmt·2445파일 사후custody를 통과했다. 선택 자막 결과는 기존 Sintel15~16초/1080×1920/H26430frames/AAC와3active run의 manual caption이며 원 worker Completed/동일key재사용/정상종료/재열기까지 입증한다. raw cue 원문 provenance·실제 CLI/typed SDK·host JSON catalog 및 전체 오류/취소/복구/ACK·GUI/색/사용자품질은 남는다. 이하 ‘실제 Jobs 진행 전’ 문장은 검증 이전 상태다.

R180 현재 구현 상태: R179에서 dedicated 영상 overlay의 실제24frame/반열린 경계·PNG guard·typed metadata/full decode를 검증했다. 기존 자막 없는 후보는 recipe/2, 전용 자막 후보는 recipe/3로 구분하며 원 증거는 보존한다. R180은 원 Jobs의 같은 worker/verifier/publication gate에 연결했고 strict를 통과했으나 실제 Jobs/CLI 인수는 아직 진행 전이다. 아래 R169~R175의 ‘다음/미구현’ 문장은 당시 상태이며 최신 검증은 [근거](../../../verification/application/clip-selected-subtitles-20261028.md)를 따른다.

고정 host 설정은 clip-render-host/1의 optional subtitle_assets{documents:{id:absolute_path},styles:{id:absolute_path}}로 제공한다. 일반 SDK/CLI submit은 저장된 프로젝트와 document/style publication SHA만 전달한다. 카탈로그 경로의 binding SHA와 전체 host 설정 SHA를 capture에 묶으며 무설정은 Unsupported,없는 ID는 NeedsInput,실행 시 파일/media/clock/style/font pin 불일치는 SourceExpired 등 기존 오류로 처리한다. optional 필드 생략은 기존 자막 없는 설정과 호환된다. 이 JSON 소비/실제 CLI는 아직 미검증이다. 원 Jobs scope에 admitted document/style/font 및 PreparedClipOverlay를 보유해 원 process·검증·불변 발행이 끝날 때까지 guard가 유지된다. 별도 worker나 예약/발행 gate를 추가하지 않는다.

R175 active CPU 합성: 같은 프레임에 활성인 cue는 원 문서 순서로 한 자막 box 안에 함께 표시한다. frame span 이벤트로 동일 active cue 집합의 run을 만들고 원 raw text를 유지한 채 newline으로 배치한다. 집합 전체에 같은 max_lines/box overflow 검사를 적용하며, 초과하면 NeedsInput으로 수정 요청한다. 기존 Sintel 원 source lease/24프레임 PTS와 결합한 실제시험에서 [0,6) 첫 cue·[6,12) 두 cue·[12,24) 둘째 cue의 CPU PNG,두 줄64px 및 전체1줄 제한 거절을 확인했다. caption 텍스트는 manual fixture이며 원 영상 대사 provenance가 아니다. image recipe/2에 cue_ids/각 원문 SHA/newline separator를 기록하며 원 recipe/1 evidence는 보존한다. 실제 영상 overlay의 경계 픽셀과 원 Jobs/CLI 연결 검증은 남는다.

다음 영상 합성은 원 crop/fit 뒤, fps 변환 전에 같은 source frame 순번에서 활성 run만 표시한다. 설치된 FFmpeg8.1.1 배포 doc/ffmpeg-filters.html(SHA169EFAEC100B513BD8389E548A44473332C397BAAB27B16EF9E62C0BBF9BC051)의 Timeline editing/framesync/overlay 절을 확인했다. enable의 n은0기반,alpha=straight를 명시할 수 있고 두 입력은 시작 PTS 정렬이 필요하다. 공식 웹 페이지 조회는 timeout이었으며 실제 고정 엔진의 경계 픽셀 시험 전에는 이 문서만으로 동작을 확정하지 않는다. 원 source·subtitle/style/font·임시 이미지 guard는 decoder 종료까지 유지하고 단일 원 Jobs gate/검증·발행 흐름을 사용한다. 이 문단은 아직 미구현 설계다.

R172 실제 CPU 이미지 1시험·strict/lib-no-run 및 2440파일 사후 custody를 통과했다. 투명/반투명 픽셀·PNG 왕복·공통 font 전달·nominal 누락 및 layout overflow 거절의 범위만 입증한다. [원 검증](../../../verification/application/clip-selected-subtitles-20261028.md)을 참조한다. backend glyph 선택/ink 전체 경계·GUI 시각 승인·사용자 품질은 미검증이다.

R173 원 프레임 연결 WIP: application의 map_subtitle_frames는 admitted segment와 전체 source pin을 다시 대조하고 같은 decoded PTS에서 시작/끝을 각각 정수 올림해 포함된 원 영상 프레임의 반열린 순번으로 옮긴다. overlap cue 순서와 0프레임 cue를 보존하며 fps 변환 뒤의 임의 부동소수 시간으로 바꾸지 않는다. native boundary lease가 기존 raw window에서 검증한 PTS를 보유하고 같은 원 source owner를 통해 이 정책을 호출한다. 이 값은 관측 사실이며 별도 encoder/파일 권한을 부여하지 않는다. 합성 원리 시험과 native 연결의 검증은 아직 진행 전이고 실제 영상 합성/Jobs/CLI는 남는다.

R169 CPU 이미지 단계 WIP: `clip_subtitle_raster`는 admitted document/style/font owner를 borrow한 채 선택 cue 하나를 기존 Meari CPU renderer로 투명 PNG에 그린다. 고정 CSS와 HTML escaping만 사용하고 style/source 파일 경로나 raw text를 실행 문자열로 해석하지 않는다. normalized box는 출력 픽셀로 내림하며 실제 slot/content layout을 같은 renderer에서 읽어 box와 max_lines 초과를 NeedsInput으로 거절한다. 자동 축소·누락·ellipsis로 성공 처리하지 않는다. native CPU premultiplied RGBA를 PNG straight alpha로 변환하고 기존 strict PNG parser로 검사한다. 아직 실제 alpha/pixel 시험 전이다.

선택된 공통 font bytes의 nominal cmap은 cached `ttf-parser=0.25.1`로 검사한다. [고정 버전 공식 API](https://docs.rs/ttf-parser/0.25.1/ttf_parser/struct.Face.html)의 glyph_index는 nominal mapping이며 실제 backend glyph 선택/shaping을 입증하지 않는다. 누락 문자는 NeedsInput으로 거절하고 source/font bytes·라이선스·release metadata·실제 asset delivery를 별도 기록한다. backend_glyph_choice_observed=false/ink_overflow_measured=false/text_source_provenance_verified=false/quality_approved=false를 유지한다. renderer는 동기 CPU 호출과 pre/post checkpoint이므로 cooperative_checkpoints_only=true이고 선점 취소·hard deadline 인수를 주장하지 않는다. renderer 내부·Meari 소유 코드는 수정하지 않았다.

의존성은 이미 lock/cache에 있던 ttf-parser 0.25.1(MIT OR Apache-2.0, crates.io package checksum d2df906b07856748fa3f6e0ad0cbaa047052d4a7dd609e231c4f72cee8c36f31)을 native feature의 직접 참조로만 추가했다. cache archive SHA를 실제 대조했다. private mirror Cargo.lock은955DEBA0338A3A32ED64D6B2A500C192CD55517336FE531D6EED374D0B2916A5→6616F5A1688488000D2C6BFECFAFBF931BE5AC38C7778AA288F54ACDEA4D0556, diff는 root package의 ttf-parser 한 줄이다. 원 lock은 preimage-r168-subtitle-raster에 보존했고 canonical root/global Cargo·다른 프로젝트는 변경하지 않았다. 2545 no-deps metadata는 종료0,2547 full metadata는 미캐시 android-activity0.6.1의 다운로드 요구를 offline가 거절해 종료1이었다. 관련 없는 target 패키지는 받지 않으며 원 실패를 보존한다.

R169 시험은 기존 R168 fixture record를 pin으로 읽고 새 fixture 복사본에서 한글/기호/일본어,투명 바깥 영역·반투명 background/straight-alpha PNG round-trip,실제 font delivery/layout,없는 cmap/overflow 거절을 확인하도록 작성했다. 이 fixture 스타일은 승인된 제품 UI 시안이 아니고 원 파일을 변경하지 않는다. 비디오 source frame 매핑·overlap cue 합성·원 Jobs/CLI 연결은 다음 단계다. UI 규칙의 자동 CPU 결과와 실제 GUI/시각 인수 구분·기존 폰트 소유권을 유지한다.

2026-10-28 중앙 PM, R167 private overlay WIP. [승인 첫 버전](shorts.md)의 선택 자막을 같은 Rust SDK/Jobs 렌더에 연결한다. 현재 단계는 문서/source/cue/style/font admission과 시간 선택이며 실제 렌더는 아직 없다. 원 소유권·UI 스타일 소유권·WIP·private overlay·원 증거/첫 실패·모든 보류를 유지한다.

`ClipSubtitleDocument`는 기존 `CueSourceSnapshot`/stable cue ID·문자열·시각·선언된 원문 SHA를 재사용하고 정확한 `ClipSourcePin` 전체를 연결한다. 별도 원문 저장소나 AI acceptance를 만들지 않는다. 명시 time_basis=media_relative_us만 수용하고 origin=explicit_manual_snapshot으로 구분한다. 이는 사용자가 승인한 AI 결과나 원 caption 자산 provenance 검증이 아니다. 문서의 raw JSON 전체 SHA와 id/revision, media SHA·clock·tracks·duration은 각각 검증한다. 미래 accepted correction 어댑터는 기존 immutable accepted projection을 재사용해야 하며 manual snapshot을 acceptance로 승격하지 않는다.

`prepare_subtitles`는 선택 ID의 존재/중복·명시 시각·media duration을 검증한다. 원 document cue 순서를 보존하며 선택 cue를 clip의 반열린 범위와 교차해 출력 기준0의 정수us로 기록한다. 겹친 cue는 합치지 않고 각각 유지하고, 범위 밖 선택은 omitted_outside_clip에 기록한다. raw text와 source 시각을 보존하며 HTML/ASS/filtergraph escaping은 렌더 어댑터의 책임이다. 아직 이 값만으로 렌더 frame 순번이나 파일 권한을 만들지 않는다.

스타일은 immutable ArtifactPin의 strict JSON이다. 명시 font refs/SHA, 고정 출력 픽셀의 font/line height, normalized box, RGBA, align/max lines를 가진 사용자/담당 입력이며 새 공통 UI 시안을 승인한 것으로 표시하지 않는다. native `SubtitleAssetFiles`는 host가 고정한 ID→문서/style 파일만 열고 원 일반 local-Disk 조상/file guard와 raw SHA를 유지한다. 폰트는 기존 Meari product_styles의 VerifiedAssets에 있는 Font role·SHA·실바이트·길이·라이선스로만 해소한다. 시스템 폰트나 임의 파일 경로로 fallback하지 않는다. 공통 폰트 README의 SHA로 release metadata 근거를 함께 보존한다. 기존 자산은 삭제/교체하지 않는다.

`PreparedClipSubtitles`는 non-Serialize 객체로 문서/style/폰트 owner를 렌더 종료까지 유지하도록 설계했다. glyph coverage·shaping·overflow·실제 font delivery·자막 픽셀 합성·video source frame 매핑·Jobs capture/실행/발행/CLI는 다음 단계다. 이 구현만으로 renderer의 기존 선택 자막 Unsupported를 해제하지 않는다. 선택/저장 구조를 기능 지원으로 계상하지 않는다.

검증은 fresh native 문서/style 파일과 실제 공통 폰트를 읽어 source/clock pin·누락 ID/시각·틀린 font SHA·취소 전 I/O·원 파일 쓰기/rename 차단과 시간 교차/겹침/제외 end를 확인한다. 미디어는 fixture이며 renderer/GUI/추론·사용자 품질 인수가 아니다. 기존 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100), code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용해 근거 범위를 나눈다.
