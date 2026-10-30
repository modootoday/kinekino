---
id: evidence:kinekino/20261029114250
title: 자산 산출물 읽기 검사 — 2026-10-29
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 자산 산출물 읽기 검사 — 2026-10-29

3112 최종 사후 custody 종료0: 실제2503개 source content/identity unchanged. R241 응답 계약 검증 범위를 확정했다.

R241 실제 결과:3107default strict·3108lib no-run·3109SDK 실제 자산/응답 schema 핵심 대응1시험·3110project 회귀13 passed/0 failed/2 ignored·3111fmt check 종료0. 고정 lib fixed-r241/native-core SHA `1B5D999367308030A17FD45E51415665AFF1C009716C4D054335FC885561E400`. fresh copy evidence runtime/verification/source-sdk-r241-inspection-r1이며 원 R236 결과는 보존했다. inventory2503 SHA `E1CE0353BBFA00DF44D2A4452DCF33DA2C92C712365A7687938A9C0517F1B864`, SOURCE.md SHA `D7497867C45D034B7932BE5E3F01AA7EA23A581F063EE1E79A1BE301BE3177CF`.3112사후 custody 실행 중이다. 외부 JSON Schema validator·새 CLI host 실행·전체190/정상 bins는 이번 범위가 아니며 R236 통합과 구분한다.

R241 응답 schema: inspection의 publication/binding·manifest/plan/layer/request/geometry·파일 descriptor를 strict object/필수 필드·정수 문자열·enum·상한으로 명시했다. 기존 asset project의 frame_input/transform/position 정의를 공통으로 소비한다. SDK 실제 응답의 top-level 필수 필드와 nested signed pivot·request schema·quality=false를 대조하는 확인을 기존 실제 시험에 추가했다. JSON Schema 전체 validator 검증으로 과장하지 않는다. 변경 전 파일은 preimage-r240-response-schema에 보존했다.3103/3104fmt·3105snapshot·3106native strict 종료0.3107default strict 실행 중이며 실제 시험/사후 custody는 아직 미확정이다. 최신 통합190/정상 bins는 R236이다.

3102 최종 사후 custody 종료0: 실제2502개 source content/identity unchanged. R240 픽셀 검사 입력·실행 범위를 확정한다.

R240 실제 픽셀 결과:3097default strict·3098lib no-run·3099실제 PNG row/straight+premultiplied/재인코딩 변조1시험·3100원 R236 Jobs 산출물 fresh 복사본/SDK 검사1시험·3101fmt check 모두 종료0. 새 evidence runtime/verification/source-sdk-r240-inspection-r1, 원 R236 marker SHA `17730d05ca01f2a92b505e70f70036b596281e286ecf75bb927216d72083e0e0`. 원본은 수정하지 않았다. 고정 lib fixed-r240/native-core SHA `28DDC152C7CB60ED28FE35D64610D268611D72E27BBB64867232F6794C2E3452`. inventory2502 SHA `7E71884C25C239700469363F00A7B79EAEC5DE622DC2AED994F23F5BDBE2B971`, SOURCE.md SHA `92AC3F0B31EE5D374FA889989A7137C178DE39609D5AA706AFBA3EAB5112737A`.3102사후 custody 실행 중이다. R240 CLI host/원 Jobs 생성 재실행·전체190·정상 bins는 실행하지 않았으며 최신 통합은 R236이다. 품질/원source재렌더·대규모 성능·모든 PNG profile fault는 미검증이다.

R239 3093native strict는 lib test의 items_after_test_module로 종료1이었다. 원 input-r239 snapshot/로그를 보존하고 시험 모듈만 파일 끝으로 이동했다.3094fmt·3095R240snapshot·3096native strict 종료0,3097default strict 실행 중이다. R240 실제 시험은 아직 실행 전이다. 기존 native 검사기/SDK 반환값에 pixel 검사가 자동 포함되지만 R236 실제 실행을 새 픽셀 검사 성공으로 계상하지 않는다.

R237~R239 픽셀 검사 구현: 원 보유 File에서 strict CRC/adler PNG decoder를 row 단위로 실행하고 8bit RGBA/sRGB Perceptual·untagged Gray8 mask·크기·비애니메이션/비interlace를 확인한다. PNG와 raw의 straight/unassociate 정수 변환 및 mask=raw alpha를 대조하며 premultiplied 채널>alpha도 거절한다. 원 source 픽셀을 다시 rasterize한 결과나 색 정확성/사용자 품질 승인은 아니다. 전체 sequence pixel buffer를 만들지 않는다. 새 의존성은 없다. 원 inspection은 preimage-r236-pixel-inspection에 보존했다.

3085/3086fmt·3087R237 snapshot,3088fmt check·3089R238 snapshot 종료0. 고정 manifest가 필요한 실제 픽셀 시험을 ignored로 분리한 것이 R238이다.3090 첫 native strict는 chunks_exact_to_as_chunks 2건으로 종료1이며 원 로그와 pixels-before-r238-lint-fix.rs를 보존했다. 배열 chunk 순회만 수정했고3091fmt 종료0,3092R239 snapshot 진행 중이다. 실제 PNG/SDK 실행은 아직 미확정이다. R236 통합190/정상 bins와 구분한다.

