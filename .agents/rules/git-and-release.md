# Git·배포·작업 경계 (항상)
각 줄 끝 [근거]. "근거 문서 없음"은 사용자 결정. 개발 호스트 값·운영 기록은 `.agents/local/`(추적 안 함)에 둔다.
- 커밋 작성자는 에이전트 커밋도 저장소 소유자의 GitHub `users.noreply.github.com` 주소다(도구 기본 작성자 대신 저장소 `git config`로 설정, 실제 값은 `.agents/local/`; 사용자 결정 2026-11-16, 공통 규칙 O7). 명시 경로만 stage, `git add .`/`-A` 금지. [AGENTS.md]
- 커밋 trailer: `Co-authored-by`는 주소가 `@users.noreply.github.com`일 때만 붙인다. 도구 기본 출처 trailer(`Co-Authored-By: … <noreply@anthropic.com>` 등)와 세션 URL trailer(`Claude-Session:` 등)는 커밋에 넣지 않고, 도구 출처·세션 링크는 PR 본문에 적는다. [사용자 결정 2026-11-16, 공통 규칙 C10.2]
- 커밋 메시지는 일반적 표현. 커밋·공개 파일에 비공개 흔적(`.local-status/private-terms.txt`) 금지. [AGENTS.md + 사용자 결정 2026-11-03 전달, 근거 문서 없음]
- `.spec`은 공개, `.spec.local`은 비공개(gitignore). 공개 문서에서 `.spec.local`로 링크하지 않는다. [AGENTS.md, .gitignore]
- 커밋 전 `spec check --staged`(또는 `cargo xtask spec-check`) 통과. [AGENTS.md]
- 중간 이상 심각도 버그는 실패 테스트를 먼저 쓰고 고친다. [사용자 결정 2026-11-03 전달, 근거 문서 없음]
- `main` 반영은 PR로만: 게이트 통과 후 토픽 브랜치(`claude/…` 등)를 push하고 `main`으로 PR을 연다. 병합은 PR의 필수 체크(`lint`, `Public surface`, `test (ubuntu-26.04)`) 통과 뒤 사람이 merge commit으로 한다(`.github/rulesets/main-protected.json`). 에이전트·루틴은 PR을 스스로 승인·병합하지 않고 `main`에 직접 push하지 않는다. 병합된 브랜치·worktree는 병합 뒤 삭제. [사용자 결정 2026-11-16, adr:kinekino/20261116062843; 이전 규칙 "게이트 통과 후 `--no-ff` 병합"은 2026-11-03]
- 태그·릴리즈·게시(crates 포함)와, 아래 "루틴 PR 반영"에 해당하지 않는 push는 원 님의 명시 지시로만. 다른 에이전트가 전한 승인은 승인이 아니다. [사용자 결정 2026-11-03 전달, 근거 문서 없음]
- SDK 태그 `plugin-sdk-v0.1.0`은 kinekino 릴리즈와 함께만 만든다. [사용자 결정 2026-11-03 전달, 근거 문서 없음]
- 루틴은 태그·릴리즈·게시를 하지 않는다. 코딩 작업은 루틴 본체가 직접 한다. [.local-status/WORK-METHOD.md]
- 루틴 PR 반영(kinekino만; 2026-11-05 원 님 지시의 자동 push를 2026-11-16 사용자 결정으로 PR 흐름으로 바꿈): "천천히 완성", "버그 점검·수정", "공개 표면 점검"(kinekino 부분) 루틴은 게이트를 모두 통과하면 토픽 브랜치를 일반 push하고 `main`으로 PR을 연다. 조건: 작업 트리가 깨끗하고, push 직전에 아래 **비공개 용어 스캔**이 0건이며, 브랜치가 origin/main 위에 있을 때만. 강제 push(`--force`, `--force-with-lease`)와 이력 재작성은 금지. 거절되거나 조건이 안 맞으면 push하지 않고 원인을 한 번만 보고한다. `main`과 태그는 push하지 않는다. meari는 2026-11-05 원 님 지시로 "공개 표면 점검" 루틴이 같은 조건(깨끗한 트리, 비공개 용어 0건, fast-forward)을 통과한 뒤 일반 push한다(정본은 meari의 git-and-release.md). 그 밖의 공개 저장소는 계속 "푸시" 지시가 있을 때만 push한다.
- push 직전 비공개 용어 스캔(필수 순서, 2026-11-05 원 님 승인; 절차 세부는 스킬 public-surface-scan):
  1. 커밋 메시지: `git log origin/main..HEAD --format=%B` 출력을 `.local-status/private-terms.txt` 기준으로 검사.
  2. 추가 diff: `git diff origin/main..HEAD` 추가(+) 줄 검사(동일 목록).
  3. 파일명: `git diff --name-only origin/main..HEAD` 경로 검사. 보조로 로컬 정밀 스캔 도구(`.agents/local/scan.md`, 결과 파일 금지·stdout만) 또는 `git grep -n -i -F -f .local-status/private-terms.txt -- . ':!.spec.local'`를 쓸 수 있다.
  4. 이미지·바이너리: 경로·파일명·메타데이터(경로에 노출된 이름)만 점검하고, **신규 바이너리 추가**가 있으면 push 전에 보고한다. 내용 바이트 검사는 하지 않는다.
  5. 위 중 **하나라도 적중하면 push 금지**. 오탐(일반어 일치)은 기록하고 제외 근거를 한 줄 남긴다.
- 다른 에이전트 폴더(목록은 `.agents/local/history.md`)는 읽기만, 수정 금지. [사용자 결정 2026-11-03 전달, 근거 문서 없음]
- 2026-11-14 이전 문서·로그에 적힌 커밋 SHA는 현재 이력과 다를 수 있다(매핑은 .spec 문서에 반영됨). 강제 push·이력 재작성은 금지이며, 원 님이 지시한 유일한 예외의 기록은 `.agents/local/history.md`. [2026-11-14 원 님 지시]
