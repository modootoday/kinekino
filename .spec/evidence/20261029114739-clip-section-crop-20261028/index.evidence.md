---
id: evidence:kinekino/20261029114739
title: 숏폼 구간별 crop 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 숏폼 구간별 crop 검증

R166 최종 사후 확인:2535 원 handle 종료0,2433파일 내용·identity 동일. 같은 R166 입력에서 정수경계4·실제픽셀1·실제CLI/typed 두crop 여정1·단일crop명령1·metadata2·기본strict/no-run/fmt·정상2bins·지정190·사후 검사를 완료했다. 구간 crop의 이 선택 경로를 검토 기준선으로 고정하며 미검증 범위와 전체 goal은 유지한다. 다음 승인 첫 버전의 주요 공백은 선택 자막의 source/cue/style/font admission과 같은 원 Jobs 렌더 연결이다.

R166 실제 결과:2523 no-run 종료0/2분54초, fixed-r166/native-core lib SHA1117B7190A598C916A56358FC5107A082B58FEB1E7CE20ACEDFF59BA14FE6B12/Jobs SHA5DF2179C00B24D0F44DA852E938F0F49F0FEC693259EB3F52F0906E3F3EE524F와PDB 보존.2524 정수/VFR/같은 frame/제외 end 경계4시험 통과.2525 실제 pinned synthetic pixel1시험/0.27초/종료0으로30frame 중0~14 빨강/15~29 파랑을 확인해 N의 one-based 연결을 검증했다. 원 RGB/Matroska/MP4/decoded RGB/argv/verification은 runtime/verification/source-sdk-r166-section-pixels-r1에 보존했다. 이는 합성 engine 어댑터 증거이며 GUI/원 source 색·품질/Jobs 검증이 아니다.

2526 정상2bins 종료0, fixed-r166/native-bins exe/PDB 보존(CLI SHA65BEF30208891BA3650E79E6BBD90AF5A90A33DB6580345170FEF8824806D452,GUI SHA660497F864381B164FB30F3DF0918DE2DE364CF59D1D23CAD5F6E12051A49C60).2527 실제 CLI/typed 여정1시험/21.54초/종료0:13 CLI·source 관측/새 프로젝트 생성/수정 저장·두crop·원 Jobs2렌더·동일 key/핀 오류·정상 EOF/completed2를 확인했다. runtime/verification/source-sdk-r166-cli-r1/clip-render,job-04244bfb00674f6396fe2aafd154c565/typed job-a1376f33333745cbaf70d812434ac648,첫 manifest41732553e26a0d149a38acf38997a3b52112ae529ca046c8a7288d94d024c5fc. recipe/2 계획에 원15000PTS/frame0/full1280×544 fit,15500PTS/frame12/left320·640×544 fill을 확인했다. 출력1080×1920/30frames/영상·AAC·컨테이너1초/전체decode,색/품질 false. 실제 VFR 전환 전체와 source 색 정확도는 이번 근거가 아니다.

2528 기존 단일crop 명령1/2529 metadata·movie2/2530fmt check 통과.2531 Product145/2532 Core18/2533 Native26/2534 OS1의execution_result와producer_integrity 모두Ok(())/동일SOURCE pin을 확인했다. Meari는 기존고정producer 재실행이다.2535 단독custody 실행 중이며 최종 동일성 확인 전 전체gate완료로 계상하지 않는다. 자막·다른 제작·전체키/해상도/RSS·오류/취소/복구/ACK·과거recipe 재열기·GUI/사용자품질 미검증과 보류·소유권/WIP/private overlay를 유지한다.

현재 진행 핸들:2523 no-run exec session68617. 다음 로그는2524부터 사용한다. 종료 후 fixed-r166/native-core에 lib/Jobs exe/PDB를 고정하고 application::clip_render::boundaries::tests:: 정책4시험, infrastructure::clip_renderer::tests::sections::pinned_section_crop_switches_exactly_between_decoded_frame_14_and_15 ignored 실제 픽셀, 정상 CLI 고정 후 clip_render::configured_cli_and_typed_sdk_render_through_original_jobs_and_owned_eof의 두crop 원 Jobs 여정을 순차 검증한다. 픽셀 시험에는 새 CLIP_SECTION_EVIDENCE_DIR와 기존 pinned encoder 환경을 사용한다. 실제 전환과 N 순번 연결을 확인하기 전 완료로 계상하지 않는다.

R166 입력 고정:2521 snapshot 종료0,2433파일/inventory02534C8A7EACE4162EC41F53E218F8338A489EF7725D6409AF1F68C9DF03EDDF,SOURCE SHA E03B25709DE2EE78FB019B85068550DA62B38E4E93823922E53A69380C0862E5.2522기본all-targets strict 종료0/29.27초.2523 lib/Jobs/CLI no-run을 시작했으며 아직 실제 정책/픽셀/CLI 시험 전이다. 입력 고정 이후 source 변경 없이 같은 원 producer의 종료를 확인한다.

R165 첫 strict 실패:2518 snapshot 종료0,2433파일/inventory5059B8FCEAAB170C7A21C538ABE6E7E985BFB73B3ABA05F81CC93AFF218B8BAB,SOURCE SHA E346B25D5892D816C2E587FF3357978B46C314CD6464F5287357CAD1ECCC085C.2519 기본all-targets strict는 새 픽셀 시험의 chunks_exact_to_as_chunks lint1건으로 종료1이다. 원 로그/input-r165와 preimage-r165-pixel-loop-recipe에 기존 시험/renderer mod를 보존했다. 고정 frame 크기 순회를 as_chunks로 수정하고 crop_sections 직렬화 구조를 기존 recipe/1과 구별하기 위해 새 렌더 recipe를 manual-mp4-seekable-stdout/2로 올렸다. 기존 프로젝트/원 manifest bytes를 덮어쓰거나 이관하지 않는다. cross-version 재열기 실제 검증은 별도 범위다.

R166은2520fmt 종료0,2521 snapshot을 exec session87674에서 실행 중이다. 새 정책·합성 픽셀·원 Jobs CLI 실제 성공과 새 통합 gate는 아직 없다. 다음은 boundary 정책4시험, pinned_section_crop_switches_exactly_between_decoded_frame_14_and_15 ignored 실제 픽셀 시험(새 CLIP_SECTION_EVIDENCE_DIR와 기존 pinned CLIP_RENDER_ENCODER/SHA), 두crop 구간으로 확장한 기존 CLI 여정이다.

2026-10-28 중앙 PM, R165 WIP. [계약](../../player/application/production/clip-section-crop.md)에 따라 source PTS→crop frame 순번·계획/명령 분기·inventory·원 Jobs admission을 확장했다. preimage-r164-section-crop에 기존 application/native command/시험·CLI 여정을 보존했다. 새 경계 정책 시험과 실제 합성 RGB 전환 시험, 기존 CLI 두 crop 구간 렌더 시험을 추가했다.

2515~2517fmt 종료0,2518 input-r165 snapshot은 exec session64458에서 진행 중이다. 기본 strict/no-run·새 정책/실제 픽셀·CLI·통합 gate는 아직 미검증이다. 최신 완료 기준선R164와 원 소유권/WIP·private overlay·첫 실패/증거·C-only/heavy1·기존 보류·품질0/9를 유지한다. 새 구간 crop을 지원 완료로 표시하지 않는다.