3084 최종 사후 custody 종료0: 실제2501개 source content/identity unchanged. R236 통합 범위를 확정했다. 이전 문단의 진행 중 표기는 각 시점의 기록이다.

R236 통합:3078Product145·3079Core18·3080Native26·3081OS1 모두 execution_result=Ok/producer_integrity=Ok, 동일 R236 SOURCE.md SHA다. 결과 root runtime/verification/source-sdk-r236-exact190-r1. Core/Native/OS는 원622/624 compiler 산출물을 실제 재실행한 범위이며 새 컴파일이 아니다.3082정상2bins·3083fmt check 종료0, exe SHA가 고정본과 다시 같았다. 정상 exe/PDB 두 쌍을 fixed-r236/native-bins에 보존했다.3084사후 custody는 진행 중이다. GUI는 수동 실행하지 않았고 전체 CLI/GUI 인수·제작 executor0/5·AI사용자 품질0/9를 바꾸지 않는다.

R236 실제 CLI/SDK:3074combined no-run·3075실제 CLI/typed 원Jobs2완료·3076검사/SDK1·3077명령수/capability 회귀1 모두 종료0. 실제 root runtime/verification/source-sdk-r236-cli-r1/asset-build. CLI job `job-34c2314d1c6a46ebaf18e3e90f5bdf01`, typed job `job-b4e1034ab30349868341351bf2edd9f1`, publication SHA `17730d05ca01f2a92b505e70f70036b596281e286ecf75bb927216d72083e0e0`, config SHA `d3d19d4d69821df1c91391e8e5179f0bddff9ea15dfa9caade86223dbef66913`. 원 host EOF/exit0/not forced/family Empty/pipe+sdk joined/completed_total2는 실제 시험 assert로 확인했다. CLI+typed artifact inspection이 현재 새 산출물에서 실행됐다. 실제 정적 PNG/수동 mask·PTS null이며 AI/영상 품질 승인이 아니다.

고정 실행 파일: fixed-r236/native-bins CLI SHA `CA0C07ADA040A4B85FB8FB1AEDCAB90D0CC84EFF59D4D06995BFC69AB63CCC86`, GUI `6B33E2D0B97947965FC28440ACE31A8CD836F246F785281FC89D25975AF9AE29`. native-core lib `2492AACDC4F361F12A9C09262BEC795575A1E84EBA013408CD0026A8DFF9D5E6`, Jobs `A589F8F89F98938684D3B018C0E6F2DC21A1175F906A0CA756B6EBD499F96FAB`, unitCLI `9BEEBD4C3D80BF50DC4D38F5DD6E200B6BA263C0FB126FBAA78A6FD1DB29EEB8`. 현재는 exe만 보존했다. 지정190/정상 bins 명시 build/fmt/사후 custody는 후속 실행 중이다.

R236 진행 결과:3072native strict·3073default strict 종료0.3074combined lib/Jobs/unitCLI no-run 실행 중. inventory2501 SHA `CD6552F91A54BA6D71C035DF05FD87CAF380A130369FCAC4912E6D65C4D514E3`, SOURCE.md SHA `8EE1B43556A301A8A994568BCB4EEEB602254E80AB346A185F436466C9836098`. 실제 CLI/typed 실행과 사후 custody, 현재 소스 통합190/정상 bins는 아직 미확정이다.

R236 구현 중: application의 AssetArtifactInspector/typed 요청·관측 DTO와 `asset_artifacts.inspect`를 SDK에 연결하고, 명시 asset host의 고정 artifact root 검사기를 CLI host에 연결했다. 요청에는 경로가 없으며 timeout/idempotency/expected_revision은 거절한다. capabilities는 point-in-time/source_admitted=false/jobs_completion_authority=false/quality_approved=false를 표시한다. API 반환 값은 원 파일 lease가 아니며 직렬화 후의 파일 불변을 약속하지 않는다. 등록 명령 수는116이지만 기능 완성률로 쓰지 않는다. typed 정상/미설정/경로주입/timeout 및 기존 실제 CLI 시나리오의 조회 시험을 추가했다.3068/3069/3070fmt·3071snapshot 종료0.3072native strict 실행 중이고 실제 인수 전이다. 변경 전 파일은 preimage-r235-inspector-api에 보존했다. 새 의존성/모델/GUI/push는 없다.

3067 최종 사후 custody 종료0: 실제2500개 source content/identity unchanged. 다음 SDK/CLI 조회 포트 연결 전 R235 입력/시험 경계를 확정한다. 전체190/정상 bins의 최신 통합 근거는 R232이며 R235로 바꾸지 않는다.

