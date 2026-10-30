---
id: sot:kinekino/review-producer-host
title: 명시 review producer/host 시작 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 명시 review producer/host 시작 계약

2026-10-27. `player-cli --serve-review --review-host-config ABS_JSON --review-host-config-sha256 SHA256`는 기존 시작 모드와 섞지 않는 명시 opt-in이다. duplicate·unknown·missing flags를 거절한다. 설정은 ordinary local file의 root/file guard·최대64KiB·SHA256을 보유한 startup 입력이다. 임의 manifest/admission/Verified authority를 역직렬화하는 경로가 아니다.

설정의 필수 필드는 source_root/published_root/user_root/runtime_root, case/media, video/audio/capture/playback 각각 path/sha256, video_start_us/video_end_us, pcm_start_pts/pcm_end_pts, start_us/end_us, job_memory_bytes/affinity_mask, production_total_ms/playback_total_ms/cleanup_ms, max_source_bytes/max_output_bytes/max_package_bytes/playback_raw_output_bytes/startup_timeout_ms/max_history다. serde default를 제공하지 않으며 unknown fields를 거절한다. source 선택과 수치 범위는 공통 SDK/domain 정책으로 재검사한다. 단위는 source microseconds와 원 PCM1/16000 PTS로 구분한다.

공통 `prepare_timeline`은 원 package-custodied source를 검사·보유하고 하나의 serial producer session/원 deadline 아래 video→audio PCM→raw PCM replay/clip lineage→origin/start/end capture→bundle→새 immutable package 발행을 수행한다. component guards는 발행까지 동시에 유지한다. source/runtime/destination/명시 excluded roots의 겹침과 capture 범위를 child 생성 전 거절한다. 기존 private publisher token만 반환하며 그것으로 PublishedTimelineResolver를 생성한다. 기존 archive 진입점과 원 package/판정은 변경하지 않는다.

component 생성 후 실패는 원 TimelineProcessFailure의 pending을 원 stack에서 poll_cleanup한다.10초 cleanup 관측 한도는 operation deadline 갱신이 아니다. 물리 종료가 확인되면 원 error를 반환하고, 미완료면 diverging handler가 원 source/tool/session/Job을 유지한 채 비정상 종료 또는 계속 관측해야 한다. 제품 CLI는 원 owner를 보유한 비정상 process exit를 사용하며 정상 정리/성공이라고 기록하지 않는다. 이 오류 분기의 전수 fault injection은 아직 미검증이다.

발행 후 producer slot을 해제하고 같은 resolver Arc로 SDK 저장·playback을 조립한다. ready에는 current endpoint/PID/new case/evidence 및 quality_approved=false가 포함된다. stdin은 ready 후 host 수명을 제어하며 EOF/I/O 실패에서 원 transport/session cleanup driver를 사용한다. 준비 중 stdin 취소·강제 crash 전수는 아직 지원/검증 완료로 표시하지 않는다. 설정 읽기와 준비는 동기 실행이며 수동 GUI·추론·네트워크 다운로드를 시작하지 않는다.

재시작은 새 producer를 실행하고 fresh host scope를 만든다. 이전 session/engine 참조를 재개하지 않는다. 새 package identity가 달라지면 과거 사용자 association을 자동 migration/재승인하지 않는다. 같은 source를 다시 관측했다는 이유로 과거 AI 품질 승인이나 case 권위를 승계하지 않는다.

CLI/SDK 연결의 실제 시험은 별도 고정 입력에서 수행하고 근거 문서로 연결한다. GUI/actual9와 사용자 품질 승인은 이 계약의 완료 조건을 대체하지 않는다.
