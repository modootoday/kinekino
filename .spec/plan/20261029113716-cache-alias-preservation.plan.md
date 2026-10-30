---
id: plan:kinekino/20261029113716
title: source·producer pin·alias 보존 준칙
status: active
created: 2026-10-29
updated: 2026-10-29
---
# source·producer pin·alias 보존 준칙

2026-10-24 launcher gate의 alias 하드링크 원복 사고를 반영한 후속 절차다. 사고 원문과 제한 수리는 `runtime/verification/product-gui-launch-r1/RESTORE-INCIDENT.md`/관련 JSON을 보존한다. 새 운영/검증 도구 구현이나 공유 cache 변경을 허가하는 문서가 아니다.

1. **입력**: 중앙 lease·canonical/library/lock·private source·현재 S/T 반환·carry fixed의 정확 핀을 먼저 대조한다. 필요한 승인 경로만 S에 동기화한 뒤 새 admission baseline을 별도 고정한다. 변경 byte만 fresh mtime으로 overlay하고 원 before bytes/mtime·신규 부재를 기록한다. 이전 baseline과 새 admission을 섞지 않는다.
2. **보호**: 실행 전 known/alias/fixed마다 actual path/hash/bytes/mtime/FileID를 기록한다. backup은 원본과 다른 FileID의 독립 사본이어야 한다. 하드링크 backup으로 보호됐다고 표시하지 않는다. fsutil hardlink list가 미지원이면 전체 link membership을 확인했다고 쓰지 않는다. 이름이 다른 두 경로도 FileID가 같으면 공유 파일로 취급한다.
3. **producer**: Cargo JSON의 package/target/profile/features/executable/filenames를 실제 산출물과 연결한다. 실제 시험 BIN은 새 독립 고정 사본으로 만들고 producer/copy hash·FileID·fresh를 기록한다. alias가 바뀌었으면 복원 **전에** 해당 새 producer exe/pdb 모두 독립 사본과 mtime/hash/actual link ID를 보존한다. 원 alias backup과 새 producer backup은 역할을 구별한다.
4. **절대 금지**: 기존 alias에 직접 Copy-Item/overwrite로 원복하지 않는다. 공유 inode의 deps producer까지 덮을 수 있다. 같은 이유로 기존 cache 파일 전체를 오래된 사본으로 rollback하지 않는다. 이전의 FileID 불변 주장과 충돌하면 오류/예외를 명시하고 원자료를 보존한다.
5. **alias 복원**: writer/원 owner가 정리된 뒤 정확 alias 한 파일의 경로·reparse 여부·현재 FileID를 확인한다. 원 byte/mtime의 별도 새 파일을 만들고 hash를 확인한 뒤, 승인된 alias 이름 하나를 unlink→새 파일로 교체해 분리한다. 전체 디렉터리 삭제/계산된 광범위 이동0이다. alias FileID가 바뀌는 사실을 기록한다. producer 경로는 그대로 두며 원 native bytes/mtime/FileID가 보존됐는지 대조한다.
6. **오류 시**: 이미 write-through가 발생했으면 affected actual path·FileID 쌍·원 복사 동작·수리 전후 snapshot을 보존한다. own Cargo JSON과 원복 전 보존 사본의 SHA로 입증된 exact producer만 중앙에 보고해 제한 복구한다. unknown cache/다른 링크 경로를 임의 덮거나 삭제하지 않는다. 새 build/test로 최초 오류를 숨기지 않는다. 증거가 부족하면 복구 완료를 선언하지 않는다.
7. **반납**: source overlay 원복을 admission 기준으로 확인하고 runtime 증거는 원 PID/receipt로 귀속해 보존한다. 고정 BIN bytes/mtime/FileID·known/alias 예외·T변경/new·root/library/index/lock을 구분해 감사한다. PID 숫자의 재사용은 원 terminal 이후 생성 시각으로 구분하며 다른 process를 종료하지 않는다. own worker/handle/pending 정리가 확인된 뒤 슬롯을 명시 반환한다. 역사적 known2 미귀속·새 alias FileID 예외는 해결 전까지 유지하며 `all_cache_protected=true`를 주장하지 않는다.

공통 Rust owner/검증 정책을 새 PowerShell/JavaScript/Python 코드로 대체하지 않는다. 현재 파일 작업은 승인된 metadata·copy/restore orchestration 범위이며 장기 자동화가 필요하면 Rust의 파일 identity/별도 source adapter 계약을 정확 경로와 함께 먼저 제안한다.
