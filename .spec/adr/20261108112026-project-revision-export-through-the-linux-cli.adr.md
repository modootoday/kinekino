---
id: adr:kinekino/20261108112026
title: Project revision export through the Linux CLI
status: active
created: 2026-11-08T11:20:26Z
updated: 2026-11-16T12:00:00Z
rel:
  governs:
    - crates/kinekino-player/src/application/projects
    - crates/kinekino-player/src/application/api/projects.rs
    - crates/kinekino-player/src/infrastructure/projects
    - crates/kinekino-host/src/runner.rs
  depends_on: [sot:kinekino/export]
---

# Project revision export through the Linux CLI

## Context

로드맵 plan:kinekino/20261031153240의 M4-2(ST-6)다. M4 완료 기준은 기존 항목의 Linux CLI 경로가 Unsupported가 아니고 fixture 기반 테스트가 있는 것이다.

관측(2026-11-08, main `cf71db44`):

- `projects.export`는 `capabilities` 명령 목록에만 있고 dispatch·schema·어댑터가 없어 항상 `unsupported`였다.
- Linux에서 `--project-root`를 주면 러너가 `Rooted project storage requires Windows`로 거절했다. 그러나 `ProjectFiles` 어댑터는 OS 정책 포트(`os_policy`)를 거쳐 Linux에서도 컴파일되고 단위 테스트가 Linux에서 돈다. 막힌 곳은 러너의 `cfg(windows)` 분기뿐이다.
- 새 파일 게시 도우미 `kinekino_core::file_publication::publish`가 있다. 임시 파일에 쓰고 sync한 뒤 create-only 링크로 확정하며, 기존 파일이면 `revision_conflict`로 거절하고 원본을 건드리지 않는다. 자막·카드 내보내기가 이미 이 계약을 쓴다.
- sot:kinekino/export는 `ExportRequest(project_revision, items, preset, destination, collision_policy)`, 충돌 시 명시적 해결 전 시작 불가, `.part`→검증→원자적 확정, 기존 원본 보존을 요구한다.

권고안(루틴 진행): 새 기능을 만들지 않고, 이미 목록에 있는 `projects.export`를 기존 저장소·게시 도우미로 완성하고 Linux 러너 차단만 푼다.

## Decision

1. **Linux에서 `--project-root`를 연다.** 러너는 Windows와 같은 `ProjectFiles::open`을 쓴다. `--proposal-root`는 이번 조각에서 Windows 전용으로 둔다(라이브 제안 캡처는 Linux에서 계속 Unsupported).
2. **`projects.export` 계약(첫 단계).**
   - 요청 payload: `project_id`, `revision`(필수, 고정된 불변 리비전), `output_path`(필수, 절대 경로, 새 파일 이름). context의 `expected_revision`·`timeout_ms`·`idempotency_key`는 거절(`invalid_argument`).
   - 출력 문서: UTF-8 JSON 한 개 `{"format":"kinekino.project-export","format_version":"1","revision_sha256":…,"project":<ProjectRevision>}`. 입력 리비전을 그대로 담고 미디어·URL을 열지 않는다. `revision_sha256`은 저장소의 정본 해시와 같다.
   - 충돌: `collision_policy`는 `reject` 하나만. 대상이 이미 있으면 `revision_conflict`, 기존 파일 바이트는 보존. 덮어쓰기·새 이름 제안·건너뛰기는 이번 단계에 없다(응답 `overwrite:false`).
   - 오류: 저장소 없음 `unsupported`, 잘못된 식별자·리비전 0·누락·상대/빈 경로 `invalid_argument`, 없는 프로젝트·리비전·출력 디렉터리 `not_found`, 권한 `permission_denied`. 실패 시 대상 경로에 파일이 생기지 않고 게시 도우미의 임시 파일은 남지 않는다.
   - 검증: 내보내기 전 `projects.inspect`와 같은 저장 기록·출처 검증을 거친다. 응답에 `output_sha256`, `output_bytes`를 준다.
3. **계층.** application `projects::export`(유스케이스)와 `ProjectExportTarget` 포트, infrastructure `ProjectExportFiles`(게시 도우미 위임), api dispatch·schema·capabilities. 호스트 코드에 사이트별 용어를 쓰지 않는다.
4. **하지 않는 것.** 여러 항목·preset·카드/자막 렌더링 묶음 내보내기, 덮어쓰기, GUI, import, 원격(`--endpoint`) 경로, 실제 추론·네트워크.

## Consequences

- Linux CLI에서 `projects.create/inspect/save_revision/reconnect_source/export`가 `--project-root`로 동작하고 CLI fixture 테스트로 잠긴다.
- 남은 일: 명시적 덮어쓰기·새 이름 정책(OUT-04), 다항목·preset 내보내기(OUT-07/08), Linux `--proposal-root`, 내보낸 문서 import.
- skill: modoo.today skills 조회(파일 내보내기 계약)했으나 맞는 도메인 스킬 없음 → 미적용.

## Record

- 2026-11-16: `--model-root` opens on Linux. The runner refused it there ("requires Windows path guards"), but `LocalModelCatalog` reads through the same `immutable_storage_paths` guards as the project stores, and those go through the OS policy port. On Linux the guards hold descriptors and refuse symlinks. They do not stop another process from replacing a file the way Windows share modes do; `models.verify` hashes the descriptors it opened, so a concurrent replacement cannot change the bytes it reports. A new cross-platform CLI test (`tests/model_catalog_root_cli.rs`) runs `models.list`, `models.inspect` and `models.verify` through the normal CLI and matches each against the typed SDK on the same root. It also checks a stale catalog pin, and that the stored files are left unchanged. It failed on Linux before the runner change. The detailed Windows custody test stays Windows-only because its file-identity guard uses Windows APIs.
