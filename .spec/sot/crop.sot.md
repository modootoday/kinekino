---
id: sot:kinekino/crop
title: 숏폼 구간별 crop
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 숏폼 구간별 crop

현재 실제 범위 R166: [검증 정본](../../../verification/application/clip-section-crop-20261028.md)의 정수경계4·pinned synthetic RGB 전환1·원 source CLI/typed 두crop 여정1과 통합 gate가 통과했다. N 순번 연결은 synthetic 30frame의15/15 분할로 확인했다. 실제 VFR 전체·모든 key/해상도 조합의 RSS·기존 recipe/1 재열기·선택 자막/다른 제작·GUI/사용자 품질은 남는다. 아래 최초 구현/검증 전 기록은 당시 상태로 보존한다.

R166 보정: 새 crop_sections 계획은 기존 단일 crop/fit 직렬화와 달라 렌더 recipe를 manual-mp4-seekable-stdout/2로 구분한다. 저장 ClipDocument schema는 그대로이며 기존 원 프로젝트/불변 산출물을 수정하지 않는다. 새 실행 및 과거 recipe/1 재열기의 실제 증거는 각각 별도 기록한다.

2026-10-28 중앙 PM, R165 private overlay WIP. [승인 숏폼 계약](shorts.md)의 첫 버전 구간별 조정을 구현한다. 자막·추천/추적 crop·다중 클립/전환과 다른 제작 요구를 삭제하거나 이번 지원으로 계상하지 않는다.

crop key는 source_us부터 다음 key 전까지 유지하는 구간별 값이다. source clock의 정수 Up 변환과 원 decoded PTS 배열에서 각 key 이전 프레임 수를 계산한다. 원 source 시작/끝은 기존 반열린 구간이며 key가 같은 decoded frame에 매핑되면 나중 key가 이긴다. 끝의 제외 frame에만 걸리는 key는 이번 영상 frame에 영향을 주지 않는다. 이 매핑은 NativeClipBoundaryLease의 원 관측에 포함하고 저장 document key와 대조한다.

ManualClipPlan은 crop_sections에 source_us·threshold_pts·frame_offset·픽셀 crop·fit을 기록한다. 분기는 clip 전체의 같은 입력 frame/PTS를 유지하며 각각 고정 crop/fit을 적용한다. 고정 숫자뿐인 split/blend graph로 원 decoded frame 순번에서 결과 하나를 선택하고, 최종 fps는 합성 뒤 한 번만 적용한다. 구간별 STARTPTS/concat이나 float T 비교를 사용하지 않는다. 단일 crop graph는 기존 경로를 유지한다. encoder admission은 여러 key일 때 split/blend의 실제 inventory를 추가로 요구한다.

512MiB의 논리 분기 buffer 사전 한도를 사용한다. 계산은 output pixels×4×(3×key 수+4)이며 실제 프로세스 RSS 보증이 아니다. 초과하면 ResourceExhausted로 거절하고 원 deadline/출력 한도/heavy1은 유지한다. domain의128 key 한도는 그대로지만 모든 해상도와 key 수 조합을 무제한 지원한다고 표시하지 않는다. 실제 비용·최대 조합은 후속 실측 범위다.

선택 근거는 [FFmpeg 8.1.1 blend 원본](https://raw.githubusercontent.com/FFmpeg/FFmpeg/n8.1.1/libavfilter/vf_blend.c)의 frame_count_out 사용이다. 어댑터는 입력에서 전달된 frame 수가1부터 증가한다고 연결했으며 이를 실제 픽셀 시험으로 입증해야 한다. [같은 버전 interleave 원본](https://raw.githubusercontent.com/FFmpeg/FFmpeg/n8.1.1/libavfilter/f_interleave.c)의 시간 정렬/입력 대기·microsecond timebase를 검토했지만 구간별 trim 분기에서의 대기/정밀도 부담을 피하려고 채택하지 않았다. 이 부담은 코드 검토에 따른 판단이며 이번 pinned engine 실측 사실이 아니다.

검증은 정수/VFR/같은 frame/제외 end 정책 시험, Rust 생성 red/blue fixture를 pinned FFmpeg로 렌더·전체 RGB decode해0~14 frame 빨강/15~29 파랑을 검사하는 실제 어댑터 시험, 기존 Sintel CLI 프로젝트의15.5초 crop 전환/fit 변경·선택 audio·typed 렌더로 나눈다. 합성 fixture는 GUI 인수·원 source custody·색 정확도/사용자 품질 승인이 아니다. 아직 실행 전이다.

기존 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100), code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용해 source 정책/합성 engine 픽셀/원 Jobs CLI/사용자 품질 근거를 구분한다. 원 소유권/WIP/private overlay·원 증거·모든 보류를 유지한다.