R235 실제 결과:3064lib no-run 종료0. fixed-r235/native-core lib SHA `35B757F9CF48CF5A2928FDCC9B36148EB309A354E98C6B80212AB41064E0AF0C`.3065 기존 산출물 복사본의 의미/무결성1시험 종료0,3066 프로젝트/capture 회귀13 passed/0 failed/2 ignored 종료0이다. 두 ignored 실제 PNG 시험을 실행했다고 계상하지 않는다. 새 evidence는 runtime/verification/source-sdk-r235-inspection-r1이다.3067 사후 custody 실행 중. SOURCE.md SHA `9B26363A1715DAD937F87AF9F9C76B8CE582ADDD14BDCE8B12C5CE5AAEF39A5C`. 내부 manifest를 변형한 부정 시험은 outer SHA 거절만 검사하지 않고 `validate_contents`에 직접 입력해 의미 대응 거절을 확인했다. inner 출력 픽셀/PNG decode와 원 Jobs 재실행은 이번 시험 범위가 아니다.

R235 후속: 기존 capture의 deterministic 계획 계산을 `plan_publication` 순수 함수로 분리했다. 원 capture의 project admission/hash/원 lease 검사는 유지했다. 읽기 검사는 보유한 project/manifest 파일의 bytes에서 strict DTO를 읽고 canonical project pin, 재계산한 계획, frame/layer/request, geometry/UV/pivot, raw/PNG alpha 변환, color, 경고, ordered descriptor와 raw bytes 길이를 대조한다. 원본 PNG/mask 픽셀 재실행·PNG decode·사용자 품질 승인은 별도다.3059/3060fmt·3061snapshot·3062native strict·3063default strict 종료0. inventory2500 SHA `ADEA06F002ADB7948FCB82D581960802D2CF32730403F47D9DE687564AEEE2C5`.3064lib no-run은 실행 중이다. 모순 manifest의 plan/unit/file/geometry/approval 부정 시험을 추가했으며 아직 실행 전이다. 변경 전 파일은 preimage-r234-semantic에 보존했다.

3058 사후 custody 종료0: 실제2499개 source content/identity unchanged. 현재 변경의 전체190·정상 bins 재빌드는 아직 실행하지 않았다. 다음은 inner manifest/project의 의미 대응 검증과 공통 SDK/CLI 읽기 포트 연결이다.

후속 실제 결과:3056 lib no-run 종료0. 고정 lib exe SHA `C202634F0672BDAAAA098938372E088524A006A3612A2213A1509DB255B9FDB9`는 mirror runtime/verification/fixed-r234/native-core에 보존했다.3057 실제 검사1시험 종료0, 1 passed/0 failed. fresh evidence는 runtime/verification/source-sdk-r234-inspection-r1이며 R232 원본은 변경하지 않았다. 아래 실행 전 기록은 작성 시점의 이력이다.3058 사후 custody는 실행 중으로 아직 확정하지 않는다. SOURCE.md SHA `8155E274A81FCF8F03EC11D40DD85F89AC4D860F80E53D7D2362B63B40E093ED`.

R234 private overlay. 원본 R232 Jobs 발행 결과와 새로운 검사 구현의 검증 범위를 구분한다. 원 Jobs 완료·사용자 품질 승인 권한을 만들거나 복구하지 않는다. 기존 소유권/WIP/보류·canonical 미적용을 유지한다.

application/asset_artifacts는 엄격한 outer publication/실행 binding/이름·순서·중복·파일 수·총 bytes 한도를 검사한다. infrastructure/asset_build_jobs/inspection은 고정 local root와 원 directory/file guard를 보유하고 publication pin 및 각 파일 SHA/크기를 읽는다. 결과는 Clone/Deserialize가 없는 보유 값이며 공개 getter는 읽기 전용이다. 내부 manifest/project 의미 대응과 PNG 재디코딩, self-contained 원본 묶음, SDK/CLI 조회 연결은 아직 별도 미완료다.

R233 3050 snapshot 성공 후3051 native strict가 manual_is_multiple_of lint로 종료1했다. 첫 로그와 수정 전 파일을 preimage-r232-artifact-inspection에 보존했다. 해당 표현만 수정했다. R234 3052fmt check·3053snapshot·3054native strict·3055default strict 종료0. 기존 Meari 경고는 보존했다. inventory2499 SHA `6EA2C83DF6E5B4B96DD8AB3E192DE6F7C0B131806E7DA4F7EFF8F91001E7055C`.3056 lib no-run은 실행 중이며 성공으로 계상하지 않는다.

추가한 실제 파일 시험은 원 R232 산출물의 fresh 복사본만 변경한다. pinned 정상 읽기, reader drop 후 retained 파일 쓰기/rename 거절, 잘못된 SHA·extra entry·변조·경로 탈출·중복·quality=true 거절을 확인하도록 작성했다. 아직 실행 전이며 전체190/정상 bins/기능 인수의 성공으로 대체하지 않는다. 원본 파일은 삭제·수정하지 않는다.

기존 modootoday QA fixture·code-change-review-verification·verdict-table의 입력 고정/첫 실패 보존/실행 범위 구분을 적용한다. 정확한 출처와 SHA는 [자산 입력 검증](asset-input-admission-20261029.md)에 연결한다. 추론·다운로드·수동 GUI·push 보류 유지. 전체 제작 executor0/5·AI사용자 품질0/9 유지.
